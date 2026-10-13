# Implementation validation — 2026-10-13

**Tagesabschluss**: Implementierung auf Nutzerwunsch pausiert.
Übergabe und konkrete Restarbeit: [implementation-status.md](implementation-status.md).

## Verified locally

- Native Rust/Cargo 1.99.0, aarch64-apple-darwin; Git 2.54.0.
- Cargo.lock and exact dependencies, four compiled-in grammar probes successful.
- Testsuite aktuell: 45 Rust test functions, including data-driven signal pairs
  (26 cases plus exact-rename positive/negative) and receiver-boundary fixtures.
- Last full local test run: 41 functions passed. Later additions passed targeted runs
  (latest association run: 20; rules run before that: 13). No final complete rerun today.
- `cargo clippy --locked --all-targets -- -D warnings`: passed before latest additions.
- `cargo build --release --locked`: passed before latest additions. `otool -L` lists
  only libSystem.B.dylib. Releasebinary may lag the current source state.
- CLI E2E: real two-commit repository, HIGH authorization change and USED JavaScript
  method evidence; ten repeated invocations byte-identical; invalid ref exit 2,
  unsupported binary next to HIGH exit 1 and empty stdout, no literal secret in stderr.
- Direct test usage in all four languages; changed test B does not activate test A;
  constructor-only, false method-name/string evidence, factories, aliases, branches,
  reassignment and nested lambda boundaries; partial two-method obligations.
- Read-only Git test preserves dirty file; textconv/external diff/fsmonitor marker helpers
  do not run; newline/tab paths and exact rename; oversized blob and binary input reject.
- macOS filesystem rejects non-UTF-8 filenames; raw non-UTF-8 metadata and escaping are
  covered by a synthetic parser test rather than a macOS file-creation claim.

## Scope and remaining verification

Implementation is not yet declared fully converged. Current tests exercise reference
cases, not every construct promised by the design. Outstanding:

- Direct imported function calls and per-language missing/unchanged-test cases are tested;
  the remaining negative matrix per language still needs expansion.
  Nonstandard test filenames, distinct suite containers, classmethod/custom decorators,
  deleted methods, invalid source and local/callback symbol shadowing now have tests.
- Changed Python class fields, multiple CSS selectors, SQL comments/quoted keywords,
  manifest subsection boundaries and mixed source/test scope now have targeted tests.
  Multiline SQL and same-line unrelated operation rejection now have targeted fixtures.
  Manifest edge syntax and exhaustive region accounting
  still need broader fixtures and implementation refinement.
- CLI enforces a 60-second overall wait; Git subprocess budget ends within that window.
  Parser nesting is bounded to 128. Cancellation/cleanup stress tests remain open.
- Missing local blob with promisor remote configured, Gitlinks/symlinks/LFS and mode-only
  changes have rejection tests. Full timeout cleanup and all resource boundaries
  still need stress coverage.
- File-count and changed-line limits have boundary tests; full cumulative-byte and
  cancellation stress coverage remains open.
- Direct helper calls without resolvable target usage are UNKNOWN rather than a false
  definitive MISSING; a corresponding regression fixture passes.
- New required API fields produce CONTRACT_BREAK instead of a purely compatible
  extension; malformed JSON/YAML infrastructure is rejected before producing a HIGH result.
- macOS-x86_64 release cross-build successful; runtime invocation fails with `bad CPU type`
  on this host, so target runtime checks remain blocked.
- Linux-arm64 musl: Docker rust:1.99-alpine + Git 2.54, tests and release build passed.
  ldd reports no valid dynamic program, confirming static binary.
- Linux-x86_64 musl: Docker --platform linux/amd64, tests and optimized build passed
  under emulation for the earlier source revision. Latest source changes not retested
  in containers; container runs paused and the running container stopped.
- Clean-system runtime certification remains open. A 100-file/10,000-changed-line
  documentation fixture completes in 4.87s in a debug-build E2E test. Full benchmark
  RSS remains open. Empty-diff native smoke: 0.38s, max RSS 5,029,888 bytes.

## Acceptance mapping

FR-001/003/004/010/011: local CLI/Git/sorting tests pass.
FR-002: read-only/helper prevention tested; full offline/missing-object matrix pending.
FR-005/006: all 14 categories have positive/negative references; broader scope validation pending.
FR-007/009/013: static reasons, separate tests, immutable risk and user documentation present.
FR-008/014/015: core four-language direct usage and selected negative boundaries pass;
full contract matrix remains open.
FR-012: unsupported/binary/high-plus-unknown tests pass; exhaustive region accounting pending.
FR-016/SC-008: native macOS-arm64 binary builds; full four-target runtime certification pending.

SC-001–004: local examples/determinism tested. SC-005–007 partially tested; SC-008 open.

No commits or pushes were performed. Checklist markers were not changed.
