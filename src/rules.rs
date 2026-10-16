use crate::{
    model::{AnalysisError, FileChange, Finding, Level, Side},
    syntax::{self, Profile},
};
use std::collections::BTreeMap;

pub fn is_doc(path: &[u8]) -> bool {
    let path = String::from_utf8_lossy(path).to_lowercase();
    let base = path.rsplit('/').next().unwrap_or_default();
    segments(&path, &["docs", "doc"])
        || [".md", ".rst", ".adoc"].iter().any(|e| path.ends_with(e))
        || ["readme", "changelog", "license"]
            .iter()
            .any(|p| base.starts_with(p))
}
pub fn is_test(path: &[u8]) -> bool {
    let path = String::from_utf8_lossy(path).to_lowercase();
    let base = path.rsplit('/').next().unwrap_or_default();
    !is_doc(path.as_bytes())
        && (segments(&path, &["test", "tests", "__tests__"])
            || base.starts_with("test_")
            || base.ends_with("_test.py")
            || base.starts_with("test") && base.ends_with(".java")
            || base.ends_with("test.java")
            || base.ends_with("tests.java")
            || [".test.js", ".spec.js", ".test.ts", ".spec.ts"]
                .iter()
                .any(|p| base.ends_with(p)))
}
pub fn segments(path: &str, names: &[&str]) -> bool {
    path.split('/').any(|s| names.contains(&s))
}
fn has_word(text: &str, words: &[&str]) -> bool {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|s| {
            if words.iter().any(|w| s.eq_ignore_ascii_case(w)) {
                return true;
            }
            let mut start = 0;
            let chars: Vec<_> = s.char_indices().collect();
            for (i, (offset, c)) in chars.iter().enumerate() {
                let boundary = *c == '_'
                    || i > 0
                        && c.is_uppercase()
                        && (chars[i - 1].1.is_lowercase()
                            || chars.get(i + 1).is_some_and(|(_, n)| n.is_lowercase()));
                if boundary {
                    if words
                        .iter()
                        .any(|w| s[start..*offset].eq_ignore_ascii_case(w))
                    {
                        return true;
                    }
                    start = if *c == '_' { offset + 1 } else { *offset };
                }
            }
            words.iter().any(|w| s[start..].eq_ignore_ascii_case(w))
        })
}
fn finding(
    file: &FileChange,
    rule: &'static str,
    level: Level,
    reason: &'static str,
    side: Side,
    line: Option<usize>,
) -> Finding {
    Finding {
        rule,
        level,
        reason,
        path: if side == Side::Base {
            file.old_path.as_deref().unwrap_or(file.path())
        } else {
            file.path()
        }
        .to_vec(),
        side,
        line,
    }
}

pub fn evaluate(files: &[FileChange]) -> Result<Vec<Finding>, AnalysisError> {
    let mut findings = vec![];
    let mut unsupported = vec![];
    for file in files {
        match evaluate_file(file) {
            Ok(mut f) => findings.append(&mut f),
            Err(_) => unsupported.push(file.path().to_vec()),
        }
    }
    if !unsupported.is_empty() {
        unsupported.sort();
        unsupported.dedup();
        let mut error =
            AnalysisError::new("UNSUPPORTED_CHANGE", "changes contain unsupported regions");
        error.paths = unsupported;
        return Err(error);
    }
    findings.sort_by(|a, b| {
        b.level
            .cmp(&a.level)
            .then(a.rule.cmp(b.rule))
            .then(a.path.cmp(&b.path))
            .then(a.side.cmp(&b.side))
            .then(a.line.cmp(&b.line))
    });
    findings.dedup();
    Ok(findings)
}

