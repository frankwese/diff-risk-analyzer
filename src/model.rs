use std::{
    fmt,
    path::PathBuf,
    time::{Duration, Instant},
};

pub const RULES_VERSION: &str = "1.0.0";
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
