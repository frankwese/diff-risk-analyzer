use risk_analyzer::{git::Git, model::Budget};
use std::{fs, process::Command};

fn git(root: &std::path::Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn reads_committed_blobs_without_worktree_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Fixture"]);
    git(root, &["config", "user.email", "fixture@example.invalid"]);
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    fs::write(root.join("README.md"), "after\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "target"]);
    fs::write(root.join("README.md"), "dirty\n").unwrap();
    let reader = Git::open(root, Budget::default()).unwrap();
    let result = reader.compare("HEAD~1", "HEAD").unwrap();
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0].old, b"before\n");
    assert_eq!(result.files[0].new, b"after\n");
    assert_eq!(fs::read(root.join("README.md")).unwrap(), b"dirty\n");
    assert!(reader.compare("no-such-revision", "HEAD").is_err());
}

fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.name", "Fixture"]);
    git(
        dir.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    dir
}

#[test]
fn rename_and_raw_non_utf8_paths_are_preserved() {
    use std::os::unix::ffi::OsStrExt;
    let dir = repository();
    let root = dir.path();
    let path = std::ffi::OsStr::from_bytes(b"odd\n\t.md");
    fs::write(root.join(path), "same\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    fs::rename(root.join(path), root.join("new.md")).unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "target"]);
    let reader = Git::open(root, Budget::default()).unwrap();
    let result = reader.compare("HEAD~1", "HEAD").unwrap();
    assert!(result.files[0].exact_rename());
    assert_eq!(
        result.files[0].old_path.as_deref(),
        Some(b"odd\n\t.md".as_slice())
    );
}

#[test]
fn external_helpers_are_not_run_and_large_binary_inputs_fail() {
    use std::os::unix::fs::PermissionsExt;
    let dir = repository();
    let root = dir.path();
    let helper = root.join("helper.sh");
    fs::write(&helper, "#!/bin/sh\ntouch marker\nexit 1\n").unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-qm", "base"]);
    fs::write(root.join("README.md"), "after\n").unwrap();
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-qm", "target"]);
    git(root, &["config", "diff.external", helper.to_str().unwrap()]);
    git(
        root,
        &["config", "diff.custom.textconv", helper.to_str().unwrap()],
    );
    git(
        root,
        &["config", "core.fsmonitor", helper.to_str().unwrap()],
    );
    fs::write(root.join(".gitattributes"), "*.md diff=custom\n").unwrap();
    let reader = Git::open(root, Budget::default()).unwrap();
    reader.compare("HEAD~1", "HEAD").unwrap();
    assert!(!root.join("marker").exists());
    git(root, &["config", "core.fsmonitor", "false"]);
    fs::write(
        root.join("large.md"),
        vec![b'x'; risk_analyzer::model::MAX_BLOB + 1],
    )
    .unwrap();
    git(root, &["add", "large.md"]);
    git(root, &["commit", "-qm", "large"]);
    assert_eq!(
        reader.compare("HEAD~1", "HEAD").unwrap_err().code,
        "LIMIT_EXCEEDED"
    );
    fs::write(root.join("binary.md"), b"a\0b").unwrap();
    git(root, &["add", "binary.md"]);
    git(root, &["commit", "-qm", "binary"]);
    assert_eq!(
        reader.compare("HEAD~1", "HEAD").unwrap_err().code,
        "UNSUPPORTED_CHANGE"
    );
}

#[test]
fn raw_parser_rejects_truncation_and_preserves_non_utf8_metadata() {
    let old = "a".repeat(40);
    let new = "b".repeat(40);
    let mut bytes = format!(":100644 100644 {old} {new} M\0").into_bytes();
    bytes.extend_from_slice(b"a\xff.java\0");
    let file = risk_analyzer::git::parse_raw(&bytes).unwrap();
    assert_eq!(file[0].path(), b"a\xff.java");
    assert_eq!(risk_analyzer::model::escape(file[0].path()), "a\\xff.java");
    assert!(risk_analyzer::git::parse_raw(b":bad\0x\0").is_err());
}

#[test]
fn symlink_gitlink_lfs_and_mode_only_changes_fail_explicitly() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = repository();
    let root = dir.path();
    fs::write(root.join("README.md"), "base\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    symlink("README.md", root.join("link")).unwrap();
    git(root, &["add", "link"]);
    git(root, &["commit", "-qm", "link"]);
    let reader = Git::open(root, Budget::default()).unwrap();
    assert_eq!(
        reader.compare("HEAD~1", "HEAD").unwrap_err().code,
        "UNSUPPORTED_CHANGE"
    );
    fs::write(
        root.join("lfs"),
        "version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 1\n",
    )
    .unwrap();
    git(root, &["add", "lfs"]);
    git(root, &["commit", "-qm", "lfs"]);
    assert_eq!(
        reader.compare("HEAD~1", "HEAD").unwrap_err().code,
        "UNSUPPORTED_CHANGE"
    );
    fs::set_permissions(root.join("README.md"), fs::Permissions::from_mode(0o755)).unwrap();
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-qm", "mode"]);
    assert_eq!(
        reader.compare("HEAD~1", "HEAD").unwrap_err().code,
        "UNSUPPORTED_CHANGE"
    );
    let oid = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let oid = String::from_utf8(oid.stdout).unwrap();
    git(
        root,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{},submodule", oid.trim()),
        ],
    );
    git(root, &["commit", "-qm", "gitlink"]);
    assert_eq!(
        reader.compare("HEAD~1", "HEAD").unwrap_err().code,
        "UNSUPPORTED_CHANGE"
    );
}

#[test]
fn locally_missing_blob_cannot_trigger_remote_fetch() {
    let dir = repository();
    let root = dir.path();
    fs::write(root.join("README.md"), "before\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    fs::write(root.join("README.md"), "after\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "target"]);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD:README.md"])
        .output()
        .unwrap();
    let oid = String::from_utf8(output.stdout).unwrap();
    let oid = oid.trim();
    fs::remove_file(root.join(".git/objects").join(&oid[..2]).join(&oid[2..])).unwrap();
    git(root, &["config", "extensions.partialClone", "origin"]);
    git(root, &["config", "remote.origin.promisor", "true"]);
    git(
        root,
        &[
            "config",
            "remote.origin.url",
            "https://example.invalid/no-fetch",
        ],
    );
    let reader = Git::open(root, Budget::default()).unwrap();
    assert!(reader.compare("HEAD~1", "HEAD").is_err());
}

#[test]
fn file_and_changed_line_limits_fail_without_truncation() {
    let header = format!(
        ":100644 100644 {} {} M\0file.java\0",
        "a".repeat(40),
        "b".repeat(40)
    );
    assert_eq!(
        risk_analyzer::git::parse_raw(
            header
                .repeat(risk_analyzer::model::MAX_FILES + 1)
                .as_bytes()
        )
        .unwrap_err()
        .code,
        "LIMIT_EXCEEDED"
    );
    let old = String::new();
    let new = (0..100001)
        .map(|i| format!("new {i}\n"))
        .collect::<String>();
    assert_eq!(
        risk_analyzer::git::diff_lines(old.as_bytes(), new.as_bytes())
            .unwrap_err()
            .code,
        "LIMIT_EXCEEDED"
    );
}
