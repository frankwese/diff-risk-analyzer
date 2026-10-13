use crate::{
    model::{FileChange, escape},
    rules,
    syntax::{self, Profile, Symbol},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Used,
    Missing,
    Unknown,
    NotApplicable,
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Used => "USED",
            Self::Missing => "MISSING",
            Self::Unknown => "UNKNOWN",
            Self::NotApplicable => "NOT_APPLICABLE",
        })
    }
}
#[derive(Debug, Clone)]
pub struct Association {
    pub path: Vec<u8>,
    pub symbol: String,
    pub status: Status,
    pub evidence: Vec<String>,
    pub reason: &'static str,
}

#[derive(Clone)]
struct TestCase {
    name: String,
    body_start: usize,
    body_end: usize,
    bytes: Vec<u8>,
    start: usize,
    end: usize,
    supported: bool,
}

fn test_cases(profile: Profile, source: &[u8]) -> Option<Vec<TestCase>> {
    let tree = profile.parse(source).ok()?;
    let mut framework_shadowed = false;
    if matches!(profile, Profile::JavaScript | Profile::TypeScript) {
        syntax::walk(tree.root_node(), &mut |n| {
            if matches!(
                n.kind(),
                "variable_declarator" | "function_declaration" | "assignment_expression"
            ) && n
                .child_by_field_name("name")
                .or_else(|| n.child_by_field_name("left"))
                .is_some_and(|name| ["test", "it"].contains(&syntax::text(name, source)))
            {
                framework_shadowed = true;
            }
        });
    }
    let mut result = vec![];
    syntax::walk(tree.root_node(), &mut |node| {
        let test = match profile {
            Profile::Java => {
                if node.kind() != "method_declaration" {
                    false
                } else {
                    let mut annotation = false;
                    syntax::walk(node, &mut |n| {
                        if matches!(n.kind(), "marker_annotation" | "annotation")
                            && let Some(name) = n.child_by_field_name("name")
                        {
                            let name = syntax::text(name, source);
                            if name == "org.junit.jupiter.api.Test"
                                || name == "Test"
                                    && String::from_utf8_lossy(source)
                                        .contains("import org.junit.jupiter.api.Test;")
                            {
                                annotation = true;
                            }
                        }
                    });
                    annotation
                }
            }
            Profile::Python => {
                node.kind() == "function_definition"
                    && node
                        .child_by_field_name("name")
                        .is_some_and(|n| syntax::text(n, source).starts_with("test_"))
            }
            _ => {
                !framework_shadowed
                    && node.kind() == "call_expression"
                    && node
                        .child_by_field_name("function")
                        .is_some_and(|n| ["test", "it"].contains(&syntax::text(n, source)))
            }
        };
        if !test {
            return;
        }
        let (name, body) = if matches!(profile, Profile::Java | Profile::Python) {
            let Some(name) = node.child_by_field_name("name") else {
                return;
            };
            let Some(body) = node.child_by_field_name("body") else {
                return;
            };
            (syntax::text(name, source).to_owned(), body)
        } else {
            let Some(args) = node.child_by_field_name("arguments") else {
                return;
            };
            let mut cursor = args.walk();
            let children: Vec<_> = args.named_children(&mut cursor).collect();
            if children.len() != 2
                || children[0].kind() != "string"
                || !matches!(children[1].kind(), "arrow_function" | "function_expression")
            {
                return;
            }
            let Some(body) = children[1].child_by_field_name("body") else {
                return;
            };
            (syntax::text(children[0], source).to_owned(), body)
        };
        result.push(TestCase {
            name: {
                let mut containers = vec![];
                let mut parent = node.parent();
                while let Some(p) = parent {
                    if matches!(p.kind(), "class_declaration" | "class_definition")
                        && let Some(n) = p.child_by_field_name("name")
                    {
                        containers.push(syntax::text(n, source).to_owned());
                    }
                    if p.kind() == "call_expression"
                        && p.child_by_field_name("function")
                            .is_some_and(|n| syntax::text(n, source) == "describe")
                        && let Some(args) = p.child_by_field_name("arguments")
                    {
                        let mut c = args.walk();
                        if let Some(n) = args.named_children(&mut c).next() {
                            containers.push(syntax::text(n, source).to_owned());
                        }
                    }
                    parent = p.parent();
                }
                containers.reverse();
                containers.push(name);
                containers.join("::")
            },
            body_start: body.start_byte(),
            body_end: body.end_byte(),
            bytes: source[node.byte_range()].to_vec(),
            start: node.start_byte(),
            end: node.end_byte(),
            supported: !node
                .parent()
                .is_some_and(|p| p.kind() == "decorated_definition"),
        });
    });
    Some(result)
}

