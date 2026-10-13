# Implementation Plan: Git Diff Risk Analyzer MVP

**Branch**: `main` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-diff-risk-analysis/spec.md`

## Summary

Lokale CLI `risk-analyzer BASE..TARGET` mit 14 deterministischen Risikosignalen,
höchster Signalstufe als Gesamtrisiko und getrennten symbolbezogenen Testhinweisen.
Rust steuert Git ausschließlich lesend an. Rohdaten und vollständige Revision-Blobs
werden ohne externe Diff-/Textfilter gelesen. Ein fester Pfad-/Textkatalog bewertet
Risiken; vier eingebundene Tree-sitter-Grammatiken liefern Syntax für begrenzte
Testnutzungszuordnung. Kein LLM, Netzwerk, Repository-Codeimport oder Testlauf der
analysierten Anwendung. Unbekannte Risikoänderung bedeutet Fehler; unklare Testnutzung
bleibt ein Hinweis. Ausführbare Regelspezifikation: [contracts/rules.md](contracts/rules.md).

## Technical Context

**Language/Version**: Rust stable >=1.90, Edition 2024; Cargo. Build-Toolchain beim
Implementierungsstart über `rust-toolchain.toml` auf eine konkrete stabile Version pinnen.

**Primary Dependencies**: Git >=2.49 mit `--no-lazy-fetch`; Rust-Crates
`tree-sitter = =0.27.1`, `tree-sitter-java = =0.23.5`,
`tree-sitter-javascript = =0.25.0`, `tree-sitter-typescript = =0.23.2`,
`tree-sitter-python = =0.25.0`; `similar = =3.2.0` für Zeilendeltas,
`serde_json = =1.0.154`, `serde_yaml_ng = =0.10.0` für strukturierte Daten.
`tempfile = =3.27.0` nur als Testabhängigkeit. CLI über `std::env::args_os`,
Git über `std::process::Command`; kein zusätzliches CLI-Framework.
Cargo.lock wird versioniert; C-Compiler/Linker nur auf Build-Maschinen für Parser benötigt.

**Storage**: Keine persistente Anwendungsablage; Revision-Blobs, Befunde und Syntaxbäume
im Speicher. Git-Integrationstests verwenden temporäre Repositorys.

**Testing**: Rust `#[test]`/`cargo test --locked`; Regel-/Bindungsfixtures,
CLI-Vertragstests und echte Git-Repositorytests. `cargo fmt` und Clippy als Qualitätsprüfungen.

**Target Platform**: macOS und Linux, x86_64/arm64; Nutzer benötigen nur passendes
Executable, OS-Systembibliotheken und Git. Je Zielarchitektur eigener Build: Darwin
native, Linux musl statisch einschließlich Parser. Keine Rust-/Python-/Java-Installation
auf Nutzersystemen. macOS arm64 ist die lokale Umgebung, noch kein geprüfter Rust-Build.

**Project Type**: Native Standalone-CLI, eine Cargo-Crate mit Binary `risk-analyzer`;
keine öffentliche Bibliotheks-API. Parser und Regeldaten sind im Executable enthalten.

**Performance Goals**: Entwicklungsziel <=5 Sekunden für 100 Dateien/10.000 geänderte
Zeilen und <=256 MiB Spitzen-RSS auf dokumentierter Referenzmaschine; kein neues
Produkt-Akzeptanzkriterium. Ein repräsentativer Smoke-Benchmark im Abschluss reicht.

**Constraints**: Offline zur Laufzeit; keine Git-Filter/Hooks/Builds ausführen.
Feste Schutzgrenzen: 1.000 Dateien, 2 MiB pro Blob, 32 MiB kumulative gelesene Blobdaten,
100.000 geänderte Zeilen; 60 Sekunden Gesamtlaufzeit. Überschreitung: Fehler ohne Risk.
Keine Mutation des analysierten Repositorys. Schutzgrenzen sind versionierter CLI-Vertrag.

**Scale/Scope**: 14 Regeln; allgemeinere Dateiklassifikation und vier Syntaxprofile für
Testzuordnung. Kein vollständiger Callgraph, Typchecker, Regelplugin oder Konfigurationssystem.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Prinzip | Vor Recherche | Nach Design |
| --- | --- | --- |
| Nachvollziehbare Anforderungen | PASS: spec.md und 16 geprüfte Checklistenpunkte | PASS: FR-001–016 auf CLI-, Regel-, Bindungs- und Auslieferungsverträge abgebildet |
| Kleinste vollständige Lösung | PASS: leeres src, keine bestehende Implementierung | PASS: eine Rust-Crate; Parser ersetzen komplexe eigene Teilparser; keine Service-/Plugin-Schichten |
| Überprüfbare Korrektheit | PASS: Akzeptanzfälle vorhanden | PASS: 14 positive/negative Regelpaare, Bindungsmatrix und Git-Vertragstests geplant |
| Explizite Eingaben und Fehler | PASS: Fehler statt falscher LOW-Bewertung | PASS: OID-Validierung, Größenlimits, Secrets ohne Quelltextausgabe, Status getrennt |
| Nachvollziehbare Ergebnisse | PASS: feste Regeln, stabile Reihenfolge | PASS: Regelstand, Datei/Seite/Zeile, deterministische Diff-/Sortierpolitik |