fn evaluate_file(file: &FileChange) -> Result<Vec<Finding>, AnalysisError> {
    let unsupported = || AnalysisError::unsupported(file.path());
    let path = String::from_utf8_lossy(file.path()).to_lowercase();
    let base = path.rsplit('/').next().unwrap_or_default();
    let mut result = vec![];
    let auth = segments(
        &path,
        &["auth", "authn", "authz", "authentication", "authorization"],
    );
    let payment = segments(
        &path,
        &["payment", "payments", "billing", "invoice", "pricing"],
    );
    let secret = segments(&path, &["secrets", "secret", "iam"]);
    let prod = segments(&path, &["prod", "production"]);
    if file.exact_rename() {
        let old =
            String::from_utf8_lossy(file.old_path.as_deref().unwrap_or_default()).to_lowercase();
        let roles = [
            (
                auth,
                segments(
                    &old,
                    &["auth", "authn", "authz", "authentication", "authorization"],
                ),
                "AUTH_CHANGE",
                "AuthN/AuthZ path role changed; security impact possible",
            ),
            (
                payment,
                segments(
                    &old,
                    &["payment", "payments", "billing", "invoice", "pricing"],
                ),
                "PAYMENT_CHANGE",
                "Payment path role changed; financial impact possible",
            ),
            (
                secret || prod,
                segments(&old, &["secret", "secrets", "iam", "prod", "production"]),
                "SECRET_IAM_PROD",
                "Secret/IAM/production path role changed",
            ),
        ];
        for (new, old, id, reason) in roles {
            if new && !old && !is_doc(file.path()) && !is_test(file.path()) {
                result.push(finding(file, id, Level::High, reason, Side::Metadata, None));
            }
        }
        if result.is_empty() {
            result.push(finding(
                file,
                "NEUTRAL_RENAME",
                Level::Low,
                "Exact file rename; contents unchanged",
                Side::Metadata,
                None,
            ));
        }
        return Ok(result);
    }
    if is_doc(file.path()) {
        return Ok(vec![finding(
            file,
            "DOC_CHANGE",
            Level::Low,
            "Documentation changed; no executable behavior evaluated",
            Side::Metadata,
            None,
        )]);
    }
    if pure_test_change(file) {
        return Ok(vec![finding(
            file,
            "TEST_CHANGE",
            Level::Low,
            "Test-only changes; production code unchanged in this file",
            Side::Metadata,
            None,
        )]);
    }
    let profile = Profile::for_path(file.path());
    if profile.is_some_and(|p| syntax::neutral(p, &file.old, &file.new)) {
        return Ok(vec![finding(
            file,
            "DOC_CHANGE",
            Level::Low,
            "Only comments or supported formatting changed",
            Side::Metadata,
            None,
        )]);
    }
    let old = std::str::from_utf8(&file.old).map_err(|_| unsupported())?;
    let new = std::str::from_utf8(&file.new).map_err(|_| unsupported())?;
    if file.old.contains(&0) || file.new.contains(&0) {
        return Err(unsupported());
    }
    if let Some(profile) = profile {
        return evaluate_source(file, profile);
    }
    if path.ends_with(".css") {
        if css_visual(old, new) {
            return Ok(vec![finding(
                file,
                "VISUAL_STYLE",
                Level::Low,
                "Local visual properties changed; no data or security effect inferred",
                Side::Metadata,
                None,
            )]);
        }
        return Err(unsupported());
    }
    if lockfile(base) || dependency_change(base, old, new)? {
        return Ok(vec![finding(
            file,
            "DEPENDENCY_UPDATE",
            Level::Medium,
            "Dependency declarations changed; indirect compatibility effects possible",
            Side::Metadata,
            None,
        )]);
    }
    if base.starts_with("openapi.") {
        let a = structured(old, base)?;
        let b = structured(new, base)?;
        validate_openapi(&a)?;
        validate_openapi(&b)?;
        validate_api_delta(&a, &b)?;
        let (added, broken) = api_changes(&a, &b);
        if broken {
            result.push(finding(
                file,
                "CONTRACT_BREAK",
                Level::High,
                "Central API contract break indicated; consumers may be affected",
                Side::Metadata,
                None,
            ));
        }
        if added {
            result.push(finding(
                file,
                "API_EXTENSION",
                Level::Medium,
                "API contract extended; compatibility review needed",
                Side::Metadata,
                None,
            ));
        }
        if result.is_empty() {
            return Err(unsupported());
        }
        return Ok(result);
    }
    let migration = segments(&path, &["migration", "migrations", "migrate"]);
    let contract = segments(&path, &["api", "domain", "contracts"]);
    if contract && let Some(p) = profile {
        let a = syntax::symbols(p, &file.old).ok_or_else(unsupported)?;
        let b = syntax::symbols(p, &file.new).ok_or_else(unsupported)?;
        for s in a.iter().filter(|s| s.public) {
            if !b.iter().any(|t| {
                t.public
                    && s.name == t.name
                    && s.container == t.container
                    && s.signature == t.signature
            }) {
                result.push(finding(
                    file,
                    "CONTRACT_BREAK",
                    Level::High,
                    "Public contract signature removed or changed; consumers may be affected",
                    Side::Base,
                    Some(s.line),
                ));
            }
        }
        for s in b.iter().filter(|s| s.public) {
            if !a
                .iter()
                .any(|t| t.public && s.name == t.name && s.container == t.container)
            {
                result.push(finding(
                    file,
                    "API_EXTENSION",
                    Level::Medium,
                    "Public contract element added; compatibility review needed",
                    Side::Target,
                    Some(s.line),
                ));
            }
        }
    }
    let infra = prod
        && [".tf", ".json", ".yaml", ".yml"]
            .iter()
            .any(|e| path.ends_with(e));
    if secret || infra {
        if base.ends_with(".json") || base.ends_with(".yaml") || base.ends_with(".yml") {
            structured(old, base)?;
            structured(new, base)?;
        }
        if old == new || sql_code(old) == sql_code(new) {
            return Ok(vec![finding(
                file,
                "DOC_CHANGE",
                Level::Low,
                "Only infrastructure comments changed",
                Side::Metadata,
                None,
            )]);
        }
        return Ok(vec![finding(
            file,
            "SECRET_IAM_PROD",
            Level::High,
            "Secret/IAM/production infrastructure changed; security or availability impact possible",
            Side::Metadata,
            None,
        )]);
    }
    let mut any = false;
    let old_tokens = profile.and_then(|p| syntax::code_tokens(p, &file.old));
    let new_tokens = profile.and_then(|p| syntax::code_tokens(p, &file.new));
    let old_tests = profile
        .map(|p| crate::association::test_ranges(p, &file.old))
        .unwrap_or_default();
    let new_tests = profile
        .map(|p| crate::association::test_ranges(p, &file.new))
        .unwrap_or_default();
    let statements = profile.map(|p| {
        (
            statement_regions(p, &file.old),
            statement_regions(p, &file.new),
        )
    });
    let offsets = |source: &[u8]| {
        let mut result = vec![0usize];
        for (index, byte) in source.iter().enumerate() {
            if *byte == b'\n' {
                result.push(index + 1);
            }
        }
        result
    };
    let old_offsets = offsets(&file.old);
    let new_offsets = offsets(&file.new);
    let old_sql = sql_code(old);
    let new_sql = sql_code(new);
    if path.ends_with(".sql") && [old, new].iter().any(|s| !sql_balanced(s)) {
        return Err(unsupported());
    }
    for line in &file.lines {
        let code;
        if profile.is_some() {
            let source = file.text(line.side);
            let tests = if line.side == Side::Base {
                &old_tests
            } else {
                &new_tests
            };
            if let Some(tokens) = if line.side == Side::Base {
                &old_tokens
            } else {
                &new_tokens
            } {
                code = tokens
                    .iter()
                    .filter(|(t, row, column)| {
                        let offsets = if line.side == Side::Base {
                            &old_offsets
                        } else {
                            &new_offsets
                        };
                        let offset = offsets
                            .get(row.saturating_sub(1))
                            .copied()
                            .unwrap_or(source.len())
                            + column;
                        *row == line.number
                            && !t.starts_with("<literal:")
                            && !tests.iter().any(|r| r.contains(&offset))
                    })
                    .map(|(t, _, _)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
            } else {
                return Err(unsupported());
            }
        } else if path.ends_with(".sql") {
            let sql = if line.side == Side::Base {
                &old_sql
            } else {
                &new_sql
            };
            let mut start = 1usize;
            let mut relevant = vec![];
            for statement in sql.split_inclusive(';') {
                let end = start + statement.bytes().filter(|b| *b == b'\n').count();
                if line.number >= start && line.number <= end && !statement.trim().is_empty() {
                    let operation = statement.split_whitespace().next().unwrap_or_default();
                    if ![
                        "SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "ALTER", "DROP",
                        "TRUNCATE",
                    ]
                    .iter()
                    .any(|op| operation.eq_ignore_ascii_case(op))
                    {
                        return Err(unsupported());
                    }
                    relevant.push(statement.trim());
                }
                start = end;
            }
            code = relevant.join(" ");
        } else {
            return Err(unsupported());
        }
        if code.trim().is_empty() {
            if profile.is_some() && (!old_tests.is_empty() || !new_tests.is_empty()) {
                result.push(finding(
                    file,
                    "TEST_CHANGE",
                    Level::Low,
                    "Test scope changed; production code assessed separately",
                    line.side,
                    Some(line.number),
                ));
                any = true;
            }
            continue;
        }
        let mut covered = false;
        let mut add = |condition: bool, id, level, reason| {
            if condition {
                result.push(finding(
                    file,
                    id,
                    level,
                    reason,
                    line.side,
                    Some(line.number),
                ));
                covered = true;
            }
        };
        add(
            auth || has_word(
                &code,
                &[
                    "authenticate",
                    "authorize",
                    "checkPermission",
                    "hasRole",
                    "verifyToken",
                ],
            ),
            "AUTH_CHANGE",
            Level::High,
            "AuthN/AuthZ change detected; security impact possible",
        );
        add(
            payment
                || has_word(
                    &code,
                    &[
                        "charge",
                        "refund",
                        "capturePayment",
                        "calculatePrice",
                        "calculateInvoice",
                    ],
                ),
            "PAYMENT_CHANGE",
            Level::High,
            "Payment/billing change detected; financial impact possible",
        );
        let upper = code.to_uppercase();
        let query_literal = profile.is_some()
            && old_tokens
                .as_ref()
                .into_iter()
                .chain(new_tokens.as_ref())
                .flat_map(|t| t.iter())
                .any(|(token, row, _)| {
                    *row == line.number
                        && token.starts_with("<literal:")
                        && has_word(token, &["SELECT", "INSERT", "UPDATE", "DELETE"])
                })
            && has_word(
                &code,
                &["execute", "executeQuery", "query", "prepareStatement"],
            );
        add(
            migration
                && (upper.contains("DROP TABLE")
                    || upper.contains("DROP COLUMN")
                    || upper.contains("TRUNCATE")
                    || has_word(&code, &["drop_table", "drop_column"])),
            "DESTRUCTIVE_MIGRATION",
            Level::High,
            "Destructive migration operation detected; data loss possible",
        );
        add(
            query_literal
                || has_word(&code, &["SELECT", "INSERT", "UPDATE", "DELETE"])
                || upper.contains("CREATE INDEX")
                || upper.contains("ALTER INDEX")
                || migration
                    && (upper.contains("CREATE TABLE")
                        || upper.contains("ALTER TABLE") && upper.contains("ADD")),
            "QUERY_PERFORMANCE",
            Level::Medium,
            "Database query/index operation changed; behavior or performance impact possible",
        );
        add(
            segments(&path, &["cache", "caching", "concurrency"])
                || has_word(
                    &code,
                    &[
                        "lock",
                        "unlock",
                        "synchronized",
                        "Thread",
                        "Executor",
                        "async",
                        "await",
                        "cache",
                        "invalidate",
                        "evict",
                    ],
                ),
            "CONCURRENCY_CACHE",
            Level::Medium,
            "Concurrency/cache operation changed; hard-to-reproduce behavior possible",
        );
        add(
            has_word(&code, &["getSecret", "putSecret", "rotateSecret"]),
            "SECRET_IAM_PROD",
            Level::High,
            "Secret management operation changed; security impact possible",
        );
        add(
            profile.is_some()
                && (has_word(
                    &code,
                    &[
                        "if", "else", "switch", "case", "match", "for", "while", "return", "throw",
                        "raise",
                    ],
                ) || ["+", "-", "*", "/", "%", "==", "!=", "<", ">"]
                    .iter()
                    .any(|op| code.split_whitespace().any(|w| w == *op))),
            "BUSINESS_LOGIC",
            Level::Medium,
            "Business behavior change detected; edge cases may be introduced",
        );
        if !covered && contract && !result.is_empty() {
            // Only signature/declaration lines are covered by contract findings.
            covered =
                code.contains("public") || code.contains("export") || code.starts_with("def ");
        }
        if !covered
            && code
                .chars()
                .all(|c| c.is_whitespace() || "{};():".contains(c))
        {
            covered = true;
        }
        if !covered {
            return Err(unsupported());
        }
        if let Some((old_statements, new_statements)) = &statements {
            let regions = if line.side == Side::Base {
                old_statements
            } else {
                new_statements
            };
            let opposite = if line.side == Side::Base {
                new_statements
            } else {
                old_statements
            };
            for (start, end, statement) in regions {
                if line.number < *start
                    || line.number > *end
                    || opposite.iter().any(|(_, _, s)| s == statement)
                {
                    continue;
                }
                let recognized = auth
                    || payment
                    || segments(&path, &["cache", "caching", "concurrency"])
                    || has_word(
                        statement,
                        &[
                            "authenticate",
                            "authorize",
                            "checkPermission",
                            "hasRole",
                            "verifyToken",
                            "charge",
                            "refund",
                            "capturePayment",
                            "calculatePrice",
                            "calculateInvoice",
                            "getSecret",
                            "putSecret",
                            "rotateSecret",
                            "lock",
                            "unlock",
                            "invalidate",
                            "evict",
                            "cache",
                            "await",
                            "execute",
                            "executeQuery",
                            "query",
                            "prepareStatement",
                        ],
                    );
                if !recognized && statement.starts_with("<expression>") {
                    return Err(unsupported());
                }
            }
        }
        any = true;
    }
    if result.is_empty() || !any {
        return Err(unsupported());
    }
    Ok(result)
}

fn pure_test_change(file: &FileChange) -> bool {
    if !is_test(file.path()) && !crate::association::is_test_file(file) {
        return false;
    }
    let Some(profile) = Profile::for_path(file.path()) else {
        return is_test(file.path());
    };
    // A conventional test path is evidence of test intent, not permission to hide changed production bodies.
    for (source, side) in [(&file.old, Side::Base), (&file.new, Side::Target)] {
        let Some(symbols) = syntax::symbols(profile, source) else {
            return false;
        };
        let tests = crate::association::test_ranges(profile, source);
        let tree = profile.parse(source).ok();
        let mut changed_field = false;
        if let Some(tree) = tree {
            syntax::walk(tree.root_node(), &mut |node| {
                if matches!(node.kind(), "field_declaration" | "public_field_definition") {
                    let text = syntax::text(node, source);
                    let other = if side == Side::Base {
                        &file.new
                    } else {
                        &file.old
                    };
                    if !String::from_utf8_lossy(other).contains(text) {
                        changed_field = true;
                    }
                }
            });
        }
        if changed_field {
            return false;
        }
        for symbol in symbols.iter().filter(|s| s.kind != "CLASS") {
            if tests.iter().any(|r| r.contains(&symbol.start)) {
                continue;
            }
            if file.lines.iter().any(|l| {
                l.side == side
                    && l.number >= symbol.line
                    && l.number <= source[..symbol.end].iter().filter(|b| **b == b'\n').count() + 1
            }) {
                let body = String::from_utf8_lossy(&symbol.bytes);
                if body
                    .rsplit_once('{')
                    .is_some_and(|(_, tail)| tail.chars().all(|c| c.is_whitespace() || c == '}'))
                {
                    continue;
                }
                return false;
            }
        }
    }
    true
}

#[derive(Clone)]
struct SourceRegion {
    start: usize,
    end: usize,
    line: usize,
    text: String,
    code: String,
    kind: String,
}

fn source_regions(profile: Profile, source: &[u8]) -> Result<Vec<SourceRegion>, AnalysisError> {
    let tree = profile.parse(source).map_err(|_| {
        AnalysisError::new(
            "UNSUPPORTED_CHANGE",
            "source is outside the readable syntax profile",
        )
    })?;
    let tests = crate::association::test_ranges(profile, source);
    let tokens = syntax::code_tokens(profile, source)
        .ok_or_else(|| AnalysisError::new("UNSUPPORTED_CHANGE", "source tokens unavailable"))?;
    let mut offsets = vec![0];
    for (i, b) in source.iter().enumerate() {
        if *b == b'\n' {
            offsets.push(i + 1);
        }
    }
    let mut result = vec![];
    syntax::walk(tree.root_node(), &mut |node| {
        if tests
            .iter()
            .any(|r| node.start_byte() >= r.start && node.end_byte() <= r.end)
        {
            return;
        }
        let kind = node.kind();
        let (start, end) = match kind {
            "return_statement"
            | "throw_statement"
            | "raise_statement"
            | "expression_statement"
            | "local_variable_declaration"
            | "lexical_declaration"
            | "variable_declaration"
            | "field_declaration"
            | "public_field_definition"
            | "import_statement"
            | "import_declaration"
            | "import_from_statement"
            | "package_declaration"
            | "break_statement"
            | "continue_statement"
            | "pass_statement"
            | "delete_statement" => {
                // Nested expressions are owned by their enclosing statement.
                let mut p = node.parent();
                while let Some(parent) = p {
                    if matches!(
                        parent.kind(),
                        "return_statement"
                            | "throw_statement"
                            | "expression_statement"
                            | "local_variable_declaration"
                            | "lexical_declaration"
                            | "variable_declaration"
                            | "field_declaration"
                    ) {
                        return;
                    }
                    p = parent.parent();
                }
                let mut cursor = node.walk();
                if node.named_children(&mut cursor).any(|child| {
                    tests
                        .iter()
                        .any(|r| r.start == child.start_byte() && r.end == child.end_byte())
                }) {
                    return;
                }
                (node.start_byte(), node.end_byte())
            }
            "class_declaration"
            | "class_definition"
            | "method_declaration"
            | "method_definition"
            | "constructor_declaration"
            | "function_definition"
            | "function_declaration" => {
                let Some(body) = node.child_by_field_name("body") else {
                    return;
                };
                (node.start_byte(), body.start_byte())
            }
            "if_statement" | "for_statement" | "while_statement" | "switch_expression"
            | "switch_statement" | "match_statement" | "else_clause" | "elif_clause" => {
                let boundary = node
                    .child_by_field_name("body")
                    .or_else(|| node.child_by_field_name("consequence"))
                    .map(|n| n.start_byte())
                    .unwrap_or(node.end_byte());
                (node.start_byte(), boundary)
            }
            _ => return,
        };
        let selected: Vec<_> = tokens
            .iter()
            .filter(|(_, row, col)| {
                let position = offsets.get(row - 1).copied().unwrap_or(source.len()) + col;
                position >= start && position < end
            })
            .collect();
        if selected.is_empty() {
            return;
        }
        let text = selected
            .iter()
            .map(|(t, _, _)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let code = selected
            .iter()
            .filter(|(t, _, _)| !t.starts_with("<literal:"))
            .map(|(t, _, _)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        result.push(SourceRegion {
            start,
            end,
            line: node.start_position().row + 1,
            text,
            code,
            kind: kind.into(),
        });
    });
    Ok(result)
}

fn evaluate_source(file: &FileChange, profile: Profile) -> Result<Vec<Finding>, AnalysisError> {
    let old = source_regions(profile, &file.old)?;
    let new = source_regions(profile, &file.new)?;
    let path = String::from_utf8_lossy(file.path()).to_lowercase();
    let auth = segments(
        &path,
        &["auth", "authn", "authz", "authentication", "authorization"],
    );
    let payment = segments(
        &path,
        &["payment", "payments", "billing", "invoice", "pricing"],
    );
    let secret = segments(&path, &["secret", "secrets", "iam"]);
    let cache = segments(&path, &["cache", "caching", "concurrency"]);
    let migration = segments(&path, &["migration", "migrations", "migrate"]);
    let contract = segments(&path, &["api", "domain", "contracts"]);
    let old_symbols = syntax::symbols(profile, &file.old).unwrap_or_default();
    let new_symbols = syntax::symbols(profile, &file.new).unwrap_or_default();
    let mut findings = vec![];
    let mut changed = false;
    for (regions, opposite, symbols, opposite_symbols, side, source) in [
        (
            &old,
            &new,
            &old_symbols,
            &new_symbols,
            Side::Base,
            &file.old,
        ),
        (
            &new,
            &old,
            &new_symbols,
            &old_symbols,
            Side::Target,
            &file.new,
        ),
    ] {
        let mut remaining: Vec<_> = opposite.iter().map(|r| r.text.as_str()).collect();
        for region in regions {
            if let Some(i) = remaining.iter().position(|s| *s == region.text) {
                remaining.remove(i);
                continue;
            }
            changed = true;
            let enclosing = symbols
                .iter()
                .filter(|s| s.start <= region.start && s.end >= region.end && s.kind != "CLASS")
                .min_by_key(|s| s.end - s.start);
            let contextual_code = format!(
                "{} {}",
                region.code,
                enclosing.map(|s| s.name.as_str()).unwrap_or_default()
            );
            let code = &contextual_code;
            let mut covered = false;
            let mut add = |condition, id, level, reason| {
                if condition {
                    findings.push(finding(file, id, level, reason, side, Some(region.line)));
                    covered = true;
                }
            };
            let auth_context = enclosing.is_some_and(|symbol| {
                let counterpart = opposite_symbols.iter().find(|s| {
                    s.name == symbol.name
                        && s.container == symbol.container
                        && s.kind == symbol.kind
                });
                auth || auth_function(profile, source, symbol)
                    || counterpart.is_some_and(|s| {
                        auth_function(
                            profile,
                            if side == Side::Base {
                                &file.new
                            } else {
                                &file.old
                            },
                            s,
                        )
                    })
            });
            add(
                auth || has_word(
                    &region.code,
                    &[
                        "authenticate",
                        "authorize",
                        "checkPermission",
                        "hasRole",
                        "verifyToken",
                    ],
                ),
                "AUTH_CHANGE",
                Level::High,
                "AuthN/AuthZ change detected; security impact possible",
            );
            add(
                payment
                    || has_word(
                        code,
                        &[
                            "charge",
                            "refund",
                            "capturePayment",
                            "calculatePrice",
                            "calculateInvoice",
                        ],
                    ),
                "PAYMENT_CHANGE",
                Level::High,
                "Payment/billing change detected; financial impact possible",
            );
            add(
                secret || has_word(code, &["getSecret", "putSecret", "rotateSecret"]),
                "SECRET_IAM_PROD",
                Level::High,
                "Secret management changed; security impact possible",
            );
            add(
                cache
                    || has_word(
                        code,
                        &[
                            "lock",
                            "unlock",
                            "synchronized",
                            "Thread",
                            "Executor",
                            "async",
                            "await",
                            "cache",
                            "invalidate",
                            "evict",
                        ],
                    ),
                "CONCURRENCY_CACHE",
                Level::Medium,
                "Concurrency/cache operation changed; hard-to-reproduce behavior possible",
            );
            add(
                migration && has_word(code, &["drop_table", "drop_column"]),
                "DESTRUCTIVE_MIGRATION",
                Level::High,
                "Destructive migration operation detected; data loss possible",
            );
            add(
                has_word(
                    code,
                    &["execute", "executeQuery", "query", "prepareStatement"],
                ) && has_word(&region.text, &["SELECT", "INSERT", "UPDATE", "DELETE"]),
                "QUERY_PERFORMANCE",
                Level::Medium,
                "Database query operation changed; behavior or performance impact possible",
            );
            let header = matches!(
                region.kind.as_str(),
                "class_declaration"
                    | "class_definition"
                    | "method_declaration"
                    | "method_definition"
                    | "constructor_declaration"
                    | "function_definition"
                    | "function_declaration"
            );
            if contract
                && header
                && let Some(symbol) = symbols.iter().find(|s| s.start == region.start && s.public)
            {
                let exact = opposite_symbols.iter().any(|s| {
                    s.public
                        && s.name == symbol.name
                        && s.container == symbol.container
                        && s.signature == symbol.signature
                });
                if !exact {
                    let existed = opposite_symbols.iter().any(|s| {
                        s.public && s.name == symbol.name && s.container == symbol.container
                    });
                    add(
                        true,
                        if side == Side::Base || existed {
                            "CONTRACT_BREAK"
                        } else {
                            "API_EXTENSION"
                        },
                        if side == Side::Base || existed {
                            Level::High
                        } else {
                            Level::Medium
                        },
                        "Public API contract element added, removed or changed; review consumer compatibility",
                    );
                }
            }
            add(
                !header
                    && (has_word(
                        code,
                        &[
                            "if", "else", "switch", "case", "match", "for", "while", "return",
                            "throw", "raise",
                        ],
                    ) || code.split_whitespace().any(|t| {
                        ["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">="].contains(&t)
                    })),
                "BUSINESS_LOGIC",
                Level::Medium,
                "Business behavior change detected; edge cases may be introduced",
            );
            // Structural headers are supported only when their body changes are independently covered.
            if header {
                covered = true;
            }
            if !covered {
                return Err(AnalysisError::unsupported(file.path()));
            }
            if auth_context {
                findings.push(finding(file,"AUTH_CONTEXT_CHANGE",Level::High,
                    "Executable logic or contract changed in an auth function; authorization context indicates security impact",
                    side,Some(region.line)));
            }
        }
        // Reject changed executable tokens not owned by any supported region or test scope.
        let tests = crate::association::test_ranges(profile, source);
        let tokens = syntax::code_tokens(profile, source).unwrap_or_default();
        let mut offsets = vec![0];
        for (i, b) in source.iter().enumerate() {
            if *b == b'\n' {
                offsets.push(i + 1);
            }
        }
        for (token, row, col) in tokens {
            if !file.lines.iter().any(|l| l.side == side && l.number == row) {
                continue;
            }
            let position = offsets[row - 1] + col;
            if regions
                .iter()
                .any(|r| position >= r.start && position < r.end)
                || tests.iter().any(|r| r.contains(&position))
                || ["{", "}", ";", ":", "export", "default"].contains(&token.as_str())
            {
                continue;
            }
            return Err(AnalysisError::unsupported(file.path()));
        }
    }
    if findings.is_empty() {
        if changed {
            return Err(AnalysisError::unsupported(file.path()));
        }
        findings.push(finding(
            file,
            "DOC_CHANGE",
            Level::Low,
            "Only supported non-executable changes detected",
            Side::Metadata,
            None,
        ));
    }
    Ok(findings)
}

fn auth_function(profile: Profile, source: &[u8], symbol: &syntax::Symbol) -> bool {
    const NAMES: &[&str] = &[
        "authenticate",
        "authorize",
        "checkPermission",
        "hasRole",
        "verifyToken",
    ];
    if NAMES.contains(&symbol.name.as_str()) {
        return true;
    }
    let Ok(tree) = profile.parse(source) else {
        return false;
    };
    let mut found = false;
    syntax::walk(tree.root_node(), &mut |node| {
        if node.start_byte() < symbol.start || node.end_byte() > symbol.end {
            return;
        }
        if !matches!(
            node.kind(),
            "method_invocation" | "call_expression" | "call"
        ) {
            return;
        }
        let mut parent = node.parent();
        while let Some(p) = parent {
            if p.start_byte() == symbol.start && p.end_byte() == symbol.end {
                break;
            }
            if matches!(
                p.kind(),
                "method_declaration"
                    | "method_definition"
                    | "function_definition"
                    | "function_declaration"
                    | "arrow_function"
                    | "function_expression"
                    | "lambda"
            ) {
                return;
            }
            parent = p.parent();
        }
        let name = if profile == Profile::Java {
            node.child_by_field_name("name")
        } else {
            node.child_by_field_name("function").map(|function| {
                function
                    .child_by_field_name("property")
                    .or_else(|| function.child_by_field_name("attribute"))
                    .unwrap_or(function)
            })
        };
        if name.is_some_and(|n| NAMES.contains(&syntax::text(n, source))) {
            found = true;
        }
    });
    found
}

fn statement_regions(profile: Profile, source: &[u8]) -> Vec<(usize, usize, String)> {
    let Ok(tree) = profile.parse(source) else {
        return vec![];
    };
    let mut regions = vec![];
    let tests = crate::association::test_ranges(profile, source);
    syntax::walk(tree.root_node(), &mut |node| {
        if tests
            .iter()
            .any(|r| r.contains(&node.start_byte()) && node.end_byte() <= r.end)
        {
            return;
        }
        if node.kind() == "expression_statement" {
            let mut cursor = node.walk();
            if node.named_children(&mut cursor).any(|child| {
                tests
                    .iter()
                    .any(|r| r.start == child.start_byte() && r.end == child.end_byte())
            }) {
                return;
            }
            let tokens = syntax::code_tokens(profile, &source[node.byte_range()]);
            // Fragment parsers do not accept Java statements; use the original expression text.
            let text = tokens
                .map(|t| {
                    t.into_iter()
                        .filter(|(s, _, _)| !s.starts_with("<literal:"))
                        .map(|(s, _, _)| s)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_else(|| syntax::text(node, source).to_owned());
            regions.push((
                node.start_position().row + 1,
                node.end_position().row + 1,
                format!("<expression>{text}"),
            ));
        }
    });
    regions
}

fn sql_code(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut quote = None;
    let mut block = false;
    let mut line = false;
    while i < bytes.len() {
        let b = bytes[i];
        let next = bytes.get(i + 1).copied();
        if line {
            out.push(if b == b'\n' {
                line = false;
                b'\n'
            } else {
                b' '
            });
            i += 1;
            continue;
        }
        if block {
            if b == b'*' && next == Some(b'/') {
                out.extend_from_slice(b"  ");
                block = false;
                i += 2;
            } else {
                out.push(if b == b'\n' { b'\n' } else { b' ' });
                i += 1;
            }
            continue;
        }
        if let Some(q) = quote {
            out.push(if b == b'\n' { b'\n' } else { b' ' });
            if b == q {
                if next == Some(q) {
                    out.push(b' ');
                    i += 2;
                    continue;
                }
                quote = None;
            }
            i += 1;
            continue;
        }
        if b == b'-' && next == Some(b'-') {
            line = true;
            out.extend_from_slice(b"  ");
            i += 2;
            continue;
        }
        if b == b'/' && next == Some(b'*') {
            block = true;
            out.extend_from_slice(b"  ");
            i += 2;
            continue;
        }
        if b == b'\'' || b == b'"' {
            quote = Some(b);
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

fn sql_balanced(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut quote = None;
    let mut block = false;
    let mut line = false;
    while i < bytes.len() {
        let b = bytes[i];
        let next = bytes.get(i + 1).copied();
        if line {
            if b == b'\n' {
                line = false;
            }
            i += 1;
            continue;
        }
        if block {
            if b == b'*' && next == Some(b'/') {
                block = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if let Some(q) = quote {
            if b == q {
                if next == Some(q) {
                    i += 2;
                    continue;
                }
                quote = None;
            }
            i += 1;
            continue;
        }
        if b == b'-' && next == Some(b'-') {
            line = true;
            i += 2;
            continue;
        }
        if b == b'/' && next == Some(b'*') {
            block = true;
            i += 2;
            continue;
        }
        if b == b'\'' || b == b'"' {
            quote = Some(b);
        }
        i += 1;
    }
    quote.is_none() && !block
}
fn lockfile(base: &str) -> bool {
    [
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "uv.lock",
        "poetry.lock",
    ]
    .contains(&base)
}
fn dependency_change(base: &str, old: &str, new: &str) -> Result<bool, AnalysisError> {
    if base == "package.json" {
        let a = structured(old, base)?;
        let b = structured(new, base)?;
        if !a.is_object() || !b.is_object() {
            return Err(AnalysisError::new(
                "UNSUPPORTED_CHANGE",
                "dependency manifest must be a mapping",
            ));
        }
        let keys = [
            "dependencies",
            "devDependencies",
            "peerDependencies",
            "optionalDependencies",
        ];
        let changed = keys.iter().any(|k| a.get(k) != b.get(k));
        if changed
            && let (Some(a), Some(b)) = (a.as_object(), b.as_object())
            && (a
                .iter()
                .any(|(k, v)| !keys.contains(&k.as_str()) && b.get(k) != Some(v))
                || b.iter()
                    .any(|(k, v)| !keys.contains(&k.as_str()) && a.get(k) != Some(v)))
        {
            return Err(AnalysisError::new(
                "UNSUPPORTED_CHANGE",
                "manifest also contains unsupported metadata changes",
            ));
        }
        return Ok(changed);
    }
    if base.starts_with("requirements") && base.ends_with(".txt") {
        let significant = |s: &str| {
            s.lines()
                .map(|l| l.split('#').next().unwrap_or_default().trim())
                .filter(|l| !l.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        return Ok(significant(old) != significant(new));
    }
    if [
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "pyproject.toml",
    ]
    .contains(&base)
    {
        fn split(base: &str, text: &str) -> (Vec<String>, Vec<String>) {
            let mut dependency = vec![];
            let mut other = vec![];
            let mut active = false;
            let mut brackets = 0i32;
            let mut section = String::new();
            for line in text.lines() {
                let trimmed = line.trim();
                if base == "pyproject.toml" {
                    if trimmed.starts_with('[') && brackets == 0 {
                        section = trimmed.to_owned();
                        active = section == "[project.optional-dependencies]"
                            || section == "[tool.poetry.dependencies]"
                            || section.starts_with("[tool.poetry.group.")
                                && section.ends_with(".dependencies]");
                    }
                    if section == "[project]" && trimmed.starts_with("dependencies") {
                        active = true;
                    }
                } else if base == "pom.xml" {
                    if trimmed.contains("<dependencies>") || trimmed.contains("<dependency>") {
                        active = true;
                    }
                } else if trimmed.starts_with("dependencies") && trimmed.contains('{') {
                    active = true;
                }
                if active {
                    dependency.push(line.into());
                } else {
                    other.push(line.into());
                }
                if base == "pyproject.toml" && section == "[project]" && active {
                    brackets += trimmed.chars().filter(|c| *c == '[').count() as i32
                        - trimmed.chars().filter(|c| *c == ']').count() as i32;
                    if brackets <= 0 {
                        active = false;
                    }
                } else if base == "pom.xml" && trimmed.contains("</dependencies>") {
                    active = false;
                } else if base.starts_with("build.gradle") && active {
                    brackets += trimmed.chars().filter(|c| *c == '{').count() as i32
                        - trimmed.chars().filter(|c| *c == '}').count() as i32;
                    if brackets <= 0 {
                        active = false;
                    }
                }
            }
            (dependency, other)
        }
        let (a, ao) = split(base, old);
        let (b, bo) = split(base, new);
        if a != b && ao != bo {
            return Err(AnalysisError::new(
                "UNSUPPORTED_CHANGE",
                "manifest contains other unclassified changes",
            ));
        }
        return Ok(a != b);
    }
    Ok(false)
}
fn css_visual(old: &str, new: &str) -> bool {
    fn parse(s: &str) -> Option<BTreeMap<String, BTreeMap<String, String>>> {
        let mut rules = BTreeMap::new();
        let mut remaining = s.trim();
        while !remaining.is_empty() {
            let (selector, tail) = remaining.split_once('{')?;
            let (body, rest) = tail.split_once('}')?;
            if selector.trim().is_empty() || selector.contains('@') || body.contains('{') {
                return None;
            }
            let mut map = BTreeMap::new();
            for declaration in body.split(';').filter(|s| !s.trim().is_empty()) {
                let (key, value) = declaration.split_once(':')?;
                if map.insert(key.trim().into(), value.trim().into()).is_some() {
                    return None;
                }
            }
            if rules.insert(selector.trim().into(), map).is_some() {
                return None;
            }
            remaining = rest.trim();
        }
        Some(rules)
    }
    let (Some(a), Some(b)) = (parse(old.trim()), parse(new.trim())) else {
        return false;
    };
    let keys = [
        "color",
        "background-color",
        "margin",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
        "padding",
        "padding-top",
        "padding-right",
        "padding-bottom",
        "padding-left",
        "font-size",
        "font-weight",
    ];
    a.keys().eq(b.keys())
        && a.iter().all(|(selector, am)| {
            let bm = &b[selector];
            am.keys().eq(bm.keys())
                && am.iter().all(|(k, v)| {
                    bm.get(k) == Some(v)
                        || keys.contains(&k.as_str())
                            && !bm[k].contains("url(")
                            && !v.contains("url(")
                })
        })
}

fn structured(text: &str, base: &str) -> Result<serde_json::Value, AnalysisError> {
    crate::structured::parse(text, base.ends_with(".json"))
}
fn validate_openapi(v: &serde_json::Value) -> Result<(), AnalysisError> {
    fn visit(v: &serde_json::Value, depth: usize) -> bool {
        if depth > 64 {
            return false;
        }
        match v {
            serde_json::Value::Object(m) => m.iter().all(|(k, v)| {
                !["$ref", "allOf", "oneOf", "anyOf"].contains(&k.as_str()) && visit(v, depth + 1)
            }),
            serde_json::Value::Array(a) => {
                a.len() < 100000 && a.iter().all(|v| visit(v, depth + 1))
            }
            _ => true,
        }
    }
    if visit(v, 0) {
        Ok(())
    } else {
        Err(AnalysisError::new(
            "UNSUPPORTED_CHANGE",
            "complex OpenAPI references or schemas are unsupported",
        ))
    }
}
fn api_changes(a: &serde_json::Value, b: &serde_json::Value) -> (bool, bool) {
    let mut added = false;
    let mut broken = false;
    fn compare(a: &serde_json::Value, b: &serde_json::Value, added: &mut bool, broken: &mut bool) {
        if let (Some(am), Some(bm)) = (a.as_object(), b.as_object()) {
            for (k, v) in am {
                if let Some(n) = bm.get(k) {
                    if k == "type" && v != n {
                        *broken = true;
                    } else {
                        compare(v, n, added, broken);
                    }
                } else if k != "required" {
                    *broken = true;
                }
            }
            for k in bm.keys() {
                if !am.contains_key(k) {
                    *added = true;
                }
            }
            if let Some(required) = am.get("required").and_then(|v| v.as_array())
                && required.iter().any(|r| {
                    !bm.get("required")
                        .and_then(|v| v.as_array())
                        .is_some_and(|a| a.contains(r))
                })
            {
                *broken = true;
            }
            if let Some(required) = bm.get("required").and_then(|v| v.as_array())
                && required.iter().any(|r| {
                    !am.get("required")
                        .and_then(|v| v.as_array())
                        .is_some_and(|old| old.contains(r))
                })
            {
                *broken = true;
            }
        }
    }
    for key in ["paths", "components"] {
        compare(
            &a.get(key).cloned().unwrap_or(serde_json::json!({})),
            &b.get(key).cloned().unwrap_or(serde_json::json!({})),
            &mut added,
            &mut broken,
        );
    }
    (added, broken)
}

fn validate_api_delta(a: &serde_json::Value, b: &serde_json::Value) -> Result<(), AnalysisError> {
    fn check(a: &serde_json::Value, b: &serde_json::Value, key: &str) -> bool {
        if a == b {
            return true;
        }
        match (a, b) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                a.iter().all(|(k, v)| match b.get(k) {
                    Some(n) => check(v, n, k),
                    None => ["paths", "properties", "schemas"].contains(&key),
                }) && b.iter().all(|(k, _)| {
                    a.contains_key(k)
                        || ["paths", "properties", "schemas"].contains(&key)
                        || k == "required"
                })
            }
            _ => key == "type" || key == "required",
        }
    }
    let empty = serde_json::json!({});
    if a == &empty || b == &empty || check(a, b, "") {
        Ok(())
    } else {
        Err(AnalysisError::new(
            "UNSUPPORTED_CHANGE",
            "API delta contains unclassified contract changes",
        ))
    }
}
