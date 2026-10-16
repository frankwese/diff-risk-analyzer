use risk_analyzer::{git::diff_lines, model::FileChange};

#[test]
fn unchanged_auth_check_is_context_for_changed_logic_not_direct_change() {
    for (path, old, new) in [
        (
            "src/Access.java",
            "class Access { boolean mayAccess() { authorize(user); return false; } }",
            "class Access { boolean mayAccess() { authorize(user); return true; } }",
        ),
        (
            "src/access.js",
            "function mayAccess() { authorize(user); return false; }",
            "function mayAccess() { authorize(user); return true; }",
        ),
        (
            "src/access.ts",
            "function mayAccess(): boolean { authorize(user); return false; }",
            "function mayAccess(): boolean { authorize(user); return true; }",
        ),
        (
            "src/access.py",
            "def may_access():\n    authorize(user)\n    return False\n",
            "def may_access():\n    authorize(user)\n    return True\n",
        ),
    ] {
        let findings = risk_analyzer::rules::evaluate(&[change(path, old, new)]).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.rule == "AUTH_CONTEXT_CHANGE"
                    && f.level == risk_analyzer::model::Level::High),
            "{path}: {findings:?}"
        );
        assert!(
            !findings.iter().any(|f| f.rule == "AUTH_CHANGE"),
            "unmodified check incorrectly reported: {findings:?}"
        );
    }
}

#[test]
fn auth_name_contract_and_removed_check_preserve_auth_context() {
    for (old, new) in [
        (
            "function authorize() { return false; }",
            "function authorize() { return true; }",
        ),
        (
            "function mayAccess(user) { authorize(user); return false; }",
            "function mayAccess(user, role) { authorize(user); return false; }",
        ),
        (
            "function mayAccess() { authorize(user); return false; }",
            "function mayAccess() { return true; }",
        ),
    ] {
        let findings =
            risk_analyzer::rules::evaluate(&[change("src/access.js", old, new)]).unwrap();
        assert!(
            findings.iter().any(|f| f.rule == "AUTH_CONTEXT_CHANGE"),
            "{findings:?}"
        );
    }
}

#[test]
fn auth_context_excludes_other_functions_comments_strings_and_nested_helpers() {
    for (old, new) in [
        (
            "function secure() { authorize(user); return false; } function other() { return 1; }",
            "function secure() { authorize(user); return false; } function other() { return 2; }",
        ),
        (
            "function mayAccess() { const text='authorize(user)'; return 1; }",
            "function mayAccess() { const text='authorize(user)'; return 2; }",
        ),
        (
            "function mayAccess() { authorize(user); return 1; } // old",
            "function mayAccess() { authorize(user); return 1; } // new",
        ),
        (
            "function mayAccess() { const helper=()=>authorize(user); return 1; }",
            "function mayAccess() { const helper=()=>authorize(user); return 2; }",
        ),
    ] {
        let findings =
            risk_analyzer::rules::evaluate(&[change("src/access.js", old, new)]).unwrap();
        assert!(
            !findings.iter().any(|f| f.rule == "AUTH_CONTEXT_CHANGE"),
            "{findings:?}"
        );
    }
}

#[test]
fn independent_source_regions_do_not_inherit_a_neighbor_signal() {
    let cases = [
        (
            "src/A.java",
            "class A { int m() { authorize(user); return 1; } }",
            "class A { int m() { authorize(user); return 2; } }",
        ),
        (
            "src/A.java",
            "class A { int m() { return 1; } }",
            "class A { int m() { authorize(user); customSideEffect(); return 2; } }",
        ),
        (
            "api/A.java",
            "class A { public int m() { return 1; } }",
            "class A { public int m(int x) { customSideEffect(); return 1; } }",
        ),
    ];
    let findings =
        risk_analyzer::rules::evaluate(&[change(cases[0].0, cases[0].1, cases[0].2)]).unwrap();
    assert!(
        !findings.iter().any(|f| f.rule == "AUTH_CHANGE"),
        "unchanged call must not trigger: {findings:?}"
    );
    for (path, old, new) in &cases[1..] {
        assert!(
            risk_analyzer::rules::evaluate(&[change(path, old, new)]).is_err(),
            "unknown neighboring statement accepted"
        );
    }
}

#[test]
fn mixed_test_filenames_and_source_constants_do_not_hide_production_changes() {
    let file = change(
        "tests/MixedTest.java",
        "class MixedTest { int compute() { return 1; } }",
        "class MixedTest { int compute() { authorize(user); return 2; } }",
    );
    let findings = risk_analyzer::rules::evaluate(&[file]).unwrap();
    assert!(findings.iter().any(|f| f.rule == "AUTH_CHANGE"));
    assert!(
        risk_analyzer::rules::evaluate(&[change(
            "src/A.java",
            "class A { String x = \"a\"; int m() { return 1; } }",
            "class A { String x = \"unknown\"; int m() { return 2; } }"
        )])
        .is_err()
    );
}

