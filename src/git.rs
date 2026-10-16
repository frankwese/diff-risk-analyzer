use crate::model::{
    AnalysisError, Budget, ChangedLine, Comparison, FileChange, InputTotals, MAX_BLOB, MAX_FILES,
    MAX_LINES, MAX_TOTAL, Side, valid_oid, validate_revision_endpoint,
};
use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub struct Git {
    executable: PathBuf,
    root: PathBuf,
    budget: Budget,
    supervised: bool,
}

impl Git {
    pub fn open(cwd: &Path, budget: Budget) -> Result<Self, AnalysisError> {
        let executable = std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|p| p.join("git"))
                    .find(|p| p.is_file())
            })
            .ok_or_else(|| AnalysisError::new("GIT_UNAVAILABLE", "Git >=2.49 is required on PATH"))?
            .canonicalize()
            .map_err(|_| {
                AnalysisError::new("GIT_UNAVAILABLE", "Git executable could not be resolved")
            })?;
        let mut git = Self {
            executable,
            root: cwd.to_path_buf(),
            budget,
            supervised: false,
        };
        let version = git.run(&["--version"], b"", 4096)?;
        let version = String::from_utf8_lossy(&version);
        let numbers: Vec<_> = version
            .split_whitespace()
            .nth(2)
            .unwrap_or_default()
            .split('.')
            .take(2)
            .filter_map(|p| p.parse::<u32>().ok())
            .collect();
        if numbers.len() != 2 || (numbers[0], numbers[1]) < (2, 49) {
            return Err(AnalysisError::new(
                "GIT_UNAVAILABLE",
                "Git >=2.49 is required",
            ));
        }
        let root = git
            .run(&["rev-parse", "--show-toplevel"], b"", MAX_BLOB)
            .map_err(|error| revision_error(error, "run inside a local Git repository"))?;
        // Git emits one trailing LF; retain embedded newlines and raw path bytes.
        let root = root.strip_suffix(b"\n").unwrap_or(&root);
        git.root = PathBuf::from(OsStr::from_bytes(root));
        if !git.root.is_absolute() {
            return Err(AnalysisError::input(
                "repository root could not be resolved",
            ));
        }
        Ok(git)
    }

    pub(crate) fn open_supervised(cwd: &Path, budget: Budget) -> Result<Self, AnalysisError> {
        let mut git = Self::open(cwd, budget)?;
        git.supervised = true;
        Ok(git)
    }

    pub fn run(
        &self,
        args: &[&str],
        input: &[u8],
        max_output: usize,
    ) -> Result<Vec<u8>, AnalysisError> {
        self.run_os(
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            input,
            max_output,
        )
    }
    fn run_os(
        &self,
        args: &[OsString],
        input: &[u8],
        max_output: usize,
    ) -> Result<Vec<u8>, AnalysisError> {
        self.budget.check()?;
        let mut cmd = Command::new(&self.executable);
        cmd.current_dir(&self.root);
        for (key, _) in std::env::vars_os() {
            if key.as_bytes().starts_with(b"GIT_") {
                cmd.env_remove(key);
            }
        }
        cmd.env("LC_ALL", "C")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ALLOW_PROTOCOL", "")
            .args([
                "--no-pager",
                "--no-optional-locks",
                "--no-lazy-fetch",
                "--no-replace-objects",
                "-C",
            ])
            .arg(&self.root)
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.attributesFile=/dev/null",
            ])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if !self.supervised {
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().map_err(|_| {
            AnalysisError::new("GIT_UNAVAILABLE", "Git process could not be started")
        })?;
        let overflow = Arc::new(AtomicBool::new(false));
        let io_failed = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        let process_group = child.id() as i32;
        let read_bounded = |mut stream: Box<dyn Read + Send>,
                            limit: usize,
                            overflow: Arc<AtomicBool>,
                            cancelled: Arc<AtomicBool>,
                            io_failed: Arc<AtomicBool>| {
            thread::spawn(move || {
                let mut output = Vec::new();
                let mut buf = [0; 8192];
                loop {
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }
                    let n = match stream.read(&mut buf) {
                        Ok(n) => n,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => {
                            io_failed.store(true, Ordering::Relaxed);
                            return Err(e);
                        }
                    };
                    if n == 0 {
                        break;
                    }
                    if output.len() + n > limit {
                        overflow.store(true, Ordering::Relaxed);
                        return Err(std::io::Error::other("output limit"));
                    }
                    output.extend_from_slice(&buf[..n]);
                }
                Ok::<_, std::io::Error>(output)
            })
        };
        let stdout_pipe = child.stdout.take().expect("stdout pipe");
        let stderr_pipe = child.stderr.take().expect("stderr pipe");
        let mut stdin = child.stdin.take().expect("stdin pipe");
        use std::os::fd::AsRawFd;
        for fd in [
            stdout_pipe.as_raw_fd(),
            stderr_pipe.as_raw_fd(),
            stdin.as_raw_fd(),
        ] {
            if let Err(error) = nonblocking(fd) {
                if self.supervised {
                    let _ = child.kill();
                } else {
                    terminate_group(process_group);
                }
                let _ = child.wait();
                return Err(error);
            }
        }
        let stdout = read_bounded(
            Box::new(stdout_pipe),
            max_output,
            overflow.clone(),
            cancelled.clone(),
            io_failed.clone(),
        );
        let stderr = read_bounded(
            Box::new(stderr_pipe),
            MAX_BLOB,
            overflow.clone(),
            cancelled.clone(),
            io_failed.clone(),
        );
        let input = input.to_vec();
        let writer_cancelled = cancelled.clone();
        let writer_failed = io_failed.clone();
        let writer = thread::spawn(move || {
            let mut remaining = input.as_slice();
            while !remaining.is_empty() {
                if writer_cancelled.load(Ordering::Relaxed) {
                    return Ok(());
                }
                match stdin.write(remaining) {
                    Ok(0) => {
                        writer_failed.store(true, Ordering::Relaxed);
                        return Err(std::io::Error::other("closed Git input"));
                    }
                    Ok(n) => remaining = &remaining[n..],
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(e) => {
                        writer_failed.store(true, Ordering::Relaxed);
                        return Err(e);
                    }
                }
            }
            Ok(())
        });
        let mut exit_status = None;
        let result = loop {
            if overflow.load(Ordering::Relaxed) {
                break Err(AnalysisError::new(
                    "LIMIT_EXCEEDED",
                    "Git output exceeded the input limit",
                ));
            }
            if let Err(e) = self.budget.check() {
                break Err(e);
            }
            if io_failed.load(Ordering::Relaxed) {
                break Err(AnalysisError::new(
                    "GIT_IO_FAILED",
                    "Git pipe operation failed",
                ));
            }
            if exit_status.is_some()
                && stdout.is_finished()
                && stderr.is_finished()
                && writer.is_finished()
            {
                break if exit_status.unwrap() {
                    Ok(())
                } else {
                    Err(AnalysisError::new(
                        "GIT_FAILED",
                        "Git operation failed; verify local objects and revisions",
                    ))
                };
            }
            if exit_status.is_some() {
                thread::sleep(Duration::from_millis(2));
                continue;
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_status = Some(status.success());
                    if !status.success() {
                        break Err(AnalysisError::new(
                            "GIT_FAILED",
                            "Git operation failed; verify local objects and revisions",
                        ));
                    }
                }
                Ok(None) => thread::sleep(Duration::from_millis(2)),
                Err(_) => {
                    break Err(AnalysisError::new(
                        "GIT_FAILED",
                        "Git process status could not be read",
                    ));
                }
            }
        };
        if result.is_err() {
            if self.supervised {
                let _ = child.kill();
            } else {
                terminate_group(process_group);
            }
            cancelled.store(true, Ordering::Relaxed);
        }
        let _ = child.wait();
        let written = writer
            .join()
            .map_err(|_| AnalysisError::new("GIT_FAILED", "Git input failed"))?;
        let output = stdout
            .join()
            .map_err(|_| AnalysisError::new("GIT_FAILED", "Git output failed"))?;
        let errors = stderr
            .join()
            .map_err(|_| AnalysisError::new("GIT_FAILED", "Git error output failed"))?;
        result?;
        if overflow.load(Ordering::Relaxed) {
            return Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "Git output exceeded the input limit",
            ));
        }
        written.map_err(|_| AnalysisError::new("GIT_FAILED", "Git input could not be written"))?;
        errors
            .map_err(|_| AnalysisError::new("GIT_FAILED", "Git error output could not be read"))?;
        output.map_err(|_| AnalysisError::new("GIT_FAILED", "Git output could not be read"))
    }

    fn resolve(&self, revision: &str) -> Result<String, AnalysisError> {
        let output = self
            .run(
                &[
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("{revision}^{{commit}}"),
                ],
                b"",
                4096,
            )
            .map_err(|error| {
                revision_error(
                    error,
                    "revision does not resolve to a locally available commit",
                )
            })?;
        let oid = String::from_utf8(output)
            .map_err(|_| AnalysisError::input("invalid revision object identifier"))?
            .trim()
            .to_owned();
        if !valid_oid(&oid) {
            return Err(AnalysisError::input("invalid revision object identifier"));
        }
        Ok(oid)
    }

    pub fn compare(&self, base: &str, target: &str) -> Result<Comparison, AnalysisError> {
        validate_revision_endpoint(base)?;
        validate_revision_endpoint(target)?;
        let base = self.resolve(base)?;
        let target = self.resolve(target)?;
        let raw = self.run(
            &[
                "diff-tree",
                "-r",
                "--no-commit-id",
                "--raw",
                "-z",
                "--no-abbrev",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "--no-relative",
                "--ignore-submodules=dirty",
                "--submodule=short",
                "--find-renames=100%",
                "--no-rename-empty",
                "-l1000",
                "-O/dev/null",
                &base,
                &target,
                "--",
            ],
            b"",
            MAX_TOTAL,
        )?;
        let mut files = parse_raw(&raw)?;
        let mut totals = InputTotals::default();
        for file in &mut files {
            self.budget.check()?;
            if file.exact_rename() {
                continue;
            }
            for mode in [&file.old_mode, &file.new_mode] {
                if !["000000", "100644", "100755"].contains(&mode.as_str()) {
                    return Err(AnalysisError::unsupported(file.path()));
                }
            }
            if file.old_mode != "000000"
                && file.new_mode != "000000"
                && file.old_mode != file.new_mode
            {
                return Err(AnalysisError::unsupported(file.path()));
            }
            file.old = self.blob(&file.old_oid, &mut totals)?;
            file.new = self.blob(&file.new_oid, &mut totals)?;
            for bytes in [&file.old, &file.new] {
                if bytes.contains(&0)
                    || std::str::from_utf8(bytes).is_err()
                    || bytes.starts_with(b"version https://git-lfs.github.com/spec/v1")
                {
                    return Err(AnalysisError::unsupported(file.path()));
                }
            }
            file.lines = diff_lines_with_budget(
                &file.old,
                &file.new,
                &self.budget,
                totals.remaining_lines(),
            )?;
            totals.add_lines(file.lines.len())?;
        }
        self.budget.check()?;
        Comparison::new(self.root.clone(), base, target, files)
    }

    fn blob(&self, oid: &str, totals: &mut InputTotals) -> Result<Vec<u8>, AnalysisError> {
        if oid.bytes().all(|b| b == b'0') {
            return Ok(vec![]);
        }
        let input = format!("{oid}\n");
        let meta = self.run(&["cat-file", "--batch-check"], input.as_bytes(), 4096)?;
        let meta = String::from_utf8(meta)
            .map_err(|_| AnalysisError::new("GIT_FAILED", "invalid blob metadata"))?;
        let fields: Vec<_> = meta.split_whitespace().collect();
        if fields.len() != 3 || fields[0] != oid || fields[1] != "blob" {
            return Err(AnalysisError::new(
                "OBJECT_UNAVAILABLE",
                "expected local blob is unavailable",
            ));
        }
        let size = fields[2]
            .parse::<usize>()
            .map_err(|_| AnalysisError::new("GIT_FAILED", "invalid blob size"))?;
        totals.reserve_blob(size)?;
        let output = self.run(&["cat-file", "--batch"], input.as_bytes(), size + 4096)?;
        parse_blob(&output, meta.strip_suffix('\n').unwrap_or(&meta), size)
    }
}

