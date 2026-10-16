use std::process::Command;

#[test]
fn git_process_failures_reach_cli_without_partial_output_or_secrets() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let git = dir.path().join("git");
    for failure in [
        "printf sensitive-value >&2; exit 7",
        "exec /usr/bin/yes sensitive-value",
    ] {
        let script = format!(
            "#!/bin/sh\ncase \"$*\" in\n*--version*) printf 'git version 2.54.0\\n';;\n*--show-toplevel*) printf '%s\\n' \"$PWD\";;\n*--verify*) printf '{}\\n';;\n*) {failure};;\nesac\n",
            "a".repeat(40)
        );
        std::fs::write(&git, script).unwrap();
        std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
            .env("PATH", dir.path())
            .current_dir(dir.path())
            .arg("HEAD~1..HEAD")
            .output()
            .unwrap();
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("sensitive-value"));
        if failure.contains("yes") {
            assert_eq!(output.status.code(), Some(1));
            assert!(stderr.contains("LIMIT_EXCEEDED"));
        } else {
            assert_eq!(output.status.code(), Some(1));
            assert!(stderr.contains("GIT_FAILED"));
        }
    }
}

#[test]
fn help_version_and_invalid_ranges_have_contract_exit_codes() {
    for arg in ["--help", "--version"] {
        let out = Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
            .arg(arg)
            .output()
            .unwrap();
        assert!(out.status.success());
        assert!(out.stderr.is_empty());
    }
    for args in [
        vec![],
        vec!["HEAD...main"],
        vec!["..HEAD"],
        vec!["-x..HEAD"],
        vec!["HEAD..HEAD", "extra"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn deterministic_empty_and_three_risk_levels() {
    use risk_analyzer::{git::diff_lines, model::FileChange};
    let cases = [
        ("README.md", "old", "new", "LOW"),
        (
            "package.json",
            "{\"dependencies\":{\"a\":\"1\"}}",
            "{\"dependencies\":{\"a\":\"2\"}}",
            "MEDIUM",
        ),
        ("migrations/V2.sql", "", "DROP TABLE users;", "HIGH"),
    ];
    for (path, old, new, level) in cases {
        let file = FileChange {
            old_path: Some(path.as_bytes().to_vec()),
            new_path: Some(path.as_bytes().to_vec()),
            old_mode: "100644".into(),
            new_mode: "100644".into(),
            old_oid: "a".repeat(40),
            new_oid: "b".repeat(40),
            status: "M".into(),
            old: old.as_bytes().to_vec(),
            new: new.as_bytes().to_vec(),
            lines: diff_lines(old.as_bytes(), new.as_bytes()).unwrap(),
        };
        let findings = risk_analyzer::rules::evaluate(&[file]).unwrap();
        let expected = risk_analyzer::cli::render(&findings);
        assert!(expected.starts_with(&format!("Risk: {level}\nReasons:\n")));
        for _ in 0..10 {
            assert_eq!(risk_analyzer::cli::render(&findings), expected);
        }
    }
    assert!(risk_analyzer::cli::render(&[]).contains("No changes"));
}

#[test]
fn end_to_end_high_with_usage_and_error_without_partial_output() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .status()
                .unwrap()
                .success()
        )
    };
    git(&["init", "-q"]);
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/guard.js"),
        "export class Guard { authorize() { return true; } }",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-qm", "base"]);
    std::fs::write(
        root.join("src/guard.js"),
        "export class Guard { authorize() { return false; } }",
    )
    .unwrap();
    std::fs::write(root.join("src/guard.test.js"),"import {Guard} from './guard.js'; test('a', () => { const g=new Guard(); expect(g.authorize()); });").unwrap();
    git(&["add", "."]);
    git(&["commit", "-qm", "target"]);
    let run = |arg: &str| {
        Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
            .current_dir(root.join("src"))
            .arg(arg)
            .output()
            .unwrap()
    };
    let expected = run("HEAD~1..HEAD");
    assert!(
        expected.status.success(),
        "{}",
        String::from_utf8_lossy(&expected.stderr)
    );
    let text = String::from_utf8_lossy(&expected.stdout);
    assert!(text.starts_with("Risk: HIGH\n"));
    assert!(text.contains("USED src/guard.js::Guard.authorize"));
    assert!(text.contains("src/guard.test.js"));
    assert!(expected.stderr.is_empty());
    for _ in 0..10 {
        assert_eq!(run("HEAD~1..HEAD").stdout, expected.stdout);
    }
    let invalid = run("missing..HEAD");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    std::fs::write(root.join("unknown.bin"), b"secret-value\0").unwrap();
    git(&["add", "."]);
    git(&["commit", "-qm", "unsupported"]);
    let invalid = run("HEAD~2..HEAD");
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&invalid.stderr).contains("secret-value"));
}

#[test]
fn representative_100_file_10000_line_diff() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .status()
                .unwrap()
                .success()
        )
    };
    git(&["init", "-q"]);
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    for i in 0..100 {
        std::fs::write(
            root.join(format!("doc{i}.md")),
            (0..50)
                .map(|n| format!("old line {n}\n"))
                .collect::<String>(),
        )
        .unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-qm", "base"]);
    for i in 0..100 {
        std::fs::write(
            root.join(format!("doc{i}.md")),
            (0..50)
                .map(|n| format!("new line {n}\n"))
                .collect::<String>(),
        )
        .unwrap();
    }
    git(&["add", "."]);
    git(&["commit", "-qm", "target"]);
    let start = std::time::Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
        .current_dir(root)
        .arg("HEAD~1..HEAD")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.starts_with(b"Risk: LOW\n"));
    println!("100 files / 10000 changed lines: {:?}", start.elapsed());
}
