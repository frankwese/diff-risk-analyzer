use std::process::Command;

#[test]
fn auth_context_cli_regressions_are_stable_for_rule_version_110() {
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
    let old = "class Access { boolean secure() { authorize(user); return false; } int other() { return 1; } }";
    std::fs::write(root.join("Access.java"), old).unwrap();
    git(&["add", "."]);
    git(&["commit", "-qm", "base"]);
    for (new, context) in [
        (old.replace("return false", "return true"), true),
        (format!("{old} // comment"), false),
        (old.replace("return 1", "return 2"), false),
    ] {
        std::fs::write(root.join("Access.java"), new).unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "target"]);
        let run = || {
            Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
                .current_dir(root)
                .arg("HEAD~1..HEAD")
                .output()
                .unwrap()
        };
        let output = run();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("Rules: 1.1.0"));
        assert_eq!(text.contains("AUTH_CONTEXT_CHANGE"), context);
        if context {
            assert!(text.starts_with("Risk: HIGH"));
            assert!(!text.contains("[AUTH_CHANGE]"));
        }
        for _ in 0..10 {
            let repeated = run();
            assert!(repeated.status.success());
            assert_eq!(repeated.stdout, output.stdout);
        }
        git(&["reset", "--hard", "HEAD~1"]);
    }
}

#[test]
fn core_findings_preserve_order_sides_locations_and_deduplication() {
    use risk_analyzer::{git::diff_lines, model::FileChange};
    let make = |path: &str, old: &str, new: &str| FileChange {
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
    let auth = make(
        "src/auth/A.java",
        "class A { boolean m() { return true; } }",
        "class A { boolean m() { return false; } }",
    );
    let docs = make("docs/a\n.md", "old", "new");
    let findings = risk_analyzer::rules::evaluate(&[docs.clone(), auth.clone(), auth]).unwrap();
    assert_eq!(
        findings.iter().filter(|f| f.rule == "DOC_CHANGE").count(),
        1
    );
    assert_eq!(
        findings.iter().filter(|f| f.rule == "AUTH_CHANGE").count(),
        2
    );
    let output = risk_analyzer::cli::render(&findings);
    assert!(output.starts_with("Risk: HIGH\nReasons:\n- [AUTH_CHANGE]"));
    assert!(output.contains("(base:1)"));
    assert!(output.contains("(target:1)"));
    assert!(output.contains("docs/a\\n.md (metadata)"));
    assert!(!output.contains('\r'));
    assert!(!output.contains('\u{1b}'));
    for _ in 0..10 {
        assert_eq!(risk_analyzer::cli::render(&findings), output);
    }
    let empty = risk_analyzer::cli::render(&[]);
    assert!(empty.starts_with("Risk: LOW\nReasons:\n- No changes"));
}

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
fn unavailable_or_unsuitable_git_is_exit_one_without_analysis_output() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
            .env("PATH", dir.path())
            .current_dir(dir.path())
            .arg("HEAD~1..HEAD")
            .output()
            .unwrap()
    };
    let assert_unavailable = |output: std::process::Output| {
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("GIT_UNAVAILABLE"));
    };
    assert_unavailable(run());
    let dummy = dir.path().join("git");
    for version in ["git version 2.48.0", "not-a-git-version"] {
        std::fs::write(&dummy, format!("#!/bin/sh\nprintf '%s\\n' '{version}'\n")).unwrap();
        std::fs::set_permissions(&dummy, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_unavailable(run());
    }
    std::fs::set_permissions(&dummy, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_unavailable(run());
}

#[test]
fn outside_repository_is_exit_two_but_help_and_version_need_no_git() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
        .current_dir(dir.path())
        .arg("HEAD~1..HEAD")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("INVALID_INPUT"));
    for arg in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
            .env("PATH", dir.path())
            .current_dir(dir.path())
            .arg(arg)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
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
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("/usr/bin/time");
        command.args(["-l", env!("CARGO_BIN_EXE_risk-analyzer")]);
        command
    } else {
        Command::new(env!("CARGO_BIN_EXE_risk-analyzer"))
    };
    let output = command
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
    if cfg!(target_os = "macos") {
        println!(
            "CLI resource measurement:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
