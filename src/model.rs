use std::{
    fmt,
    path::PathBuf,
    time::{Duration, Instant},
};

pub const RULES_VERSION: &str = "1.1.0";
pub const MAX_BLOB: usize = 2 * 1024 * 1024;
pub const MAX_TOTAL: usize = 32 * 1024 * 1024;
pub const MAX_FILES: usize = 1000;
pub const MAX_LINES: usize = 100_000;

#[derive(Debug, Clone)]
pub struct AnalysisError {
    pub code: &'static str,
    pub message: &'static str,
    pub exit: i32,
    pub paths: Vec<Vec<u8>>,
}
impl AnalysisError {
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message,
            exit: 1,
            paths: vec![],
        }
    }
    pub fn input(message: &'static str) -> Self {
        Self {
            code: "INVALID_INPUT",
            message,
            exit: 2,
            paths: vec![],
        }
    }
    pub fn unsupported(path: &[u8]) -> Self {
        Self {
            code: "UNSUPPORTED_CHANGE",
            message: "change is outside the supported rule scope",
            exit: 1,
            paths: vec![path.to_vec()],
        }
    }
}
impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {} — {}", self.code, self.message)?;
        for path in &self.paths {
            write!(f, "\n- {}", escape(path))?;
        }
        Ok(())
    }
}
impl std::error::Error for AnalysisError {}

