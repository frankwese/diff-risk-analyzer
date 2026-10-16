# Implementation Plan: Lokale Risikobewertung

**Branch**: `main` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)
**Input**: Migrated feature 002; bestehende Implementierung fortführen.

## Summary

Kern-CLI mit 14 Heuristiken, read-only Git-Diff, konkreten Gründen und vollständigem
Fehlervertrag. Regeln und Klassifikation bleiben zusammen; keine eigene neue Crate.
003 besitzt zusätzliche Symbolnutzung; 004 besitzt Zielartefakt-/Runtime-Freigaben.

## Technical Context

**Language/Version**: Rust 1.99.0, Edition 2024, Cargo.lock versioniert.
**Primary Dependencies**: Bestehende Pins aus Cargo.toml: tree-sitter 0.27.1 mit Java
0.23.5/JS 0.25.0/TS 0.23.2/Python 0.25.0 für Neutralitäts-/Kontextabgrenzung; similar
3.2.0, serde 1.0.229/serde_json 1.0.154/serde_yaml_ng 0.10.0;
libc 0.2.190 für Unix-Pipeflags/Prozessgruppen; Git >=2.49;
tempfile 3.27.0 nur Entwicklungstests.
**Storage**: Rohblobs/Regionen im Speicher, keine persistente Anwendungsablage.
**Testing**: tests/rules.rs, tests/git.rs, Kernverträge in tests/cli.rs; Rust-Testharness.
**Target Platform**: Entwicklungsziel macOS arm64; Zielreleaseprüfung bei 004.
**Project Type**: Bestehende CLI-Crate; keine neue Service-/Pluginarchitektur.
**Performance Goals**: Bisheriges internes 100-Dateien/10.000-Zeilen-Ziel <=5s und
<=256 MiB dokumentieren; kein neues Produktkriterium.
**Constraints**: 1.000 Dateien, 2 MiB je Blob, 32 MiB kumulativ, 100.000 geänderte Zeilen,
60s Gesamtlaufzeit; keine Shell/Fetch/Codeausführung oder Teil-Risk.
**Scale/Scope**: 14 Regeln; endlicher Text-/Pfad-/Vertragsumfang, kein Semantikbeweis.

## Constitution Check

Vor und nach Dokumentendesign PASS: Anforderungen/Zuordnung in feature-split.md;
eine bestehende Crate; Referenzpaare und Regressionen; explizite unsupported/Limit-
Fehler; deterministische Befunde. Keine Abweichung. Alte Testnachweise nicht neu ausgeführt.

## Project Structure

### Documentation (this feature)

spec.md, plan.md, research.md, data-model.md, quickstart.md, tasks.md, status.md,
checklists/requirements.md, contracts/cli.md und contracts/rules.md.

### Source Code (repository root)

Cargo.toml/lock, src/main.rs, src/lib.rs, src/model.rs, src/git.rs, src/rules.rs,
src/structured.rs, src/syntax.rs, src/cli.rs; tests/git.rs, tests/rules.rs,
tests/cli.rs, tests/fixtures/rules/. Gemeinsame Syntax wird von 003 mitgenutzt.

**Structure Decision**: Bestehende Module verwenden; keine Datei-/Codeverschiebung für
die Spec-Aufteilung. Gemeinsame Module sequenziell bearbeiten und andere Featureverträge erhalten.

## Execution and completion

Altimplementierung prüfen statt neu erzeugen. Raw-Framing/Timeout/Limits und Klassifikation
vervollständigen; jede relevante Region anhand rules.md bewerten, inklusive gemischter
Dateien und unbekannter Nachbaroperationen. Danach eigener FR-/SC-Abgleich mit lokalem
Kern-End-to-End-Nachweis. 003-Zustände oder vier Releaseziele sind keine zusätzlichen Gates.
Bestehende Erweiterung wird jedoch nicht entfernt oder künstlich deaktiviert.

## Complexity Tracking

Keine Constitution-Verletzung. Syntaxparser dienen der Abgrenzung/Neutralität;
Risikosignale bleiben endliche Heuristiken. Kein erneutes Stack-/Dependency-Research nötig.
