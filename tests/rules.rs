use risk_analyzer::{git::diff_lines, model::FileChange};

pub fn change(path: &str, old: &str, new: &str) -> FileChange {
    FileChange {
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
    }
}

#[test]
fn signal_reference_pairs() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/rules/signals.json")).unwrap();
    for case in cases {
        let file = change(
            case["path"].as_str().unwrap(),
            case["old"].as_str().unwrap(),
            case["new"].as_str().unwrap(),
        );
        let result = risk_analyzer::rules::evaluate(&[file]);
        let id = case["signal"].as_str().unwrap();
        if case["positive"].as_bool().unwrap() {
            assert!(
                result
                    .as_ref()
                    .is_ok_and(|r| r.iter().any(|f| f.rule == id)),
                "{case}: {result:?}"
            );
        } else {
            assert!(
                !result
                    .as_ref()
                    .is_ok_and(|r| r.iter().any(|f| f.rule == id)),
                "false positive {case}"
            );
        }
    }
}

#[test]
fn unknown_change_beside_high_is_not_success() {
    let auth = change(
        "src/auth/Guard.java",
        "class Guard { boolean m() { return true; } }",
        "class Guard { boolean m() { return false; } }",
    );
    let unknown = change(
        "src/Other.java",
        "class Other { void m() {} }",
        "class Other { void m() { customSideEffect(); } }",
    );
    assert!(risk_analyzer::rules::evaluate(&[auth, unknown]).is_err());
}

#[test]
fn exact_rename_positive_and_negative() {
    let mut file = change("src/B.java", "class A {}", "class A {}");
    file.old_path = Some(b"src/A.java".to_vec());
    file.status = "R100".into();
    file.new_oid = file.old_oid.clone();
    assert!(
        risk_analyzer::rules::evaluate(&[file.clone()])
            .unwrap()
            .iter()
            .any(|f| f.rule == "NEUTRAL_RENAME")
    );
    file.new_path = Some(b"src/auth/B.java".to_vec());
    let findings = risk_analyzer::rules::evaluate(&[file]).unwrap();
    assert!(findings.iter().any(|f| f.rule == "AUTH_CHANGE"));
    assert!(!findings.iter().any(|f| f.rule == "NEUTRAL_RENAME"));
}

#[test]
fn source_literals_and_comments_do_not_trigger_auth_operations() {
    let file = change(
        "src/A.java",
        "class A { String m() { return \"a\"; } }",
        "class A { String m() { return \"authorize()\"; } }",
    );
    let findings = risk_analyzer::rules::evaluate(&[file]).unwrap();
    assert!(!findings.iter().any(|f| f.rule == "AUTH_CHANGE"));
    let file = change(
        "src/auth/A.java",
        "class A {} // before",
        "class A {} // authorize",
    );
    let findings = risk_analyzer::rules::evaluate(&[file]).unwrap();
    assert!(findings.iter().all(|f| f.rule == "DOC_CHANGE"));
}

#[test]
fn duplicate_contract_keys_must_not_silently_override() {
    let file = change(
        "api/openapi.json",
        "{\"paths\":{}}",
        "{\"paths\":{\"/a\":{}},\"paths\":{}}",
    );
    assert!(risk_analyzer::rules::evaluate(&[file]).is_err());
}

#[test]
fn same_line_return_must_not_hide_unknown_side_effect() {
    let file = change(
        "src/A.java",
        "class A { int m() { return 1; } }",
        "class A { int m() { customSideEffect(); return 2; } }",
    );
    assert!(risk_analyzer::rules::evaluate(&[file]).is_err());
}

#[test]
fn manifest_metadata_changes_are_not_dependency_updates() {
    for (path, old, new) in [
        (
            "pyproject.toml",
            "[project]\nname='before'\ndependencies=['lib']\n",
            "[project]\nname='after'\ndependencies=['lib']\n",
        ),
        (
            "pom.xml",
            "<project>\n<name>before</name>\n<dependencies>\n<dependency>x</dependency>\n</dependencies>\n</project>",
            "<project>\n<name>after</name>\n<dependencies>\n<dependency>x</dependency>\n</dependencies>\n</project>",
        ),
    ] {
        assert!(risk_analyzer::rules::evaluate(&[change(path, old, new)]).is_err());
    }
    let old = "[project]\nname='app'\ndependencies=['lib==1']\n";
    let new = "[project]\nname='app'\ndependencies=['lib==2']\n";
    assert!(
        risk_analyzer::rules::evaluate(&[change("pyproject.toml", old, new)])
            .unwrap()
            .iter()
            .any(|f| f.rule == "DEPENDENCY_UPDATE")
    );
}