pub fn escape(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let valid = match std::str::from_utf8(remaining) {
            Ok(text) => {
                append_escaped(&mut out, text);
                break;
            }
            Err(error) => error.valid_up_to(),
        };
        append_escaped(
            &mut out,
            std::str::from_utf8(&remaining[..valid]).expect("valid prefix"),
        );
        out.push_str(&format!("\\x{:02x}", remaining[valid]));
        remaining = &remaining[valid + 1..];
    }
    out
}
fn append_escaped(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Low,
    Medium,
    High,
}
impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Base,
    Target,
    Metadata,
}
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Base => "base",
            Self::Target => "target",
            Self::Metadata => "metadata",
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub level: Level,
    pub reason: &'static str,
    pub path: Vec<u8>,
    pub side: Side,
    pub line: Option<usize>,
}
#[derive(Debug, Clone)]
pub struct ChangedLine {
    pub side: Side,
    pub number: usize,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct FileChange {
    pub old_path: Option<Vec<u8>>,
    pub new_path: Option<Vec<u8>>,
    pub old_mode: String,
    pub new_mode: String,
    pub old_oid: String,
    pub new_oid: String,
    pub status: String,
    pub old: Vec<u8>,
    pub new: Vec<u8>,
    pub lines: Vec<ChangedLine>,
}
impl FileChange {
    pub fn path(&self) -> &[u8] {
        self.new_path
            .as_deref()
            .or(self.old_path.as_deref())
            .unwrap_or_default()
    }
    pub fn text(&self, side: Side) -> &[u8] {
        if side == Side::Base {
            &self.old
        } else {
            &self.new
        }
    }
    pub fn exact_rename(&self) -> bool {
        self.status == "R100" && self.old_oid == self.new_oid && self.old_mode == self.new_mode
    }
}
#[derive(Debug)]
pub struct Comparison {
    pub root: PathBuf,
    pub base: String,
    pub target: String,
    pub files: Vec<FileChange>,
}

impl Comparison {
    pub(crate) fn new(
        root: PathBuf,
        base: String,
        target: String,
        files: Vec<FileChange>,
    ) -> Result<Self, AnalysisError> {
        if !root.is_absolute()
            || !valid_oid(&base)
            || !valid_oid(&target)
            || base.len() != target.len()
        {
            return Err(AnalysisError::new(
                "GIT_FAILED",
                "invalid resolved comparison metadata",
            ));
        }
        Ok(Self {
            root,
            base,
            target,
            files,
        })
    }
}

pub(crate) fn valid_oid(oid: &str) -> bool {
    [40, 64].contains(&oid.len())
        && oid
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub(crate) fn validate_revision_endpoint(endpoint: &str) -> Result<(), AnalysisError> {
    if endpoint.is_empty()
        || endpoint.starts_with(['-', '^'])
        || endpoint.contains("..")
        || endpoint
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
        || endpoint.contains(':')
        || endpoint.contains("{/")
        || endpoint.ends_with("^!")
        || endpoint.ends_with("^@")
    {
        Err(AnalysisError::input(
            "expected a single non-empty local revision",
        ))
    } else {
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Budget {
    deadline: Instant,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            deadline: Instant::now() + Duration::from_secs(60),
        }
    }
}
impl Budget {
    pub(crate) fn deadline(&self) -> Instant {
        self.deadline
    }
    pub(crate) fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
    #[cfg(test)]
    pub(crate) fn for_duration(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
    pub fn check(&self) -> Result<(), AnalysisError> {
        if Instant::now() >= self.deadline {
            Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "analysis exceeded its time limit",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
pub(crate) struct InputTotals {
    blob_bytes: usize,
    changed_lines: usize,
}

impl InputTotals {
    pub(crate) fn reserve_blob(&mut self, size: usize) -> Result<(), AnalysisError> {
        let total = self
            .blob_bytes
            .checked_add(size)
            .filter(|total| *total <= MAX_TOTAL);
        if size > MAX_BLOB || total.is_none() {
            return Err(AnalysisError::new(
                "LIMIT_EXCEEDED",
                "blob exceeds size or total input limit",
            ));
        }
        self.blob_bytes = total.unwrap();
        Ok(())
    }

    pub(crate) fn remaining_lines(&self) -> usize {
        MAX_LINES - self.changed_lines
    }

    pub(crate) fn add_lines(&mut self, count: usize) -> Result<(), AnalysisError> {
        self.changed_lines = self
            .changed_lines
            .checked_add(count)
            .filter(|count| *count <= MAX_LINES)
            .ok_or_else(|| {
                AnalysisError::new("LIMIT_EXCEEDED", "diff exceeds 100000 changed lines")
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    #[test]
    fn blob_budget_counts_both_sides_and_all_files_with_inclusive_limits() {
        let mut totals = InputTotals::default();
        for _ in 0..8 {
            totals.reserve_blob(MAX_BLOB).unwrap();
            totals.reserve_blob(MAX_BLOB).unwrap();
        }
        totals.reserve_blob(0).unwrap();
        assert_eq!(totals.reserve_blob(1).unwrap_err().code, "LIMIT_EXCEEDED");
        assert!(InputTotals::default().reserve_blob(MAX_BLOB + 1).is_err());
        assert!(InputTotals::default().reserve_blob(usize::MAX).is_err());
    }

    #[test]
    fn changed_lines_are_cumulative_and_count_insertions_plus_deletions() {
        let mut totals = InputTotals::default();
        totals.add_lines(60_000).unwrap();
        assert_eq!(totals.remaining_lines(), 40_000);
        assert!(totals.add_lines(60_000).is_err());
        let mut totals = InputTotals::default();
        totals.add_lines(MAX_LINES).unwrap();
        totals.add_lines(0).unwrap();
        assert!(totals.add_lines(1).is_err());
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn resolved_comparison_requires_absolute_root_and_same_full_oid_format() {
        for width in [40, 64] {
            let comparison = Comparison::new(
                PathBuf::from("/repo"),
                "a".repeat(width),
                "b".repeat(width),
                vec![],
            )
            .unwrap();
            assert!(comparison.root.is_absolute());
            assert_eq!(comparison.base.len(), width);
        }
        for (root, base, target) in [
            ("relative", "a".repeat(40), "b".repeat(40)),
            ("/repo", "HEAD".into(), "b".repeat(40)),
            ("/repo", "a".repeat(40), "b".repeat(64)),
            ("/repo", "g".repeat(40), "b".repeat(40)),
        ] {
            assert!(Comparison::new(root.into(), base, target, vec![]).is_err());
        }
    }

    #[test]
    fn revision_endpoints_are_single_revisions_not_sets_or_options() {
        for endpoint in ["HEAD", "HEAD~1", "main", "v1.2", "HEAD^2", "HEAD^{commit}"] {
            validate_revision_endpoint(endpoint).unwrap();
        }
        for endpoint in [
            "",
            "--all",
            "^HEAD",
            "main..HEAD",
            "main...HEAD",
            "HEAD^!",
            "HEAD^@",
            "HEAD:path",
            "main HEAD",
            "HEAD\n",
            "HEAD^{/secret}",
        ] {
            assert_eq!(
                validate_revision_endpoint(endpoint).unwrap_err().exit,
                2,
                "{endpoint:?}"
            );
        }
    }

    #[test]
    fn finding_metadata_and_error_paths_have_no_implicit_source_content() {
        let finding = Finding {
            rule: "DOC_CHANGE",
            level: Level::Low,
            reason: "Documentation changed",
            path: b"a\n\xff.md".to_vec(),
            side: Side::Metadata,
            line: None,
        };
        assert_eq!(finding.line, None);
        assert_eq!(escape(&finding.path), "a\\n\\xff.md");
        let error = AnalysisError::unsupported(&finding.path);
        assert_eq!(error.exit, 1);
        assert!(error.to_string().contains("a\\n\\xff.md"));
        assert_eq!(AnalysisError::input("invalid revision").exit, 2);
    }
}