fn nonblocking(fd: std::os::fd::RawFd) -> Result<(), AnalysisError> {
    // The descriptors belong to live Child pipes; preserve their existing flags.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        Err(AnalysisError::new(
            "GIT_IO_FAILED",
            "Git pipes could not be configured",
        ))
    } else {
        Ok(())
    }
}

fn terminate_group(group: i32) {
    // Each command starts a new process group, so inherited pipe holders are stopped too.
    unsafe {
        libc::kill(-group, libc::SIGKILL);
    }
}

fn revision_error(error: AnalysisError, message: &'static str) -> AnalysisError {
    if error.code == "GIT_FAILED" {
        AnalysisError::input(message)
    } else {
        error
    }
}

fn parse_blob(output: &[u8], header: &str, size: usize) -> Result<Vec<u8>, AnalysisError> {
    let error = || AnalysisError::new("GIT_FAILED", "invalid blob framing");
    let end = output.iter().position(|b| *b == b'\n').ok_or_else(error)?;
    if output[..end] != *header.as_bytes()
        || output.len() != end + size + 2
        || output.last() != Some(&b'\n')
    {
        return Err(error());
    }
    Ok(output[end + 1..end + 1 + size].to_vec())
}

pub fn parse_raw(raw: &[u8]) -> Result<Vec<FileChange>, AnalysisError> {
    let error = || AnalysisError::new("GIT_FAILED", "malformed Git raw diff");
    if raw.is_empty() {
        return Ok(vec![]);
    }
    let body = raw.strip_suffix(b"\0").ok_or_else(error)?;
    let mut fields = body.split(|b| *b == 0);
    let mut files = vec![];
    while let Some(header) = fields.next() {
        if header.is_empty() {
            return Err(error());
        }
        let header = std::str::from_utf8(header).map_err(|_| error())?;
        let parts: Vec<_> = header
            .strip_prefix(':')
            .ok_or_else(error)?
            .split(' ')
            .collect();
        if parts.len() != 5
            || !valid_oid(parts[2])
            || !valid_oid(parts[3])
            || parts[2].len() != parts[3].len()
            || !parts[..2]
                .iter()
                .all(|mode| ["000000", "100644", "100755", "120000", "160000"].contains(mode))
        {
            return Err(error());
        }
        let path = fields.next().ok_or_else(error)?.to_vec();
        if !valid_path(&path) {
            return Err(error());
        }
        let (old_path, new_path) = match parts[4] {
            "A" => (None, Some(path)),
            "D" => (Some(path), None),
            "R100" => {
                let new_path = fields.next().ok_or_else(error)?.to_vec();
                if !valid_path(&new_path) || new_path == path || parts[2] != parts[3] {
                    return Err(error());
                }
                (Some(path), Some(new_path))
            }
            "M" | "T" => (Some(path.clone()), Some(path)),
            _ => return Err(error()),
        };
        for (mode, oid, present) in [
            (parts[0], parts[2], old_path.is_some()),
            (parts[1], parts[3], new_path.is_some()),
        ] {
            let zero = oid.bytes().all(|b| b == b'0');
            if present && (mode == "000000" || zero) || !present && (mode != "000000" || !zero) {
                return Err(error());
            }
        }
        files.push(FileChange {
            old_path,
            new_path,
            old_mode: parts[0].into(),
            new_mode: parts[1].into(),
            old_oid: parts[2].into(),
            new_oid: parts[3].into(),
            status: parts[4].into(),
            old: vec![],
            new: vec![],
            lines: vec![],
        });
        if files.len() > MAX_FILES {
            return Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "diff exceeds 1000 files",
            ));
        }
    }
    Ok(files)
}

