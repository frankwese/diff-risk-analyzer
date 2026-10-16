use tree_sitter::{Language, Parser, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Java,
    JavaScript,
    TypeScript,
    Python,
}

impl Profile {
    pub fn for_path(path: &[u8]) -> Option<Self> {
        let path = std::str::from_utf8(path).ok()?;
        match path.rsplit('.').next()? {
            "java" => Some(Self::Java),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" => Some(Self::TypeScript),
            "py" => Some(Self::Python),
            _ => None,
        }
    }
    fn language(self) -> Language {
        match self {
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
        }
    }

    pub fn parse(self, source: &[u8]) -> Result<Tree, &'static str> {
        std::str::from_utf8(source).map_err(|_| "source is not UTF-8")?;
        if self == Self::Java && source.windows(2).any(|w| w == b"\\u") {
            return Err("Java Unicode escapes are outside the supported syntax profile");
        }
        let mut parser = Parser::new();
        parser
            .set_language(&self.language())
            .map_err(|_| "incompatible grammar ABI")?;
        let tree = parser.parse(source, None).ok_or("parser cancelled")?;
        if tree.root_node().has_error() {
            Err("source contains syntax errors")
        } else {
            let mut cursor = tree.walk();
            let mut depth = 0usize;
            loop {
                if cursor.goto_first_child() {
                    depth += 1;
                    if depth > 128 {
                        return Err("syntax nesting exceeds supported depth");
                    }
                } else {
                    while !cursor.goto_next_sibling() {
                        if !cursor.goto_parent() {
                            drop(cursor);
                            return Ok(tree);
                        }
                        depth = depth.saturating_sub(1);
                    }
                }
            }
        }
    }
}

pub fn text<'a>(node: tree_sitter::Node<'_>, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.byte_range()]).unwrap_or_default()
}

pub fn walk(node: tree_sitter::Node<'_>, f: &mut impl FnMut(tree_sitter::Node<'_>)) {
    f(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, f);
    }
}

pub fn code_tokens(profile: Profile, source: &[u8]) -> Option<Vec<(String, usize, usize)>> {
    if profile == Profile::Java && source.windows(2).any(|w| w == b"\\u") {
        return None;
    }
    let tree = profile.parse(source).ok()?;
    let mut tokens = vec![];
    fn collect(
        node: tree_sitter::Node<'_>,
        source: &[u8],
        tokens: &mut Vec<(String, usize, usize)>,
    ) {
        if node.kind().contains("comment") {
            return;
        }
        if matches!(
            node.kind(),
            "string" | "string_literal" | "character_literal" | "template_string" | "regex"
        ) {
            tokens.push((
                format!("<literal:{}>", text(node, source)),
                node.start_position().row + 1,
                node.start_position().column,
            ));
            // Literal text is opaque, but interpolation expressions execute at runtime.
            fn interpolations(
                node: tree_sitter::Node<'_>,
                source: &[u8],
                tokens: &mut Vec<(String, usize, usize)>,
            ) {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if matches!(child.kind(), "template_substitution" | "interpolation") {
                        let mut expression_cursor = child.walk();
                        for expression in child.named_children(&mut expression_cursor) {
                            collect(expression, source, tokens);
                        }
                    } else {
                        interpolations(child, source, tokens);
                    }
                }
            }
            interpolations(node, source, tokens);
            return;
        }
        if node.child_count() == 0 {
            tokens.push((
                text(node, source).to_owned(),
                node.start_position().row + 1,
                node.start_position().column,
            ));
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect(child, source, tokens);
            }
        }
    }
    collect(tree.root_node(), source, &mut tokens);
    Some(tokens)
}

pub fn neutral(profile: Profile, old: &[u8], new: &[u8]) -> bool {
    let (Some(a), Some(b)) = (code_tokens(profile, old), code_tokens(profile, new)) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(&b).all(|(a, b)| {
        a.0 == b.0
            && match profile {
                Profile::Java => true,
                Profile::Python => a.1 == b.1 && a.2 == b.2,
                Profile::JavaScript | Profile::TypeScript => a.1 == b.1,
            }
    })
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub container: String,
    pub kind: &'static str,
    pub signature: String,
    pub public: bool,
    pub static_: bool,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub bytes: Vec<u8>,
    pub class_context: Vec<u8>,
    pub supported: bool,
}