pub fn test_ranges(profile: Profile, source: &[u8]) -> Vec<std::ops::Range<usize>> {
    test_cases(profile, source)
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.start..t.end)
        .collect()
}

pub fn is_test_file(file: &FileChange) -> bool {
    if rules::is_doc(file.path()) {
        return false;
    }
    rules::is_test(file.path())
        || Profile::for_path(file.path()).is_some_and(|p| {
            [&file.old, &file.new]
                .iter()
                .any(|source| test_cases(p, source).is_some_and(|t| !t.is_empty()))
                && [&file.old, &file.new].iter().all(|source| {
                    let ranges = test_ranges(p, source);
                    syntax::symbols(p, source).is_some_and(|symbols| {
                        symbols.iter().all(|s| {
                            ranges.iter().any(|r| r.contains(&s.start))
                                || s.kind == "CLASS"
                                    && symbols.iter().any(|m| m.container == s.name)
                                    && symbols
                                        .iter()
                                        .filter(|m| m.container == s.name)
                                        .all(|m| ranges.iter().any(|r| r.contains(&m.start)))
                        })
                    })
                })
        })
}

pub fn evaluate(files: &[FileChange]) -> Vec<Association> {
    let mut result = vec![];
    for file in files.iter().filter(|f| {
        !rules::is_doc(f.path()) && !is_test_file(f) && Profile::for_path(f.path()).is_some()
    }) {
        if file.exact_rename() {
            continue;
        }
        let profile = Profile::for_path(file.path()).unwrap();
        let Some(new) = syntax::symbols(profile, &file.new) else {
            result.push(Association {
                path: file.path().to_vec(),
                symbol: "<unresolved>".into(),
                status: Status::Unknown,
                evidence: vec![],
                reason: "SOURCE_SYNTAX_UNRESOLVED",
            });
            continue;
        };
        let Some(old) = syntax::symbols(profile, &file.old) else {
            result.push(Association {
                path: file.path().to_vec(),
                symbol: "<unresolved>".into(),
                status: Status::Unknown,
                evidence: vec![],
                reason: "BASE_SYNTAX_UNRESOLVED",
            });
            continue;
        };
        if new.is_empty() && old.is_empty() && file.old != file.new {
            result.push(Association {
                path: file.path().to_vec(),
                symbol: "<unresolved>".into(),
                status: Status::Unknown,
                evidence: vec![],
                reason: "SYMBOL_STRUCTURE_UNRESOLVED",
            });
        }
        for symbol in &new {
            if test_ranges(profile, &file.new)
                .iter()
                .any(|r| r.contains(&symbol.start))
            {
                continue;
            }
            let previous = old.iter().find(|s| same_symbol(s, symbol));
            if previous.is_some_and(|p| p.bytes == symbol.bytes) {
                continue;
            }
            // Method changes are not also a class-level test obligation.
            if symbol.kind == "CLASS"
                && previous.is_some_and(|p| p.class_context == symbol.class_context)
                && new
                    .iter()
                    .any(|s| s.container == symbol.name && s.kind == "METHOD")
            {
                continue;
            }
            let mut evidence = vec![];
            let mut unknown = !symbol.supported;
            let overload = new
                .iter()
                .filter(|s| s.name == symbol.name && s.container == symbol.container)
                .count()
                > 1;
            for test_file in files
                .iter()
                .filter(|f| is_test_file(f) && Profile::for_path(f.path()) == Some(profile))
            {
                let Some(tests) = test_cases(profile, &test_file.new) else {
                    unknown = true;
                    continue;
                };
                let Some(old_tests) = test_cases(profile, &test_file.old) else {
                    unknown = true;
                    continue;
                };
                for test in &tests {
                    if tests.iter().filter(|t| t.name == test.name).count() != 1 {
                        unknown = true;
                        continue;
                    }
                    let previous: Vec<_> =
                        old_tests.iter().filter(|t| t.name == test.name).collect();
                    if previous.len() > 1 {
                        unknown = true;
                        continue;
                    }
                    if previous.first().is_some_and(|t| t.bytes == test.bytes) {
                        continue;
                    }
                    if !test.supported || !symbol.supported {
                        unknown = true;
                        continue;
                    }
                    match usage(profile, file, symbol, test_file, test, files, overload) {
                        Usage::Used(line) => evidence.push(format!(
                            "{}::{} (target:{line})",
                            escape(test_file.path()),
                            test.name
                        )),
                        Usage::Unknown => unknown = true,
                        Usage::Absent => {}
                    }
                }
            }
            evidence.sort();
            evidence.dedup();
            result.push(Association {
                path: file.path().to_vec(),
                symbol: symbol_name(symbol),
                status: if !evidence.is_empty() {
                    Status::Used
                } else if unknown {
                    Status::Unknown
                } else {
                    Status::Missing
                },
                reason: if unknown {
                    "BINDING_UNRESOLVED"
                } else {
                    "no changed test using this symbol detected"
                },
                evidence,
            });
        }
        for symbol in old
            .iter()
            .filter(|s| !new.iter().any(|n| same_symbol(s, n)))
        {
            if test_ranges(profile, &file.old)
                .iter()
                .any(|r| r.contains(&symbol.start))
            {
                continue;
            }
            result.push(Association {
                path: file.path().to_vec(),
                symbol: symbol_name(symbol),
                status: Status::NotApplicable,
                evidence: vec![],
                reason: "symbol deleted",
            });
        }
    }
    result.sort_by(|a, b| a.path.cmp(&b.path).then(a.symbol.cmp(&b.symbol)));
    result
}
fn same_symbol(a: &Symbol, b: &Symbol) -> bool {
    a.name == b.name && a.container == b.container && a.signature == b.signature
}
fn symbol_name(s: &Symbol) -> String {
    if s.container.is_empty() {
        s.name.clone()
    } else {
        format!("{}.{}", s.container, s.name)
    }
}
enum Usage {
    Used(usize),
    Absent,
    Unknown,
}

