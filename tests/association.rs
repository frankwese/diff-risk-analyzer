use risk_analyzer::{
    association::{self, Status},
    git::diff_lines,
    model::FileChange,
};
fn change(path: &str, old: &str, new: &str) -> FileChange {
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
fn direct_use_in_changed_test_for_four_profiles() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/association/direct.json")).unwrap();
    for c in cases {
        let files = [
            change(
                c["path"].as_str().unwrap(),
                c["old"].as_str().unwrap(),
                c["new"].as_str().unwrap(),
            ),
            change(
                c["test_path"].as_str().unwrap(),
                "",
                c["test"].as_str().unwrap(),
            ),
        ];
        let found = association::evaluate(&files);
        assert!(
            found
                .iter()
                .any(|a| a.symbol.ends_with(".authorize") && a.status == Status::Used),
            "{c}: {found:?}"
        );
    }
}
#[test]
fn test_names_strings_and_other_test_changes_do_not_prove_usage() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let old = "import {Guard} from './guard.js'; test('a', () => { const g = new Guard(); g.authorize(); }); test('b', () => { expect(1); });";
    let new = old.replace("expect(1)", "expect(2)");
    let result =
        association::evaluate(&[production.clone(), change("src/guard.test.js", old, &new)]);
    assert!(result.iter().any(|a| a.status == Status::Missing));
    let test = "import {Guard} from './guard.js'; test('authorize', () => { const name='authorize'; const g = new Guard(); g.other(); });";
    let result = association::evaluate(&[production, change("src/guard.test.js", "", test)]);
    assert!(result.iter().any(|a| a.status == Status::Missing));
}

#[test]
fn conservative_receiver_boundaries() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/association/boundaries.json")).unwrap();
    for case in cases {
        let production = change(
            "src/guard.js",
            "export class Guard { authorize() { return true; } }",
            "export class Guard { authorize() { return false; } }",
        );
        let test = format!(
            "import {{Guard}} from './guard.js'; test('case', () => {{ {} }});",
            case["body"].as_str().unwrap()
        );
        let found = association::evaluate(&[production, change("src/guard.test.js", "", &test)]);
        assert!(
            found.iter().any(|a| a.symbol == "Guard.authorize"
                && a.status.to_string() == case["status"].as_str().unwrap()),
            "{case}: {found:?}"
        );
    }
}

#[test]
fn unchanged_usage_in_changed_test_counts_but_deleted_test_does_not() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let old = "import {Guard} from './guard.js'; test('a', () => { const g=new Guard(); expect(g.authorize()).toBe(true); });";
    let new = old.replace("toBe(true)", "toBe(false)");
    let found =
        association::evaluate(&[production.clone(), change("src/guard.test.js", old, &new)]);
    assert!(found.iter().any(|a| a.status == Status::Used));
    let found = association::evaluate(&[production, change("src/guard.test.js", old, "")]);
    assert!(found.iter().any(|a| a.status == Status::Missing));
}

#[test]
fn independently_changed_methods_have_separate_obligations() {
    let production = change(
        "src/guard.js",
        "export class Guard { a() { return 1; } b() { return 1; } }",
        "export class Guard { a() { return 2; } b() { return 2; } }",
    );
    let test =
        "import {Guard} from './guard.js'; test('a', () => { const g=new Guard(); g.a(); });";
    let found = association::evaluate(&[production, change("src/guard.test.js", "", test)]);
    assert!(
        found
            .iter()
            .any(|a| a.symbol == "Guard.a" && a.status == Status::Used)
    );
    assert!(
        found
            .iter()
            .any(|a| a.symbol == "Guard.b" && a.status == Status::Missing)
    );
}

#[test]
fn class_field_change_and_module_import_are_not_lost() {
    let production = change(
        "src/guard.py",
        "class Guard:\n    enabled = True\n    def authorize(self):\n        return True\n",
        "class Guard:\n    enabled = False\n    def authorize(self):\n        return True\n",
    );
    let test = "import guard\ndef test_guard():\n    g = guard.Guard()\n    assert g.authorize()\n";
    let found = association::evaluate(&[production, change("tests/test_guard.py", "", test)]);
    assert!(
        found
            .iter()
            .any(|a| a.symbol == "Guard" && a.status == Status::Used),
        "{found:?}"
    );
}

#[test]
fn overloaded_methods_and_parser_errors_are_unknown() {
    let production = change(
        "src/main/java/demo/Guard.java",
        "package demo; class Guard { boolean authorize(int x) { return true; } boolean authorize() { return true; } }",
        "package demo; class Guard { boolean authorize(int x) { return false; } boolean authorize() { return true; } }",
    );
    let test = "package demo; import org.junit.jupiter.api.Test; import demo.Guard; class GuardTest { @Test void a() { Guard g=new Guard(); g.authorize(1); } }";
    let found = association::evaluate(&[
        production,
        change("src/test/java/demo/GuardTest.java", "", test),
    ]);
    assert!(found.iter().any(|a| a.status == Status::Unknown));
}