Keine ungerechtfertigten Abweichungen. Plan endet nach Design; Umsetzung erst über Aufgaben.

## Project Structure

### Documentation (this feature)

```text
specs/001-diff-risk-analysis/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/requirements.md
└── contracts/
    ├── cli.md
    ├── rules.md
    └── test-association.md
```

`tasks.md` wird erst durch `/speckit.tasks` erzeugt.

### Source Code (repository root)

Geplante Struktur, noch nicht implementiert:

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
README.md
src/
├── main.rs
├── lib.rs
├── cli.rs
├── git.rs
├── model.rs
├── rules.rs
├── association.rs
└── syntax.rs
tests/
├── rules.rs
├── association.rs
├── git.rs
├── cli.rs
└── fixtures/
    ├── rules/
    └── association/
```

**Structure Decision**: Das vorhandene leere `src/` wird genutzt. Module entsprechen
konkreten Verantwortungen; keine austauschbaren Parser-/Repository-Frameworks.
IDE-Java-Metadaten sind keine Produktvorgabe. README beschreibt Installation und Grenzen.

## Design Execution

1. CLI validiert genau einen Zwei-Revisionsvergleich und löst beide Endpunkte in OIDs auf.
2. Git liefert NUL-getrennte Raw-Änderungen und rohe Blobs. Inhaltsvergleich über
   `similar` mit explizitem `Algorithm::Myers` auf vollständigen Zeilen; Limits vor Vergleich.
   Zeitlimit darf nur abbrechen, nicht einen zeitabhängigen unvollständigen Diff liefern.
3. Risikoanalyse klassifiziert Pfade und geänderte Inhalte anhand Regelkatalog 1.0.0.
   Jeder substanzielle Änderungsbereich muss ein unterstütztes Bewertungsmuster besitzen;
   ein HIGH-Treffer macht andere unbekannte Bereiche nicht automatisch bewertbar.
4. Syntaxanalyse der geänderten Quell-/Testdateien bestimmt Symbol- und Testfallbereiche.
   Die Risikoregeln sind keine AST-/Compiler-Vollanalyse und verlangen kein Sprachprofil.
5. Testfälle werden anhand eigener Alt-/Neu-Inhalte abgeglichen. Direkte Nutzung plus
   eindeutige lokale Bindung bestätigt ein Symbol; indirekte Nutzung bleibt unklar.
6. Gesamtergebnis erst nach vollständiger Risikoanalyse ausgeben; Gründe und Testhinweise
   stabil sortieren, Quelltext und Geheimniswerte nie ausgeben.

## Verification Strategy

- FR-001/002/011/012: Integrationstests in temporären Repositorys; invalid refs,
  Dirty Worktree, fehlende Blobs, ungewöhnliche Pfade, Gitlink/Binärdatei und Limits.
- FR-003–007/010: alle Referenzpaare aus rules.md, gemischte Stufen, unbekannter Bereich
  neben HIGH, zehn Wiederholungen mit byteidentischer Ausgabe.
- FR-008/009/014/015: Testfall-/Receiver-Matrix aus test-association.md in allen vier Profilen.
- FR-004/013: exakte stdout/stderr-/Exit-Verträge, escaped Pfade und Secret-Redaktion.
- FR-016/Packaging: `cargo build --release --locked`, je Ziel Binary separat testen;
  CLI außerhalb Quellbaum und in Repository-Unterverzeichnis. Laufzeittest offline auf
  sauberem Zielsystem ohne Python, JVM oder Rust; eingebettete vier Grammatiken prüfen.

## Complexity Tracking

Keine Constitution-Verletzungen. Tree-sitter mit vier Grammatiken ist durch präzise
Testfallgrenzen, Kommentar-/String-Abgrenzung und Klassen-/Methodennutzung begründet.
serde_yaml_ng ersetzt einen eigenen fehleranfälligen YAML-Teilparser. Ein eigener Parser
für vier Sprachen wäre mehr Code und schlechter überprüfbar.
