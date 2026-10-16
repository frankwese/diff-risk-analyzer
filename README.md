# Git Diff Risk Analyzer

Native Rust CLI for local deterministic Git diff risk analysis. No LLM or network API.
Specifications: [Spec-Kit feature overview](specs/README.md).

```sh
./target/release/risk-analyzer HEAD~1..HEAD
```

Output contains `Risk: LOW|MEDIUM|HIGH`, reasons with file/line evidence, separate test
usage states and the rule version. The highest signal wins; test changes never reduce risk.
Only committed endpoints are compared, never uncommitted/staged changes.

## Rules and test usage

The 14 signal categories include documentation, exact renames, tests, visual styling,
business logic, API extensions, dependencies, queries, concurrency/cache, authorization,
destructive migrations, payments, secret/IAM/production infrastructure and contract breaks.
The finite patterns and examples are in [rules.md](specs/002-local-risk-analysis/contracts/rules.md).
Unknown changes fail rather than producing an optimistic LOW classification.

Java/JavaScript/TypeScript/Python syntax is parsed without executing repository code.
Changed test cases must actually use a changed symbol; a matching filename is insufficient.
Direct imports and directly constructed local receivers are supported. Tests are matched
individually: changing test B does not activate unchanged usage in test A.
`USED` is syntactic evidence, not proof of coverage or working assertions.
`MISSING`, `UNKNOWN` and `NOT_APPLICABLE` distinguish absent, ambiguous and deleted symbols.
Factories, aliases, fixtures, indirect calls and overloads are handled conservatively.

Exit 0 means successful analysis (including HIGH). Exit 2 means invalid invocation/revision;
exit 1 means processing/unsupported input. Errors have empty stdout and messages on stderr.
Limits: 1,000 files, 2 MiB/blob, 32 MiB total blobs, 100,000 changed lines, 60 seconds.
No checkout, fetch, external diff, textconv, hooks or application test execution.

## Development

Requires Rust/Cargo 1.99.0, Git >=2.49 and a C compiler/linker for the embedded Tree-sitter
grammars. The toolchain is pinned in `rust-toolchain.toml` and dependencies in `Cargo.lock`.

```sh
cargo check --locked
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
./target/release/risk-analyzer --help
```

In IntelliJ Ultimate, install the JetBrains Rust plugin, open `Cargo.toml` and choose
**Attach Cargo Project** if not detected automatically. Toolchain: `~/.cargo/bin`.
Use the Cargo tool window to build/test, or create a Cargo run configuration for the binary.

## Distribution targets

Planned binaries: `aarch64-apple-darwin`, `x86_64-apple-darwin`,
`aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl`.
End users need only the target-specific executable, OS system libraries and Git.
No Rust, Python or JVM installation and no external grammar files will be required.
Each target needs its own native/musl build environment and runtime validation.

Copy the matching release executable to a directory on PATH and run
`risk-analyzer --version`. Parser libraries are embedded. See the actual completed and
pending checks in the feature status reports:
[core](specs/002-local-risk-analysis/status.md),
[test association](specs/003-test-association/status.md),
[distribution](specs/004-standalone-distribution/status.md).
