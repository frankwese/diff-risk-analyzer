# Standalone: Release-Validierung

Erst 002-Funktionsmeilenstein, für Vollprodukt zusätzlich 003 abschließen. Build benötigt
Rust 1.99.0/Cargo und passenden C-/musl-Linker; Nutzer nur passendes Executable/OS/Git.

```sh
cargo build --release --locked --target aarch64-apple-darwin
```

Analog x86_64-apple-darwin, aarch64-unknown-linux-musl und x86_64-unknown-linux-musl in
geeigneten Zielumgebungen. `otool -L` auf macOS nur OS-Systembibliotheken; Linux statisch.
Artefakt in saubere Umgebung ohne Rust/Python/JVM/externen Parser kopieren und offline
help/version/002-Referenzen ausführen; bei Vollprodukt zusätzlich alle vier 003-Profilfälle.
Commit/Version/Target/Verknüpfung/Runtime-Nachweis in status.md dokumentieren.

Docker-Emulation ist als Emulation zu kennzeichnen; Nutzer brauchen kein Docker.
Fehlender Intel-macOS-Runner blockiert dessen Freigabe, nicht 002/003-Funktionsabschluss.
Historische frühere Builds nicht für aktuelle Quellen als geprüft ausgeben.
