# Implementation Plan: Zugehörige Teständerungen

**Branch**: `main` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)
**Input**: Migrierte Testzuordnung aus Gesamt-Spec 001, bestehende Module fortführen.

## Summary

003 erweitert 002 um geänderte Symbole/Testfälle und echte syntaktische Nutzung,
ohne Risikoabsenkung oder Coveragebehauptung. Eigene Fixturematrix und CLI-Erweiterung.

## Technical Context

**Language/Version**: Rust 1.99.0, Edition 2024.
**Primary Dependencies**: Bereits gebündelte Tree-sitter-Pins aus Cargo.toml für vier
Sprachen; Basis-Modelle und Git-Blobs von 002. Keine neuen Abhängigkeiten.
**Storage**: Symbol-/Testfall-/Bindungsdaten im Speicher.
**Testing**: tests/association.rs, tests/fixtures/association/, Erweiterungsfälle in tests/cli.rs.
**Target Platform**: Entwicklungsziel macOS arm64; vier Zielreleases bei 004.
**Project Type**: Module einer bestehenden CLI-Crate.
**Performance Goals**: Keine eigene neue Zielzahl; gesamte Budgetgrenzen aus 002 einhalten.
**Constraints**: Keine Typ-/Vererbungs-/Callgraph-Vollanalyse oder Repositoryimporte.
**Scale/Scope**: Vier endliche Profile, direkte Imports/Receiver, UNKNOWN für unaufgelöste Nutzung.

## Constitution Check

Vor/nach Design PASS: Nutzerforderung echte Nutzung erhalten; bestehende Module nutzen;
positive/negative Fixturematrix und Statusbelege; UNKNOWN explizit statt falschem USED;
deterministische Ausgabe. Keine neue Erfolgsbehauptung zur Implementierung.

## Project Structure

### Documentation (this feature)

spec/plan/research/data-model/quickstart/tasks/status.md, checklists/requirements.md,
contracts/test-association.md und contracts/cli-extension.md.

### Source Code (repository root)

src/association.rs, src/syntax.rs, gemeinsame src/model.rs/cli.rs/lib.rs;
tests/association.rs, tests/fixtures/association/, tests/cli.rs. 002 besitzt die
Risiko-/Gitbasis; gemeinsame Änderungen nicht parallel im selben Modul.

**Structure Decision**: Eigene Arbeitseinheit, gemeinsame Crate. Contract-/Nutzungsprofile
übernommen; Parserfehler sperren USED. Geänderte Einzeltests über eigene Alt-/Neu-Bytes matchen.

## Execution and completion

Import-/Receiver-/Testfallmatrix abschließen; Namensgleichheit nicht als Bindung betrachten.
Eindeutige direkte Nutzung auch ohne Testdateinamenskonvention; gemischte Scopes erhalten.
USED/MISSING/UNKNOWN/NOT_APPLICABLE mit Fundstellen integrieren; Risk/Exit unverändert.
Featureabschluss anhand eigener Matrix plus Integration gegen 002, ohne Zielrelease-Gate.

## Complexity Tracking

Keine Abweichung; bestehende Parser ersetzen eigene Lexer. Keine zweite Regex-Nutzungs-
implementierung und kein Aufruf analysierter Laufzeitumgebungen.
