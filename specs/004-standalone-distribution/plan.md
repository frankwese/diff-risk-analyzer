# Implementation Plan: Standalone-Auslieferung

**Branch**: `main` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)
**Input**: Migrierte Vier-Ziel-Auslieferung aus Gesamt-Spec 001.

## Summary

Bestehende Rust-Crate je Ziel bauen und prüfen. 002-Kernrelease und vollständiges
Produkt mit 003 klar unterscheiden; alte Buildnachweise nicht als aktuelle Freigaben nutzen.

## Technical Context

**Language/Version**: Rust/Cargo 1.99.0; bestehender Releaseprofile/Lockfile.
**Primary Dependencies**: Cargo-Abhängigkeiten der Funktionsfeatures; C-Compiler für
gebündelte Grammatiken, macOS-Linker oder Linux-musl-Toolchain. Keine Nutzer-Sprachlaufzeit.
**Storage**: Native Artefakte und dokumentierte Release-Matrix; keine Anwendungspersistenz.
**Testing**: Zielruntimefälle aus 002, für Vollprodukt zusätzlich 003; Linker-/Architekturprüfung.
**Target Platform**: aarch64/x86_64-apple-darwin; aarch64/x86_64-unknown-linux-musl.
**Project Type**: Auslieferung desselben CLI-Binaries, keine neue Crate.
**Performance Goals**: Vergleichbare Referenzabläufe; kein neues Score-/Benchmarkprodukt.
**Constraints**: Linux statisch, macOS nur OS-Systembibliotheken; offline mit Git >=2.49.
**Scale/Scope**: Vier Targets, kein Store/Autoupdate/CI-Produktfeature.

## Constitution Check

Vor/nach Design PASS: vier Ziele vollständig erhalten; bestehende Cargo-Struktur;
Runtimeprüfung statt bloß Cross-Build; explizite Blocker; Quellstand-/Versionsnachweise.
Keine wiederholten Containerläufe vor Abschluss relevanter Funktionsänderungen.

## Project Structure

### Documentation (this feature)

spec/plan/research/data-model/quickstart/tasks/status.md, checklists/requirements.md,
contracts/release.md. Release-Matrix in status.md, Nachweise dort ergänzen.

### Source Code (repository root)

Cargo.toml/lock, rust-toolchain.toml, README.md; target/<triple>/release/risk-analyzer
als ignorierte Buildausgabe. Bestehende tests/cli.rs/Git-/Regel-/Zuordnungsfixtures nutzen.

**Structure Decision**: Nach Funktionsabschluss aktuelle Zielartefakte bauen, ohne
Anwendungslogik für Packaging umzubauen. Zusätzliche CI-/Containerdateien nur bei konkretem Bedarf.

## Execution and completion

002 muss für Kernrelease abgeschlossen sein; 003 für vollständige Produktfreigabe.
Version/Commit/Target dokumentieren; Build, Verknüpfung und reale Laufzeit getrennt.
Intel-macOS benötigt geeigneten Runner; Docker-Emulation als Emulation kennzeichnen.
Saubere Laufzeitumgebung enthält nur OS/Git, kein Rust/Python/JVM oder externe Parser.

## Complexity Tracking

Keine Abweichung; vier Zielumgebungen sind durch FR-001 vorgeschrieben. Fehlender Runner
blockiert dessen Freigabe, nicht den funktionalen Abschluss von 002 oder 003.