#[test]
fn multiple_style_rules_are_checked_independently() {
    let old = ".a { color: red; }\n.b { margin: 1px; }";
    let new = ".a { color: blue; }\n.b { margin: 2px; }";
    assert!(
        risk_analyzer::rules::evaluate(&[change("style.css", old, new)])
            .unwrap()
            .iter()
            .any(|f| f.rule == "VISUAL_STYLE")
    );
    assert!(
        risk_analyzer::rules::evaluate(&[change(
            "style.css",
            old,
            ".a { color: blue; }\n.b { display: none; }"
        )])
        .is_err()
    );
}

#[test]
fn sql_comments_and_quoted_keywords_are_not_destructive_operations() {
    for text in [
        "-- DROP TABLE users;\nSELECT id FROM users;",
        "SELECT 'DROP TABLE users' AS example;",
        "/* DROP TABLE users */\nSELECT id FROM users;",
    ] {
        let findings =
            risk_analyzer::rules::evaluate(&[change("migrations/V2.sql", "", text)]).unwrap();
        assert!(
            !findings.iter().any(|f| f.rule == "DESTRUCTIVE_MIGRATION"),
            "{text}: {findings:?}"
        );
    }
}

#[test]
fn python_indentation_and_js_line_terminators_are_not_assumed_neutral() {
    let python = change(
        "src/a.py",
        "if ok:\n    return_value = 1\n",
        "if ok:\n    pass\nreturn_value = 1\n",
    );
    assert!(
        !risk_analyzer::rules::evaluate(&[python])
            .is_ok_and(|f| f.iter().all(|f| f.rule == "DOC_CHANGE"))
    );
    let js = change(
        "src/a.js",
        "function a() { return 1; }",
        "function a() { return\n1; }",
    );
    assert!(
        !risk_analyzer::rules::evaluate(&[js])
            .is_ok_and(|f| f.iter().all(|f| f.rule == "DOC_CHANGE"))
    );
}

#[test]
fn multiline_query_changes_keep_operation_context() {
    let old = "SELECT id\nFROM users\nWHERE active = 1;\n";
    let new = "SELECT id\nFROM users\nWHERE active = 0;\n";
    let findings = risk_analyzer::rules::evaluate(&[change("query.sql", old, new)]).unwrap();
    assert!(findings.iter().any(|f| f.rule == "QUERY_PERFORMANCE"));
    assert!(
        risk_analyzer::rules::evaluate(&[change(
            "query.sql",
            old,
            "SELECT id\nFROM users;\nCUSTOM_UNKNOWN;\n"
        )])
        .is_err()
    );
    assert!(
        risk_analyzer::rules::evaluate(&[change(
            "query.sql",
            "",
            "SELECT id FROM users; CUSTOM_UNKNOWN;"
        )])
        .is_err()
    );
}

#[test]
fn adding_required_api_fields_is_not_a_compatible_extension() {
    let old = r#"{"components":{"schemas":{"User":{"properties":{"id":{"type":"string"}},"required":["id"]}}}}"#;
    let new = r#"{"components":{"schemas":{"User":{"properties":{"id":{"type":"string"},"name":{"type":"string"}},"required":["id","name"]}}}}"#;
    let findings = risk_analyzer::rules::evaluate(&[change("openapi.json", old, new)]).unwrap();
    assert!(findings.iter().any(|f| f.rule == "CONTRACT_BREAK"));
}

#[test]
fn malformed_critical_infrastructure_is_not_successfully_assessed() {
    assert!(
        risk_analyzer::rules::evaluate(&[change("production/service.json", "{}", "{invalid")])
            .is_err()
    );
}