pub fn symbols(profile: Profile, source: &[u8]) -> Option<Vec<Symbol>> {
    let tree = profile.parse(source).ok()?;
    let mut symbols = vec![];
    walk(tree.root_node(), &mut |node| {
        let kind = match node.kind() {
            "class_declaration" | "class_definition" => "CLASS",
            "method_declaration" | "method_definition" | "constructor_declaration" => "METHOD",
            "function_definition" | "function_declaration" => "FUNCTION",
            _ => return,
        };
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let mut parent = node.parent();
        let mut container = String::new();
        let mut nested = false;
        while let Some(p) = parent {
            if matches!(p.kind(), "class_declaration" | "class_definition") {
                if !container.is_empty() {
                    nested = true;
                }
                container = p
                    .child_by_field_name("name")
                    .map(|n| text(n, source).to_owned())
                    .unwrap_or_default();
            }
            if matches!(
                p.kind(),
                "method_declaration"
                    | "method_definition"
                    | "function_definition"
                    | "function_declaration"
                    | "arrow_function"
                    | "function_expression"
            ) {
                nested = true;
            }
            parent = p.parent();
        }
        if nested || kind == "CLASS" && !container.is_empty() {
            return;
        }
        let body = node.child_by_field_name("body");
        if body.is_none() {
            return;
        }
        let header = &source[node.start_byte()..body.unwrap().start_byte()];
        let signature: String = String::from_utf8_lossy(header)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let kind = if kind == "FUNCTION" && !container.is_empty() {
            "METHOD"
        } else {
            kind
        };
        let name = text(name, source).to_owned();
        let supported = !signature
            .split_whitespace()
            .any(|word| ["get", "set"].contains(&word))
            && !node.parent().is_some_and(|parent| {
                parent.kind() == "decorated_definition" && {
                    let decorators = &source[parent.start_byte()..node.start_byte()];
                    let decorators = String::from_utf8_lossy(decorators);
                    decorators
                        .lines()
                        .any(|line| line.trim().starts_with('@') && line.trim() != "@staticmethod")
                }
            });
        let class_context = if kind == "CLASS" {
            let mut context = header.to_vec();
            if let Some(body) = body {
                let mut cursor = body.walk();
                for member in body.named_children(&mut cursor) {
                    if !matches!(
                        member.kind(),
                        "method_declaration"
                            | "method_definition"
                            | "constructor_declaration"
                            | "function_definition"
                    ) {
                        context.extend_from_slice(&source[member.byte_range()]);
                    }
                }
            }
            context
        } else {
            vec![]
        };
        symbols.push(Symbol {
            public: match profile {
                Profile::Java => signature.split_whitespace().any(|w| w == "public"),
                Profile::Python => !name.starts_with('_'),
                _ => {
                    signature.contains("export")
                        || node
                            .parent()
                            .is_some_and(|n| n.kind() == "export_statement")
                }
            },
            static_: signature.split_whitespace().any(|w| w == "static")
                || node.parent().is_some_and(|p| {
                    p.kind() == "decorated_definition" && text(p, source).contains("@staticmethod")
                }),
            name,
            container,
            kind,
            signature,
            start: node.start_byte(),
            end: node.end_byte(),
            line: node.start_position().row + 1,
            bytes: source[node.byte_range()].to_vec(),
            class_context,
            supported,
        });
    });
    Some(symbols)
}

#[cfg(test)]
mod tests {
    use super::{Profile, code_tokens, neutral};

