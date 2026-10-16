use std::ffi::OsString;

pub fn run(args: impl Iterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.collect();
    if args.first().is_some_and(|a| a == "--internal-analysis") {
        let result = parse_range(&args[1..]).and_then(|(base, target)| analyze(&base, &target));
        return emit(result);
    }
    if args == [OsString::from("--help")] {
        println!("Usage: risk-analyzer BASE..TARGET\nLocal, read-only Git diff analysis.");
        return 0;
    }
    if args == [OsString::from("--version")] {
        println!(
            "risk-analyzer {} (rules {})",
            env!("CARGO_PKG_VERSION"),
            crate::model::RULES_VERSION
        );
        return 0;
    }
    let result = parse_range(&args);
    match result {
        Err(error) => {
            eprintln!("{error}");
            error.exit
        }
        Ok((base, target)) => {
            let result = std::env::current_exe()
                .map_err(|_| {
                    crate::model::AnalysisError::new(
                        "PROCESS_FAILED",
                        "analysis executable unavailable",
                    )
                })
                .and_then(|exe| {
                    let mut command = std::process::Command::new(exe);
                    command.args(["--internal-analysis", &format!("{base}..{target}")]);
                    supervise(command, crate::model::Budget::default())
                });
            match result {
                Ok((output, error, exit)) => {
                    print!("{output}");
                    eprint!("{error}");
                    exit
                }
                Err(error) => emit(Err(error)),
            }
        }
    }
}

fn emit(result: Result<String, crate::model::AnalysisError>) -> i32 {
    match result {
        Ok(output) => {
            print!("{output}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            error.exit
        }
    }
}

fn analyze(base: &str, target: &str) -> Result<String, crate::model::AnalysisError> {
    let budget = crate::model::Budget::default();
    let cwd = std::env::current_dir()
        .map_err(|_| crate::model::AnalysisError::input("working directory unavailable"))?;
    let comparison =
        crate::git::Git::open_supervised(&cwd, budget.clone())?.compare(base, target)?;
    let findings = crate::rules::evaluate(&comparison.files)?;
    let tests = crate::association::evaluate(&comparison.files);
    budget.check()?;
    Ok(render_with_tests(&findings, &tests))
}

fn supervise(
    mut command: std::process::Command,
    budget: crate::model::Budget,
) -> Result<(String, String, i32), crate::model::AnalysisError> {
    use std::{
        io::Read,
        os::unix::process::CommandExt,
        process::Stdio,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };
    let error = || crate::model::AnalysisError::new("PROCESS_FAILED", "analysis process failed");
    let mut child = command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| error())?;
    let group = child.id() as i32;
    let overflow = Arc::new(AtomicBool::new(false));
    fn read_output(
        mut pipe: impl Read + Send + 'static,
        overflow: Arc<AtomicBool>,
    ) -> thread::JoinHandle<std::io::Result<Vec<u8>>> {
        thread::spawn(move || {
            let mut bytes = vec![];
            let mut buffer = [0; 8192];
            loop {
                let n = pipe.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                if bytes.len() + n > crate::model::MAX_TOTAL {
                    overflow.store(true, Ordering::Relaxed);
                    return Err(std::io::Error::other("output exceeded limit"));
                }
                bytes.extend_from_slice(&buffer[..n]);
            }
            Ok(bytes)
        })
    }
    let stdout = read_output(child.stdout.take().expect("stdout pipe"), overflow.clone());
    let stderr = read_output(child.stderr.take().expect("stderr pipe"), overflow.clone());
    let status = loop {
        if overflow.load(Ordering::Relaxed) {
            break Err(crate::model::AnalysisError::new(
                "LIMIT_EXCEEDED",
                "analysis output exceeded limit",
            ));
        }
        if let Err(e) = budget.check() {
            break Err(e);
        }
        match child.try_wait() {
            Ok(Some(s)) => break Ok(s),
            Ok(None) => thread::sleep(Duration::from_millis(2)),
            Err(_) => break Err(error()),
        }
    };
    // Worker and its Git children share this dedicated group. Stop pipe holders as well.
    unsafe {
        libc::kill(-group, libc::SIGKILL);
    }
    let _ = child.wait();
    let out = stdout.join().map_err(|_| error())?;
    let err = stderr.join().map_err(|_| error())?;
    let status = status?;
    let output = String::from_utf8(out.map_err(|_| error())?).map_err(|_| error())?;
    let errors = String::from_utf8(err.map_err(|_| error())?).map_err(|_| error())?;
    let exit = status
        .code()
        .filter(|s| [0, 1, 2].contains(s))
        .ok_or_else(error)?;
    if exit != 0 && !output.is_empty() {
        return Err(error());
    }
    Ok((output, errors, exit))
}

pub fn render(findings: &[crate::model::Finding]) -> String {
    render_with_tests(findings, &[])
}

pub fn render_with_tests(
    findings: &[crate::model::Finding],
    tests: &[crate::association::Association],
) -> String {
    use crate::model::{Level, RULES_VERSION, escape};
    let risk = findings.iter().map(|f| f.level).max().unwrap_or(Level::Low);
    let mut output = format!("Risk: {risk}\nReasons:\n");
    if findings.is_empty() {
        output.push_str("- No changes between the requested revisions.\n");
    }
    for f in findings {
        output.push_str(&format!(
            "- [{}] {} — {} ({}{})\n",
            f.rule,
            f.reason,
            escape(&f.path),
            f.side,
            f.line.map(|l| format!(":{l}")).unwrap_or_default()
        ));
    }
    if tests.is_empty() {
        output.push_str("Tests: Not applicable.\n");
    } else {
        output.push_str("Tests:\n");
        for test in tests {
            output.push_str(&format!(
                "- {} {}::{} — {}",
                test.status,
                escape(&test.path),
                test.symbol,
                if test.evidence.is_empty() {
                    test.reason
                } else {
                    "usage detected in changed test"
                }
            ));
            for evidence in &test.evidence {
                output.push_str(&format!("; {evidence}"));
            }
            output.push('\n');
        }
    }
    output.push_str(&format!("Rules: {RULES_VERSION}\nNote: Heuristic review aid; test usage does not prove coverage or correctness.\n"));
    output
}

pub fn parse_range(args: &[OsString]) -> Result<(String, String), crate::model::AnalysisError> {
    let error = || {
        crate::model::AnalysisError::input("expected BASE..TARGET (two non-empty local revisions)")
    };
    if args.len() != 1 {
        return Err(error());
    }
    let text = args[0].to_str().ok_or_else(error)?;
    if text.contains("...") || text.matches("..").count() != 1 {
        return Err(error());
    }
    let (base, target) = text.split_once("..").ok_or_else(error)?;
    for endpoint in [base, target] {
        crate::model::validate_revision_endpoint(endpoint)?;
    }
    Ok((base.into(), target.into()))
}

#[cfg(test)]
mod supervisor_tests {
    #[test]
    fn total_timeout_kills_worker_and_pipe_holding_children() {
        let dir = tempfile::tempdir().unwrap();
        let pid = dir.path().join("pid");
        let mut command = std::process::Command::new("/bin/sh");
        command.args([
            "-c",
            &format!("echo $$ > '{}'; /bin/sleep 30 & wait", pid.display()),
        ]);
        let start = std::time::Instant::now();
        let result = super::supervise(
            command,
            crate::model::Budget::for_duration(std::time::Duration::from_secs(1)),
        );
        assert_eq!(result.unwrap_err().exit, 1);
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
        let pid = std::fs::read_to_string(pid)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    }
}
