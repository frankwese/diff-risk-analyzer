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
        .any(|s| words.iter().any(|w| s.eq_ignore_ascii_case(w)))
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
    if crate::association::is_test_file(file) {
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
        return Ok(true);
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
                        active = section.starts_with("[project.optional-dependencies")
                            || section.starts_with("[tool.poetry.dependencies")
                            || section.starts_with("[tool.poetry.group.");
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