fn usage(
    profile: Profile,
    production: &FileChange,
    symbol: &Symbol,
    file: &FileChange,
    test: &TestCase,
    files: &[FileChange],
    overload: bool,
) -> Usage {
    let source = &file.new;
    let tree = match profile.parse(source) {
        Ok(t) => t,
        Err(_) => return Usage::Unknown,
    };
    let class = if symbol.kind == "CLASS" || symbol.kind == "FUNCTION" {
        &symbol.name
    } else {
        &symbol.container
    };
    let bindings = imports(profile, production, class, file, files);
    let mut invalid_bindings = std::collections::BTreeSet::new();
    syntax::walk(tree.root_node(), &mut |n| {
        if matches!(
            n.kind(),
            "formal_parameters" | "parameters" | "lambda_parameters"
        ) && n.start_byte() >= test.start
            && n.end_byte() <= test.end
        {
            syntax::walk(n, &mut |parameter| {
                let name = syntax::text(parameter, source);
                if parameter.kind() == "identifier" && bindings.iter().any(|b| b == name) {
                    invalid_bindings.insert(name.to_owned());
                }
            });
        }
    });
    let mut constructors = std::collections::BTreeMap::<String, Option<String>>::new();
    let mut invalid_receivers = std::collections::BTreeSet::new();
    let mut assignments = std::collections::BTreeMap::<String, usize>::new();
    syntax::walk(tree.root_node(), &mut |node| {
        if node.start_byte() < test.body_start || node.end_byte() > test.body_end {
            return;
        }
        if matches!(
            node.kind(),
            "variable_declarator" | "assignment" | "assignment_expression"
        ) && let Some(n) = node
            .child_by_field_name("name")
            .or_else(|| node.child_by_field_name("left"))
        {
            let name = syntax::text(n, source).to_owned();
            if bindings.contains(&name) {
                invalid_bindings.insert(name.clone());
            }
            let count = assignments.entry(name.clone()).or_default();
            *count += 1;
            if *count > 1 {
                invalid_receivers.insert(name);
            }
        }
    });
    let mut relevant = false;
    let mut used = None;
    let mut unknown = false;
    syntax::walk(tree.root_node(), &mut |node| {
        if node.start_byte() < test.body_start || node.end_byte() > test.body_end {
            return;
        }
        let mut parent = node.parent();
        let mut nested = false;
        while let Some(p) = parent {
            if p.start_byte() <= test.body_start {
                break;
            }
            if matches!(
                p.kind(),
                "arrow_function"
                    | "function_expression"
                    | "lambda"
                    | "function_definition"
                    | "if_statement"
                    | "for_statement"
                    | "while_statement"
            ) {
                nested = true;
            }
            parent = p.parent();
        }
        if matches!(node.kind(), "variable_declarator" | "assignment") {
            let name = node
                .child_by_field_name("name")
                .or_else(|| node.child_by_field_name("left"));
            let value = node
                .child_by_field_name("value")
                .or_else(|| node.child_by_field_name("right"));
            if let (Some(name), Some(value)) = (name, value) {
                let name = syntax::text(name, source).to_owned();
                let constructor = constructor_name(profile, value, source);
                let bound = constructor;
                match constructors.entry(name) {
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        entry.insert(None);
                        unknown = true;
                    }
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(if nested { None } else { bound });
                    }
                }
            }
        }
        if symbol.kind == "CLASS"
            && let Some(name) = constructor_name(profile, node, source)
            && bindings.contains(&name)
        {
            relevant = true;
            if !nested && !overload {
                used = Some(node.start_position().row + 1);
            } else {
                unknown = true;
            }
        }
        if !matches!(
            node.kind(),
            "call_expression" | "method_invocation" | "call"
        ) {
            return;
        }
        let (receiver, name) = if profile == Profile::Java {
            (
                node.child_by_field_name("object"),
                node.child_by_field_name("name"),
            )
        } else {
            let function = node.child_by_field_name("function");
            match function {
                Some(f) if matches!(f.kind(), "member_expression" | "attribute") => (
                    f.child_by_field_name("object"),
                    f.child_by_field_name("property")
                        .or_else(|| f.child_by_field_name("attribute")),
                ),
                Some(f) => (None, Some(f)),
                None => (None, None),
            }
        };
        let Some(name) = name else {
            return;
        };
        let name = syntax::text(name, source);
        if !bindings.is_empty()
            && receiver.is_none()
            && ![
                "expect",
                "assertTrue",
                "assertFalse",
                "assertEquals",
                "assertEqual",
                "assertNotNull",
            ]
            .contains(&name)
            && name != symbol.name
            && !bindings.iter().any(|b| b == name)
        {
            // A helper could call the target, but this MVP does not traverse its body.
            relevant = true;
            unknown = true;
        }
        if name != symbol.name
            && !(symbol.kind == "CLASS" && name == class)
            && !(symbol.kind == "FUNCTION" && bindings.iter().any(|b| b == name))
        {
            return;
        }
        relevant = true;
        if nested || overload {
            unknown = true;
            return;
        }
        if symbol.kind == "FUNCTION"
            && receiver.is_none()
            && bindings.iter().any(|b| b == name)
            && !invalid_bindings.contains(name)
        {
            used = Some(node.start_position().row + 1);
            return;
        }
        let Some(receiver) = receiver else {
            if symbol.kind != "CLASS" {
                unknown = true;
            }
            return;
        };
        let receiver_text = syntax::text(receiver, source);
        let bound = !invalid_receivers.contains(receiver_text)
            && !invalid_bindings.contains(receiver_text)
            && (constructors
                .get(receiver_text)
                .and_then(|v| v.as_ref())
                .is_some_and(|n| bindings.contains(n) && !invalid_bindings.contains(n))
                || constructor_name(profile, receiver, source)
                    .is_some_and(|n| bindings.contains(&n) && !invalid_bindings.contains(&n))
                || symbol.static_ && bindings.iter().any(|b| b == receiver_text));
        if bound {
            used = Some(node.start_position().row + 1);
        } else {
            // Explicitly constructed different class is not an ambiguous target receiver.
            let other = constructors
                .get(receiver_text)
                .and_then(|v| v.as_ref())
                .is_some_and(|n| !bindings.contains(n))
                && !invalid_receivers.contains(receiver_text);
            if !other {
                unknown = true;
            }
        }
    });
    if let Some(line) = used {
        Usage::Used(line)
    } else if relevant && unknown {
        Usage::Unknown
    } else {
        Usage::Absent
    }
}

