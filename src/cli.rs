use std::ffi::OsString;

pub fn run(args: impl Iterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.collect();
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
            let budget = crate::model::Budget::default();
            let worker_budget = budget.clone();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let result = std::env::current_dir()
                    .map_err(|_| {
                        crate::model::AnalysisError::input("working directory unavailable")
                    })
                    .and_then(|cwd| crate::git::Git::open(&cwd, worker_budget.clone()))
                    .and_then(|git| git.compare(&base, &target))
                    .and_then(|comparison| {
                        crate::rules::evaluate(&comparison.files).map(|findings| {
                            render_with_tests(
                                &findings,
                                &crate::association::evaluate(&comparison.files),
                            )
                        })
                    })
                    .and_then(|output| {
                        worker_budget.check()?;
                        Ok(output)
                    });
                let _ = sender.send(result);
            });
            let result = receiver
                .recv_timeout(budget.remaining())
                .unwrap_or_else(|_| {
                    Err(crate::model::AnalysisError::new(
                        "LIMIT_EXCEEDED",
                        "analysis exceeded its total time limit",
                    ))
                });
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
    }
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