#[test]
fn critical_source_path_comments_and_literals_still_require_readable_source() {
    for (path, old, new) in [
        (
            "src/secrets/A.java",
            "class A {} // old",
            "class A {} // new",
        ),
        (
            "src/auth/A.java",
            "class A { int m() { return 1; } }",
            "class A { int m() { return 2; } }",
        ),
    ] {
        assert!(risk_analyzer::rules::evaluate(&[change(path, old, new)]).is_ok());
    }
    assert!(
        risk_analyzer::rules::evaluate(&[change(
            "src/secrets/A.java",
            "class A {}",
            "not valid java !!!"
        )])
        .is_err()
    );
}

#[test]
fn yaml_contracts_reject_duplicates_aliases_and_complex_schemas() {
    for new in [
        "paths: {}\npaths: {}\n",
        "paths: &x {}\ncomponents: *x\n",
        "components:\n  schemas:\n    A:\n      allOf: []\n",
        "paths: !custom {}\n",
    ] {
        assert!(
            risk_analyzer::rules::evaluate(&[change("openapi.yaml", "paths: {}\n", new)]).is_err()
        );
    }
    let old =
        "components:\n  schemas:\n    A:\n      properties:\n        id:\n          type: string\n";
    let new = format!("{old}        name:\n          type: string\n");
    assert!(
        risk_analyzer::rules::evaluate(&[change("openapi.yaml", old, &new)])
            .unwrap()
            .iter()
            .any(|f| f.rule == "API_EXTENSION")
    );
}

#[test]
fn api_extension_does_not_cover_independent_metadata_changes() {
    let old = r#"{"info":{"title":"old"},"paths":{}}"#;
    let new = r#"{"info":{"title":"new"},"paths":{"/users":{}}}"#;
    assert!(risk_analyzer::rules::evaluate(&[change("openapi.json", old, new)]).is_err());
}

#[test]
fn identifier_components_and_test_class_fields_have_bounded_classification() {
    let findings = risk_analyzer::rules::evaluate(&[change(
        "src/A.java",
        "class A { void m() { user_cache.invalidate(a); } }",
        "class A { void m() { user_cache.invalidate(b); } }",
    )])
    .unwrap();
    assert!(findings.iter().any(|f| f.rule == "CONCURRENCY_CACHE"));
    assert!(
        risk_analyzer::rules::evaluate(&[change(
            "tests/A.java",
            "class A { String secret = \"old\"; }",
            "class A { String secret = \"new\"; }"
        )])
        .is_err()
    );
}

#[test]
fn manifest_and_style_neighbor_changes_are_not_covered_by_a_valid_signal() {
    for (path, old, new) in [
        (
            "package.json",
            r#"{"description":"a","dependencies":{"x":"1"}}"#,
            r#"{"description":"b","dependencies":{"x":"2"}}"#,
        ),
        (
            "build.gradle",
            "plugins { id 'java' }\ndependencies { implementation 'x:1' }\n",
            "plugins { id 'other' }\ndependencies { implementation 'x:2' }\n",
        ),
        (
            "style.css",
            ".a { color: red; display: block; }",
            ".a { color: blue; display: none; }",
        ),
        ("migrations/a.sql", "", "DROP TABLE users; UNKNOWN_OP;"),
        ("migrations/a.sql", "", "SELECT 'unterminated;"),
    ] {
        assert!(
            risk_analyzer::rules::evaluate(&[change(path, old, new)]).is_err(),
            "{path}"
        );
    }
}

#[test]
fn all_reference_pairs_remain_deterministic_and_have_expected_rationale() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/rules/signals.json")).unwrap();
    for c in cases.iter().filter(|c| c["positive"] == true) {
        let file = change(
            c["path"].as_str().unwrap(),
            c["old"].as_str().unwrap(),
            c["new"].as_str().unwrap(),
        );
        let expected = risk_analyzer::rules::evaluate(std::slice::from_ref(&file)).unwrap();
        for _ in 0..10 {
            assert_eq!(
                risk_analyzer::rules::evaluate(std::slice::from_ref(&file)).unwrap(),
                expected
            );
        }
        assert!(expected.iter().all(|f| !f.reason.is_empty()
            && !f.path.is_empty()
            && (f.side == risk_analyzer::model::Side::Metadata || f.line.is_some())));
    }
}

#[test]
fn interpolated_authorization_call_is_not_hidden_as_literal_text() {
    for (path, old, new) in [
        ("src/a.js", "", "const x = `value ${authorize(user)}`;"),
        (
            "src/a.ts",
            "",
            "const x: string = `value ${authorize(user)}`;",
        ),
        ("src/a.py", "", "x = f'value {authorize(user)}'\n"),
    ] {
        let findings = risk_analyzer::rules::evaluate(&[change(path, old, new)]).unwrap();
        assert!(
            findings.iter().any(|f| f.rule == "AUTH_CHANGE"),
            "{path}: {findings:?}"
        );
    }
}

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