fn constructor_name(
    profile: Profile,
    node: tree_sitter::Node<'_>,
    source: &[u8],
) -> Option<String> {
    let n = match (profile, node.kind()) {
        (Profile::Java, "object_creation_expression") => node.child_by_field_name("type"),
        (Profile::JavaScript | Profile::TypeScript, "new_expression") => {
            node.child_by_field_name("constructor")
        }
        (Profile::Python, "call") => node.child_by_field_name("function"),
        _ => None,
    }?;
    Some(syntax::text(n, source).to_owned())
}
fn imports(
    profile: Profile,
    production: &FileChange,
    class: &str,
    file: &FileChange,
    files: &[FileChange],
) -> Vec<String> {
    let source = &file.new;
    let tree = match profile.parse(source) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    let mut result = vec![];
    let path = String::from_utf8_lossy(production.path());
    match profile {
        Profile::Java => {
            let code = String::from_utf8_lossy(&production.new);
            let package = code
                .split("package ")
                .nth(1)
                .and_then(|s| s.split(';').next())
                .unwrap_or_default()
                .trim();
            let fqn = if package.is_empty() {
                class.into()
            } else {
                format!("{package}.{class}")
            };
            syntax::walk(tree.root_node(), &mut |n| {
                if n.kind() == "import_declaration"
                    && syntax::text(n, source).trim() == format!("import {fqn};")
                {
                    result.push(class.to_owned());
                }
            });
            result.push(fqn);
            let test_code = String::from_utf8_lossy(source);
            let test_package = test_code
                .split("package ")
                .nth(1)
                .and_then(|s| s.split(';').next())
                .unwrap_or_default()
                .trim();
            let module = |p: &[u8]| {
                String::from_utf8_lossy(p)
                    .split("src/")
                    .next()
                    .unwrap_or_default()
                    .to_owned()
            };
            if !package.is_empty()
                && package == test_package
                && module(production.path()) == module(file.path())
                && files
                    .iter()
                    .filter(|f| {
                        !rules::is_test(f.path())
                            && syntax::symbols(profile, &f.new).is_some_and(|s| {
                                s.iter().any(|s| s.kind == "CLASS" && s.name == class)
                            })
                    })
                    .count()
                    == 1
            {
                result.push(class.to_owned());
            }
        }
        Profile::JavaScript | Profile::TypeScript => syntax::walk(tree.root_node(), &mut |n| {
            if n.kind() != "import_statement" {
                return;
            }
            let Some(module) = n.child_by_field_name("source") else {
                return;
            };
            let module = syntax::text(module, source).trim_matches(['\'', '"']);
            if !module.starts_with('.') {
                return;
            }
            let test_path = String::from_utf8_lossy(file.path());
            let dir = test_path
                .rsplit_once('/')
                .map(|(d, _)| d)
                .unwrap_or_default();
            let resolved = normalize(&format!("{dir}/{module}"));
            let candidates: Vec<_> = files
                .iter()
                .filter(|f| !rules::is_test(f.path()))
                .filter(|f| {
                    let p = String::from_utf8_lossy(f.path());
                    p == resolved
                        || [".js", ".ts", ".mjs", ".cjs", "/index.js", "/index.ts"]
                            .iter()
                            .any(|ext| p == format!("{resolved}{ext}"))
                })
                .collect();
            if candidates.len() != 1 || candidates[0].path() != production.path() {
                return;
            }
            syntax::walk(n, &mut |s| {
                if s.kind() == "import_specifier"
                    && s.child_by_field_name("name")
                        .is_some_and(|name| syntax::text(name, source) == class)
                    && !syntax::text(n, source).starts_with("import type ")
                {
                    result.push(
                        s.child_by_field_name("alias")
                            .map(|a| syntax::text(a, source))
                            .unwrap_or(class)
                            .to_owned(),
                    );
                }
            });
            let default_export = syntax::text(tree.root_node(), source);
            let production_text = String::from_utf8_lossy(&production.new);
            if production_text.contains(&format!("export default class {class}")) {
                syntax::walk(n, &mut |s| {
                    if s.kind() == "import_clause" {
                        let mut cursor = s.walk();
                        for child in s.named_children(&mut cursor) {
                            if child.kind() == "identifier"
                                && !default_export.starts_with("import type ")
                            {
                                result.push(syntax::text(child, source).into());
                            }
                        }
                    }
                });
            }
        }),
        Profile::Python => syntax::walk(tree.root_node(), &mut |n| {
            let target = path
                .strip_prefix("src/")
                .unwrap_or(&path)
                .trim_end_matches(".py")
                .trim_end_matches("/__init__");
            if n.kind() == "import_statement" {
                let mut cursor = n.walk();
                for child in n.named_children(&mut cursor) {
                    if child.kind() == "dotted_name"
                        && syntax::text(child, source).replace('.', "/") == target
                    {
                        result.push(format!("{}.{class}", syntax::text(child, source)));
                    }
                    if child.kind() == "aliased_import"
                        && child.child_by_field_name("name").is_some_and(|name| {
                            syntax::text(name, source).replace('.', "/") == target
                        })
                        && let Some(alias) = child.child_by_field_name("alias")
                    {
                        result.push(format!("{}.{class}", syntax::text(alias, source)));
                    }
                }
                return;
            }
            if n.kind() != "import_from_statement" {
                return;
            }
            let Some(module) = n.child_by_field_name("module_name") else {
                return;
            };
            let module = syntax::text(module, source).replace('.', "/");
            let target = path
                .strip_prefix("src/")
                .unwrap_or(&path)
                .trim_end_matches(".py")
                .trim_end_matches("/__init__");
            if target != module {
                return;
            }
            let count = files
                .iter()
                .filter(|f| !rules::is_test(f.path()))
                .filter(|f| {
                    let p = String::from_utf8_lossy(f.path());
                    p.strip_prefix("src/")
                        .unwrap_or(&p)
                        .trim_end_matches(".py")
                        .trim_end_matches("/__init__")
                        == target
                })
                .count();
            if count != 1 {
                return;
            }
            let mut cursor = n.walk();
            for child in n.named_children(&mut cursor) {
                if child.kind() == "dotted_name" && syntax::text(child, source) == class {
                    result.push(class.to_owned());
                }
                if child.kind() == "aliased_import"
                    && child
                        .child_by_field_name("name")
                        .is_some_and(|name| syntax::text(name, source) == class)
                    && let Some(alias) = child.child_by_field_name("alias")
                {
                    result.push(syntax::text(alias, source).into());
                }
            }
        }),
    }
    result
}
fn normalize(path: &str) -> String {
    let mut parts = vec![];
    for p in path.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}