fn valid_path(path: &[u8]) -> bool {
    !path.is_empty()
        && !path.starts_with(b"/")
        && path
            .split(|b| *b == b'/')
            .all(|part| !part.is_empty() && part != b"." && part != b"..")
}

pub fn diff_lines(old: &[u8], new: &[u8]) -> Result<Vec<ChangedLine>, AnalysisError> {
    diff_lines_with_budget(old, new, &Budget::default(), MAX_LINES)
}

fn diff_lines_with_budget(
    old: &[u8],
    new: &[u8],
    budget: &Budget,
    max_lines: usize,
) -> Result<Vec<ChangedLine>, AnalysisError> {
    budget.check()?;
    let old = std::str::from_utf8(old)
        .map_err(|_| AnalysisError::new("UNSUPPORTED_CHANGE", "source is not UTF-8"))?;
    let new = std::str::from_utf8(new)
        .map_err(|_| AnalysisError::new("UNSUPPORTED_CHANGE", "source is not UTF-8"))?;
    let diff = similar::TextDiff::configure()
        .algorithm(similar::Algorithm::Myers)
        .deadline(budget.deadline())
        .diff_lines(old, new);
    // similar may return an approximation at its deadline: never accept that as a delta.
    budget.check()?;
    let mut result = vec![];
    for change in diff.iter_all_changes() {
        budget.check()?;
        let (side, index) = match change.tag() {
            similar::ChangeTag::Delete => (Side::Base, change.old_index()),
            similar::ChangeTag::Insert => (Side::Target, change.new_index()),
            similar::ChangeTag::Equal => continue,
        };
        if result.len() >= max_lines {
            return Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "diff exceeds 100000 changed lines",
            ));
        }
        result.push(ChangedLine {
            side,
            number: index.unwrap_or(0) + 1,
            text: change.value().into(),
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_consumes_only_remaining_global_line_allowance() {
        let budget = Budget::default();
        assert_eq!(
            diff_lines_with_budget(b"old\n", b"new\n", &budget, 2)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            diff_lines_with_budget(b"old\n", b"new\n", &budget, 1)
                .unwrap_err()
                .code,
            "LIMIT_EXCEEDED"
        );
        assert!(
            diff_lines_with_budget(b"same\n", b"same\n", &budget, 0)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn expired_delta_budget_returns_error_not_approximate_changes() {
        let budget = Budget::for_duration(Duration::ZERO);
        assert_eq!(
            diff_lines_with_budget(b"old\n", b"new\n", &budget, MAX_LINES)
                .unwrap_err()
                .code,
            "LIMIT_EXCEEDED"
        );
    }

    #[test]
    fn expensive_delta_stops_at_deadline_without_returning_fallback_diff() {
        let old = (0..10_000)
            .map(|i| format!("old-{i}\n"))
            .collect::<String>();
        let new = (0..10_000)
            .map(|i| format!("new-{i}\n"))
            .collect::<String>();
        let budget = Budget::for_duration(Duration::from_millis(1));
        let start = std::time::Instant::now();
        assert_eq!(
            diff_lines_with_budget(old.as_bytes(), new.as_bytes(), &budget, MAX_LINES)
                .unwrap_err()
                .code,
            "LIMIT_EXCEEDED"
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    fn dummy(script: &str) -> (tempfile::TempDir, Git) {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let executable = dir.path().join("git-dummy");
        std::fs::write(&executable, format!("#!/bin/sh\n{script}\n")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let git = Git {
            executable,
            root: dir.path().to_owned(),
            budget: Budget::for_duration(Duration::from_secs(1)),
            supervised: false,
        };
        (dir, git)
    }

    #[test]
    fn timeout_cancels_blocked_stdin_and_reaps_process() {
        let (dir, git) = dummy("echo $$ > pid; exec /bin/sleep 30");
        let start = std::time::Instant::now();
        let error = git.run(&[], &vec![b'x'; MAX_BLOB], 1024).unwrap_err();
        assert_eq!(error.code, "LIMIT_EXCEEDED");
        assert_eq!(error.exit, 1);
        assert!(start.elapsed() < Duration::from_secs(2));
        let pid = std::fs::read_to_string(dir.path().join("pid"))
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap();
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "child still exists or is not reaped"
        );
    }

    #[test]
    fn inherited_output_pipes_cannot_block_cleanup_after_parent_exit() {
        let (_dir, git) = dummy("/bin/sleep 30 &\nexit 0");
        let start = std::time::Instant::now();
        assert_eq!(git.run(&[], b"", 1024).unwrap_err().code, "LIMIT_EXCEEDED");
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn stdout_and_stderr_overflow_abort_instead_of_partial_success() {
        for script in ["exec /usr/bin/yes output", "exec /usr/bin/yes output >&2"] {
            let (_dir, git) = dummy(script);
            let error = git.run(&[], b"", 32).unwrap_err();
            assert_eq!(error.code, "LIMIT_EXCEEDED");
            assert_eq!(error.exit, 1);
            assert!(!error.to_string().contains("output\noutput"));
        }
    }

    #[test]
    fn broken_input_pipe_and_nonzero_exit_are_processing_errors() {
        for script in [
            "exec 0<&-; exec /bin/sleep 30",
            "echo sensitive-value >&2; exit 7",
        ] {
            let (_dir, git) = dummy(script);
            let error = git.run(&[], &vec![b'x'; MAX_BLOB], 1024).unwrap_err();
            assert_eq!(error.exit, 1);
            assert_ne!(error.code, "INVALID_INPUT");
            assert!(!error.to_string().contains("sensitive-value"));
        }
    }

    #[test]
    fn small_output_from_fast_success_is_fully_drained() {
        let (_dir, git) = dummy("printf complete-output");
        assert_eq!(git.run(&[], b"", 1024).unwrap(), b"complete-output");
    }

    #[test]
    fn batch_blob_framing_requires_exact_header_size_and_terminator() {
        let header = format!("{} blob 3", "a".repeat(40));
        let valid = format!("{header}\na\nb\n");
        assert_eq!(parse_blob(valid.as_bytes(), &header, 3).unwrap(), b"a\nb");
        for bytes in [
            format!("{header}\na\nb"),
            format!("{header}\na\nb\nextra"),
            "wrong\na\nb\n".to_owned(),
            format!("{header}\na\n"),
        ] {
            assert!(parse_blob(bytes.as_bytes(), &header, 3).is_err());
        }
        let empty_header = format!("{} blob 0", "b".repeat(64));
        assert_eq!(
            parse_blob(format!("{empty_header}\n\n").as_bytes(), &empty_header, 0).unwrap(),
            b""
        );
    }

    #[test]
    fn revision_lookup_does_not_relabel_limits_or_unavailable_git() {
        for code in ["LIMIT_EXCEEDED", "GIT_UNAVAILABLE"] {
            let result = revision_error(
                AnalysisError::new(code, "operation failed"),
                "invalid revision",
            );
            assert_eq!(result.code, code);
            assert_eq!(result.exit, 1);
        }
        assert_eq!(
            revision_error(
                AnalysisError::new("GIT_FAILED", "lookup failed"),
                "invalid revision"
            )
            .exit,
            2
        );
    }
}
