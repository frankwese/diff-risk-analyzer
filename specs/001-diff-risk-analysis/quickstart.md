# Quickstart and Validation Guide

Dieser Guide beschreibt die implementierte lokale CLI. Ausgeführte Prüfungen und noch
offene Vertrags-/Releaseprüfungen stehen in [validation.md](validation.md).

## Prerequisites and setup

Nutzer: macOS/Linux, passendes natives Executable für x86_64/arm64 und Git >=2.49.
Keine Python-, Java- oder Rust-Installation nötig; Parser und Regeln sind eingebunden.
Download kann Internet benötigen; Analyse läuft danach ausschließlich lokal.

Entwickler/Buildmaschinen: Rust >=1.90 mit Cargo, passende C-Toolchain und Linker für
Tree-sitter; Linux-Releasebuilds benötigen musl-Toolchain für ihr jeweiliges Ziel.
Lokale native Rust-/Cargo-Toolchain 1.99.0 für `aarch64-apple-darwin` geprüft.
Projekt-Toolchain ist auf 1.99.0 gepinnt; alle vier Parser bauen und bestehen direkte Proben.

Aus Projektroot nach Implementierung:

```sh
cargo build --release --locked
./target/release/risk-analyzer --help
./target/release/risk-analyzer --version
```

`--version` nennt Anwendungsversion und Regelstand; `--help` beschreibt BASE..TARGET.
Keine Installation von Code aus dem zu analysierenden Fremdrepository.

## Run checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Regeltests: 14 positive/negative Paare plus Grenzfälle aus [rules.md](contracts/rules.md).
Testzuordnung: Matrix aus [test-association.md](contracts/test-association.md).
CLI/Git-Tests erstellen und entfernen selbst temporäre Repositorys; lokale Projektdateien
werden nicht zum Zweck der Validierung geändert. Git-Konfiguration nur innerhalb Testrepos.

## End-to-end scenarios

In einem separaten lokalen Repository mit zwei bekannten Commits aufrufen; CLI vorher
installieren und über absoluten Installationspfad starten:

```sh
/ABSOLUTER/INSTALLATIONSPFAD/risk-analyzer HEAD~1..HEAD
```

| Vorbereitung in Testrepository | Erwartetes Ergebnis |
| --- | --- |
| Nur README geändert | Risk LOW, DOC_CHANGE, Exit 0 |
| package.json dependency version geändert | Risk MEDIUM, DEPENDENCY_UPDATE mit Pfad, Exit 0 |
| src/auth/Guard.java ausführbare authorize-Methode geändert | Risk HIGH, AUTH_CHANGE; fehlende geänderte Tests separat |
| migrations/V2.sql DROP TABLE ergänzt | Risk HIGH, DESTRUCTIVE_MIGRATION, keine Testabdeckungsbehauptung |
| Passender GuardTest-Test geändert, Guard direkt konstruiert und authorize aufgerufen | Gleiches HIGH; USED mit Testfall/Fundstelle |
| Passender Testname, aber nur andere Methode verwendet | MISSING für authorize |
| Test B geändert, nur unveränderter Test A nutzt authorize | MISSING; Test A zählt nicht |
| Fixture/Helper liefert Receiver | UNKNOWN mit Ursache, Risikoanalyse weiterhin erfolgreich |
| Identische Revisionen | LOW und No changes; Exit 0 |
| Unbekannte Revision/außerhalb Repository | stdout leer, Fehler auf stderr, Exit 2 |
| Binäränderung oder unbekannte Region neben HIGH | stdout leer, UNSUPPORTED_CHANGE, Exit 1 |

Exakte Format-/Sortierungsregeln stehen in [cli.md](contracts/cli.md).

## Offline and read-only validation

Erfolgsfälle nach Installation ohne Netzwerk erneut ausführen. Vorher/nachher HEAD,
Branch, Index, Worktree-Status und Repositorydateien vergleichen: unverändert.
External-Diff-/Textconv-/fsmonitor-Helfer im temporären Testrepository als Markerprobes
konfigurieren; Analyse darf keine Probe ausführen. Partial-Clone-Objekt fehlt: Fehler,
kein Lazy-Fetch. Jeden Referenzdiff zehnmal ausführen und stdout byteweise vergleichen.

## Packaging and limits

Release-Executable pro Ziel bauen: `aarch64-apple-darwin`, `x86_64-apple-darwin`,
`aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl`. Beispielsweise mit passender
nativer/musl-Buildumgebung `cargo build --release --locked --target TARGET`.
Cargo-Abhängigkeiten vor Build beziehen; Analyse benötigt sie nicht zur Laufzeit.
Executable auf sauberes Zielsystem mit Git kopieren; alle Referenzfälle offline ohne
Rust, Python, JVM oder externe Grammatikbibliotheken ausführen. Dynamische Verknüpfung
prüfen: Linux statisch; macOS nur OS-Systembibliotheken. Jede der vier Grammatiken
mit direktem Testnutzungsfall prüfen. Ziele erst nach erfolgreichem Test als geprüft nennen.
Grenzfallfixtures für Limits aus cli.md müssen ohne Teil-Risk abbrechen. Repräsentativen
100-Dateien-/10.000-Zeilen-Fall einmal hinsichtlich Zeit/RSS messen und Maschine nennen.