#[test]
fn static_calls_import_aliases_and_default_imports() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/association/direct.json")).unwrap();
    for c in cases {
        let path = c["path"].as_str().unwrap();
        let test_path = c["test_path"].as_str().unwrap();
        let (old, new, test) = if path.ends_with(".java") {
            ("package demo; public class Guard { public static boolean authorize() { return true; } }".into(),"package demo; public class Guard { public static boolean authorize() { return false; } }".into(),"package demo; import demo.Guard; import org.junit.jupiter.api.Test; class GuardTest { @Test void a() { assertTrue(Guard.authorize()); } }".into())
        } else if path.ends_with(".py") {
            (
                "class Guard:\n    @staticmethod\n    def authorize():\n        return True\n"
                    .into(),
                "class Guard:\n    @staticmethod\n    def authorize():\n        return False\n"
                    .into(),
                "from guard import Guard as Alias\ndef test_a():\n    assert Alias.authorize()\n"
                    .into(),
            )
        } else {
            let old = c["old"]
                .as_str()
                .unwrap()
                .replace("authorize()", "static authorize()");
            let new = c["new"]
                .as_str()
                .unwrap()
                .replace("authorize()", "static authorize()");
            let test = format!(
                "import {{Guard as Alias}} from './{}'; test('a', () => {{ expect(Alias.authorize()); }});",
                path.rsplit('/').next().unwrap()
            );
            (old, new, test)
        };
        let result =
            association::evaluate(&[change(path, &old, &new), change(test_path, "", &test)]);
        assert!(
            result
                .iter()
                .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Used),
            "{path}: {result:?}"
        );
    }
    let result = association::evaluate(&[
        change(
            "src/guard.js",
            "export default class Guard { authorize() { return true; } }",
            "export default class Guard { authorize() { return false; } }",
        ),
        change(
            "src/guard.test.js",
            "",
            "import Alias from './guard.js'; test('a', () => { const g=new Alias(); g.authorize(); });",
        ),
    ]);
    assert!(result.iter().any(|a| a.status == Status::Used));
}

#[test]
fn nonstandard_test_filename_is_recognized_by_test_syntax() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let test = "import {Guard} from './guard.js'; test('a', () => { const g=new Guard(); g.authorize(); });";
    let files = [production, change("src/checks.js", "", test)];
    let found = association::evaluate(&files);
    assert!(
        found
            .iter()
            .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Used),
        "{found:?}"
    );
    assert!(risk_analyzer::rules::evaluate(&files).is_ok());
}

#[test]
fn shadowed_framework_and_imported_class_do_not_prove_usage() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    for body in [
        "const Guard=Fake; const g=new Guard(); g.authorize();",
        "const g=new Guard(); g.authorize(); const Guard=Fake;",
    ] {
        let test = format!("import {{Guard}} from './guard.js'; test('a', () => {{ {body} }});");
        let found =
            association::evaluate(&[production.clone(), change("src/guard.test.js", "", &test)]);
        assert!(!found.iter().any(|a| a.status == Status::Used), "{found:?}");
    }
    let test = "import {Guard} from './guard.js'; const test=customRunner; test('a', () => { const g=new Guard(); g.authorize(); });";
    let found = association::evaluate(&[production, change("src/guard.test.js", "", test)]);
    assert!(!found.iter().any(|a| a.status == Status::Used), "{found:?}");
}

#[test]
fn mixed_test_and_production_file_is_not_classified_as_tests_only() {
    let file = change(
        "src/mixed.js",
        "export function authorize() { return true; } test('a', () => { expect(true); });",
        "export function authorize() { return false; } test('a', () => { expect(false); });",
    );
    let findings = risk_analyzer::rules::evaluate(&[file]).unwrap();
    assert!(
        findings.iter().any(|f| f.rule == "AUTH_CONTEXT_CHANGE"),
        "{findings:?}"
    );
}

#[test]
fn callback_parameter_shadowing_and_unknown_structures_are_not_used() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let test = "import {Guard} from './guard.js'; test('a', (Guard) => { const g=new Guard(); g.authorize(); });";
    assert!(
        !association::evaluate(&[production, change("src/guard.test.js", "", test)])
            .iter()
            .any(|a| a.status == Status::Used)
    );
}

#[test]
fn same_test_name_in_different_suites_is_not_ambiguous() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let old = "import {Guard} from './guard.js'; describe('one', () => { test('a', () => { const g=new Guard(); g.authorize(); expect(1); }); }); describe('two', () => { test('a', () => { expect(1); }); });";
    let new = old.replacen("expect(1)", "expect(2)", 1);
    let found = association::evaluate(&[production, change("src/guard.test.js", old, &new)]);
    assert!(
        found
            .iter()
            .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Used),
        "{found:?}"
    );
}

