# Kernanalyse: Validierung und Aufruf

Rust/Cargo 1.99.0 und C-Compiler beim Build, Git >=2.49 zur Analyse.
Kernumfang 002 lokal abgeschlossen; aktuelle Nachweise und Grenzen in status.md.

```sh
cargo test --locked --test rules --test git --test cli
cargo run --locked -- HEAD~1..HEAD
```

Referenzen aus contracts/rules.md: DOC LOW, Dependency MEDIUM, Auth/destruktive Migration
HIGH, gemischte höchste Stufe und alle Gründe. Zehn Wiederholungen identisch.
Leerer Vergleich LOW/No changes; ungültige Revision Exit 2; unbekannte Nachbarregion
neben HIGH Exit 1/stdout leer. Dirty Worktree/Git-Helfer dürfen unverändert/unbenutzt bleiben.
Fixtures nur temporäre Repositorys, keine Projektcommits zu Testzwecken.

Abschluss 002: eigene Referenz-/Fehler-/Read-only-/Limitkriterien nachweisen. 003-/004-
Restarbeit nicht als Kern-Gate behandeln, bereits integrierte Erweiterung nicht beschädigen.
Fortschritt [status.md](status.md), Aufgaben [tasks.md](tasks.md).

Die Analyse kann unsupported liefern, wenn eine Anweisung außerhalb des festen
Regelkatalogs liegt. Ein unbekannter Funktionsaufruf wird nicht automatisch MEDIUM.
Keine Garantie semantischer Neutralität oder allgemeiner Vertragskompatibilität.