    #[test]
    fn comments_are_ignored_but_literals_and_docstrings_are_preserved() {
        for (profile, old, new) in [
            (
                Profile::Java,
                "class A {} // before",
                "class A {} // authorize()",
            ),
            (
                Profile::JavaScript,
                "const x = 1; // before",
                "const x = 1; // authorize()",
            ),
            (
                Profile::TypeScript,
                "const x: number = 1; // before",
                "const x: number = 1; // authorize()",
            ),
            (Profile::Python, "x = 1 # before\n", "x = 1 # authorize()\n"),
        ] {
            assert!(
                neutral(profile, old.as_bytes(), new.as_bytes()),
                "{profile:?}"
            );
            assert!(
                !code_tokens(profile, new.as_bytes())
                    .unwrap()
                    .iter()
                    .any(|(t, _, _)| t == "authorize")
            );
        }
        assert!(!neutral(
            Profile::Python,
            b"def f():\n    \"\"\"old\"\"\"\n    return 1\n",
            b"def f():\n    \"\"\"new\"\"\"\n    return 1\n"
        ));
        assert!(!neutral(
            Profile::Java,
            b"class A { String x = \"a\"; }",
            b"class A { String x = \"b\"; }"
        ));
    }

    #[test]
    fn interpolations_are_code_but_template_text_and_regex_are_not() {
        for (profile, source) in [
            (
                Profile::JavaScript,
                "const x = `authorize ${authorize(user)}`;",
            ),
            (
                Profile::TypeScript,
                "const x: string = `authorize ${authorize(user)}`;",
            ),
            (Profile::Python, "x = f'authorize {authorize(user)}'\n"),
        ] {
            let tokens = code_tokens(profile, source.as_bytes()).unwrap();
            assert_eq!(
                tokens.iter().filter(|(t, _, _)| t == "authorize").count(),
                1,
                "{profile:?}: {tokens:?}"
            );
        }
        let tokens = code_tokens(
            Profile::JavaScript,
            b"const pattern = /authorize(user)/; const text = `authorize(user)`;",
        )
        .unwrap();
        assert!(!tokens.iter().any(|(t, _, _)| t == "authorize"));
        let nested = code_tokens(
            Profile::JavaScript,
            b"const x = `${`inner ${authorize(user)}`}`;",
        )
        .unwrap();
        assert_eq!(
            nested.iter().filter(|(t, _, _)| t == "authorize").count(),
            1
        );
    }

    #[test]
    fn invalid_syntax_and_missing_tokens_never_establish_neutrality() {
        for (profile, source) in [
            (Profile::Java, "class A {"),
            (Profile::JavaScript, "const x = ;"),
            (Profile::TypeScript, "const x: = ;"),
            (Profile::Python, "def f(:\n"),
        ] {
            assert!(profile.parse(source.as_bytes()).is_err());
            assert!(code_tokens(profile, source.as_bytes()).is_none());
            assert!(!neutral(profile, source.as_bytes(), source.as_bytes()));
        }
        assert!(Profile::JavaScript.parse(b"//\xff").is_err());
        assert!(Profile::Java.parse(b"class A {} // \\u0061").is_err());
    }

    #[test]
    fn significant_layout_and_token_boundaries_are_not_neutral() {
        assert!(!neutral(
            Profile::Python,
            b"if ok:\n    x = 1\ny = 2\n",
            b"if ok:\n    x = 1\n    y = 2\n"
        ));
        for profile in [Profile::JavaScript, Profile::TypeScript] {
            assert!(!neutral(
                profile,
                b"function f() { return 1; }",
                b"function f() { return\n1; }"
            ));
        }
        assert!(!neutral(
            Profile::Java,
            b"class A { int x = 1 + 2; }",
            b"class A { int x = 1 - 2; }"
        ));
        assert!(neutral(
            Profile::Java,
            b"class A { int x=1; }",
            b"class A {\n int x = 1;\n}"
        ));
    }

    #[test]
    fn all_embedded_grammars_parse_classes_and_calls() {
        let cases: &[(Profile, &[u8])] = &[
            (Profile::Java, b"class Guard { boolean authorize() { return true; } }"),
            (Profile::JavaScript, b"class Guard { authorize() { return true; } } new Guard().authorize();"),
            (Profile::TypeScript, b"class Guard { authorize(): boolean { return true; } } new Guard().authorize();"),
            (Profile::Python, b"class Guard:\n    def authorize(self):\n        return True\nGuard().authorize()\n"),
        ];
        for &(profile, source) in cases {
            assert!(profile.parse(source).is_ok(), "{profile:?}");
        }
    }
}