#[test]
fn deleted_method_and_invalid_source_are_explicit() {
    let file = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard {}",
    );
    assert!(
        association::evaluate(&[file])
            .iter()
            .any(|a| a.symbol == "Guard.authorize" && a.status == Status::NotApplicable)
    );
    let file = change(
        "src/guard.js",
        "export class Guard {}",
        "export class Guard { authorize( }",
    );
    assert!(
        association::evaluate(&[file])
            .iter()
            .any(|a| a.status == Status::Unknown)
    );
}

#[test]
fn custom_decorators_and_generated_tests_remain_unknown() {
    let file = change(
        "src/guard.py",
        "class Guard:\n    @classmethod\n    def authorize(cls):\n        return True\n",
        "class Guard:\n    @classmethod\n    def authorize(cls):\n        return False\n",
    );
    let test = "from guard import Guard\ndef test_a():\n    assert Guard.authorize()\n";
    assert!(
        association::evaluate(&[file, change("tests/test_guard.py", "", test)])
            .iter()
            .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Unknown)
    );
    let file = change(
        "src/guard.py",
        "class Guard:\n    def authorize(self):\n        return True\n",
        "class Guard:\n    def authorize(self):\n        return False\n",
    );
    let test = "from guard import Guard\n@custom_test\ndef test_a():\n    g=Guard()\n    assert g.authorize()\n";
    assert!(
        !association::evaluate(&[file, change("tests/test_guard.py", "", test)])
            .iter()
            .any(|a| a.status == Status::Used)
    );
}

#[test]
fn direct_imported_function_calls_are_used() {
    for (path, old, new, test_path, test) in [
        (
            "src/guard.js",
            "export function authorize() { return true; }",
            "export function authorize() { return false; }",
            "src/guard.test.js",
            "import {authorize} from './guard.js'; test('a', () => { expect(authorize()); });",
        ),
        (
            "src/guard.ts",
            "export function authorize(): boolean { return true; }",
            "export function authorize(): boolean { return false; }",
            "src/guard.test.ts",
            "import {authorize as check} from './guard.ts'; test('a', () => { expect(check()); });",
        ),
        (
            "src/guard.py",
            "def authorize():\n    return True\n",
            "def authorize():\n    return False\n",
            "tests/test_guard.py",
            "from guard import authorize\ndef test_a():\n    assert authorize()\n",
        ),
    ] {
        let found = association::evaluate(&[change(path, old, new), change(test_path, "", test)]);
        assert!(
            found
                .iter()
                .any(|a| a.symbol == "authorize" && a.status == Status::Used),
            "{path}: {found:?}"
        );
    }
}

#[test]
fn per_language_missing_other_method_and_unchanged_test_cases() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/association/direct.json")).unwrap();
    for case in cases {
        let production = change(
            case["path"].as_str().unwrap(),
            case["old"].as_str().unwrap(),
            case["new"].as_str().unwrap(),
        );
        let text = case["test"].as_str().unwrap();
        let test_path = case["test_path"].as_str().unwrap();
        let found = association::evaluate(&[
            production.clone(),
            change(test_path, "", &text.replace("g.authorize()", "g.other()")),
        ]);
        assert!(
            found
                .iter()
                .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Missing),
            "other method {case}: {found:?}"
        );
        let unchanged = change(test_path, text, text);
        let found = association::evaluate(&[production, unchanged]);
        assert!(
            found
                .iter()
                .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Missing),
            "unchanged {case}: {found:?}"
        );
    }
}

#[test]
fn indirect_helper_usage_is_unknown_not_missing() {
    let file = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let test = "import {Guard} from './guard.js'; test('a', () => { checkThroughHelper(); });";
    let result = association::evaluate(&[file, change("src/guard.test.js", "", test)]);
    assert!(
        result
            .iter()
            .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Unknown)
    );
}

#[test]
fn mixed_test_file_with_empty_production_class_is_not_tests_only() {
    let file = change(
        "src/mixed.py",
        "class Guard:\n    enabled = True\n\ndef test_a():\n    assert True\n",
        "class Guard:\n    enabled = False\n\ndef test_a():\n    assert False\n",
    );
    assert!(!association::is_test_file(&file));
}

#[test]
fn setup_import_changes_and_line_shifts_do_not_activate_unchanged_tests() {
    let production = change(
        "src/guard.js",
        "export class Guard { authorize() { return true; } }",
        "export class Guard { authorize() { return false; } }",
    );
    let old = "import {Guard} from './guard.js';\ntest('a', () => { const g=new Guard(); g.authorize(); });\n";
    for new in [
        format!("\n{old}"),
        old.replace("import {Guard}", "import {Guard, Other}"),
        format!("{old}\nbeforeEach(() => {{ setup(); }});"),
    ] {
        let found =
            association::evaluate(&[production.clone(), change("src/guard.test.js", old, &new)]);
        assert!(
            found
                .iter()
                .any(|a| a.symbol == "Guard.authorize" && a.status == Status::Missing),
            "{found:?}"
        );
    }
}
