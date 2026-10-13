use crate::model::{
    AnalysisError, Budget, ChangedLine, Comparison, FileChange, MAX_BLOB, MAX_FILES, MAX_LINES,
    MAX_TOTAL, Side,
};
use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
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
}

impl Git {
    pub fn open(cwd: &Path, budget: Budget) -> Result<Self, AnalysisError> {
        let executable = std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|p| p.join("git"))
                    .find(|p| p.is_file())
            })
            .ok_or_else(|| {
                AnalysisError::new("GIT_UNAVAILABLE", "Git >=2.49 is required on PATH")
            })?;
        let mut git = Self {
            executable,
            root: cwd.to_path_buf(),
            budget,
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
            .map_err(|_| AnalysisError::input("run inside a local Git repository"))?;
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
        let mut child = cmd.spawn().map_err(|_| {
            AnalysisError::new("GIT_UNAVAILABLE", "Git process could not be started")
        })?;
        let overflow = Arc::new(AtomicBool::new(false));
        let read_bounded =
            |mut stream: Box<dyn Read + Send>, limit: usize, overflow: Arc<AtomicBool>| {
                thread::spawn(move || {
                    let mut output = Vec::new();
                    let mut buf = [0; 8192];
                    loop {
                        let n = stream.read(&mut buf)?;
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
        let stdout = read_bounded(
            Box::new(child.stdout.take().expect("stdout pipe")),
            max_output,
            overflow.clone(),
        );
        let stderr = read_bounded(
            Box::new(child.stderr.take().expect("stderr pipe")),
            MAX_BLOB,
            overflow.clone(),
        );
        let mut stdin = child.stdin.take().expect("stdin pipe");
        let input = input.to_vec();
        let writer = thread::spawn(move || stdin.write_all(&input));
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
            match child.try_wait() {
                Ok(Some(status)) => {
                    break if status.success() {
                        Ok(())
                    } else {
                        Err(AnalysisError::new(
                            "GIT_FAILED",
                            "Git operation failed; verify local objects and revisions",
                        ))
                    };
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
            let _ = child.kill();
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
            .map_err(|_| {
                AnalysisError::input("revision does not resolve to a locally available commit")
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
        let mut total = 0;
        let mut line_total = 0;
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
            file.old = self.blob(&file.old_oid, &mut total)?;
            file.new = self.blob(&file.new_oid, &mut total)?;
            for bytes in [&file.old, &file.new] {
                if bytes.contains(&0)
                    || std::str::from_utf8(bytes).is_err()
                    || bytes.starts_with(b"version https://git-lfs.github.com/spec/v1")
                {
                    return Err(AnalysisError::unsupported(file.path()));
                }
            }
            file.lines = diff_lines(&file.old, &file.new)?;
            line_total += file.lines.len();
            if line_total > MAX_LINES {
                return Err(AnalysisError::new(
                    "LIMIT_EXCEEDED",
                    "diff exceeds 100000 changed lines",
                ));
            }
        }
        Ok(Comparison {
            root: self.root.clone(),
            base,
            target,
            files,
        })
    }

    fn blob(&self, oid: &str, total: &mut usize) -> Result<Vec<u8>, AnalysisError> {
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
        if size > MAX_BLOB || *total + size > MAX_TOTAL {
            return Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "blob exceeds size or total input limit",
            ));
        }
        *total += size;
        let output = self.run(&["cat-file", "--batch"], input.as_bytes(), size + 4096)?;
        let end = output
            .iter()
            .position(|b| *b == b'\n')
            .ok_or_else(|| AnalysisError::new("GIT_FAILED", "invalid blob header"))?;
        if output[..end] != meta.trim_end().as_bytes()[..]
            || output.len() != end + size + 2
            || output.last() != Some(&b'\n')
        {
            return Err(AnalysisError::new("GIT_FAILED", "invalid blob framing"));
        }
        Ok(output[end + 1..end + 1 + size].to_vec())
    }
}

fn valid_oid(oid: &str) -> bool {
    [40, 64].contains(&oid.len()) && oid.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn parse_raw(raw: &[u8]) -> Result<Vec<FileChange>, AnalysisError> {
    let error = || AnalysisError::new("GIT_FAILED", "malformed Git raw diff");
    let mut fields = raw.split(|b| *b == 0);
    let mut files = vec![];
    while let Some(header) = fields.next() {
        if header.is_empty() {
            break;
        }
        let header = std::str::from_utf8(header).map_err(|_| error())?;
        let parts: Vec<_> = header
            .strip_prefix(':')
            .ok_or_else(error)?
            .split_whitespace()
            .collect();
        if parts.len() != 5 || !valid_oid(parts[2]) || !valid_oid(parts[3]) {
            return Err(error());
        }
        let path = fields.next().ok_or_else(error)?.to_vec();
        if path.is_empty() || path.starts_with(b"/") {
            return Err(error());
        }
        let (old_path, new_path) = match parts[4] {
            "A" => (None, Some(path)),
            "D" => (Some(path), None),
            "R100" => (Some(path), Some(fields.next().ok_or_else(error)?.to_vec())),
            "M" | "T" => (Some(path.clone()), Some(path)),
            _ => return Err(error()),
        };
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

pub fn diff_lines(old: &[u8], new: &[u8]) -> Result<Vec<ChangedLine>, AnalysisError> {
    let old = std::str::from_utf8(old)
        .map_err(|_| AnalysisError::new("UNSUPPORTED_CHANGE", "source is not UTF-8"))?;
    let new = std::str::from_utf8(new)
        .map_err(|_| AnalysisError::new("UNSUPPORTED_CHANGE", "source is not UTF-8"))?;
    let diff = similar::TextDiff::configure()
        .algorithm(similar::Algorithm::Myers)
        .diff_lines(old, new);
    let mut result = vec![];
    for change in diff.iter_all_changes() {
        let (side, index) = match change.tag() {
            similar::ChangeTag::Delete => (Side::Base, change.old_index()),
            similar::ChangeTag::Insert => (Side::Target, change.new_index()),
            similar::ChangeTag::Equal => continue,
        };
        result.push(ChangedLine {
            side,
            number: index.unwrap_or(0) + 1,
            text: change.value().into(),
        });
        if result.len() > MAX_LINES {
            return Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "diff exceeds 100000 changed lines",
            ));
        }
    }
    Ok(result)
}
