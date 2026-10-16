---
description: "Migrierte Aufgaben für die lokale Risikobewertung"
---

# Tasks: Lokale Risikobewertung

**Input**: spec.md, plan.md, research.md, data-model.md, contracts/.
**Tests**: Referenz-/Regressionstests gemäß Constitution und SC-001–005.
**Progress**: Marker aus 001 übernommen, keine neue Codeprüfung. Alt-ID pro Aufgabe.

**Aktuelle Umsetzung**: T004 abgeschlossen: gemeinsame Modell-/Revisionsvalidierung
und fehlende Dateiseiten gezielt geprüft. T005 abgeschlossen: Raw-/Blob-Framing, OID-/Pfadvalidierung,
Git-Pfadauflösung, Fehlerklassifikation sowie nicht blockierendes Pipe-I/O und
Prozessgruppen-Cleanup geprüft; siehe status.md. T006 ist ebenfalls abgeschlossen:
kumulative Blob-/Zeilenzählung und gemeinsame Git-/Delta-/CLI-Deadline geprüft.

## Format and Path Conventions

`- [ ] Tnnn [P?] [USn] Beschreibung mit Dateipfad`; IDs lokal zu 002.
Bestehende src/ und tests/ verwenden; keine zweite Crate. Gemeinsame Moduländerungen sequenziell.

## Phase 1: Setup

- [X] T001 Übernimm Cargo-Crate/Toolchain 1.99.0 und target-Ignore in `Cargo.toml`, `rust-toolchain.toml`, `src/main.rs`, `src/lib.rs`, `.gitignore` (alt T001).
- [X] T002 Übernimm Crate-Pins/Lockfile und vier Parser-ABI-Proben in `Cargo.toml`, `Cargo.lock`, `src/syntax.rs` (alt T002).
- [X] T003 Übernimm dokumentierte Build-/IntelliJ-Grundlagen in `README.md`; Releasevoraussetzungen besitzt 004 (alt T003).

## Phase 2: Foundational

- [X] T004 Vervollständige gemeinsame Kernmodelle in `src/model.rs`: `Option<T>`, `Vec<u8>`, `Result<T, AnalysisError>`, absoluter root/volle SHA-1/SHA-256-OIDs; „beide Endpunkte nicht leer, keine Option/Revisionliste/Dreipunktnotation“ und „keine Symlinkverfolgung/Checkout“ gemäß data-model.md nachweisen (alt T004).
- [X] T005 Prüfe und korrigiere Raw-NUL-/Batch-Framing, Pfadvalidierung, read-only Prozessoptionen und Cleanup in `src/git.rs`; kein Shell-/Fetch-/Hook-/Textconv-Aufruf gemäß contracts/cli.md (alt T005).
- [X] T006 Schließe Delta-/Budgetgrenzen in `src/git.rs` ab: „1.000 Dateien, 2 MiB je Blob, 32 MiB kumulativ, 100.000 geänderte Zeilen, 60s gesamte Analyse“; Zeitlimit nur Fehler, kein approximativer Erfolg (alt T006).
- [X] T007 Vervollständige Neutralitäts-/Kontextabgrenzung in `src/syntax.rs`: ERROR/MISSING sperren positive Neutralität, Python-Einrückung/Docstrings und JS-Zeilengrenzen erhalten; Testnutzung selbst besitzt 003 (alt T007).
- [X] T008 Übernimm Aufrufparser/help/version und stdout/stderr-/Exitbasis in `src/cli.rs`, `src/main.rs` (alt T008).
- [X] T009 Belege vollständige Basisfehler-/Raw-/Budget-/Dirty-Worktree-Matrix in `tests/git.rs`, `tests/cli.rs`, `src/syntax.rs`; vorhandene Tests gezielt um fehlende Vertragsfälle ergänzen (alt T009).

## Phase 3: US1 — Begründete Risikobewertung

**Goal**: Alle 14 Signale und konkrete stabile Risk/Reasons.
**Independent Test**: Eigene Regelpaare und lokale LOW/MEDIUM/HIGH-CLI-Fälle, keine 003-/004-Gates.

- [X] T010 [US1] Übernimm 14 positive/negative Referenzpaare und Rename-/Scopegrenzen in `tests/fixtures/rules/`, `tests/rules.rs` (alt T010).
- [X] T011 [US1] Vervollständige Kernformat-/BASE/TARGET/METADATA-/Deduplizierungsfälle in `tests/cli.rs`, zehn gleiche Ausgaben ohne Testzuordnungs-Gate (alt T011).
- [X] T012 [US1] Übernimm Fixture-/HIGH-plus-unknown-/additive-Migration-Prüfungen in `tests/rules.rs` (alt T012).
- [X] T013 [US1] Schließe Klassifikation/Gemischtscopes und Segment-/Identifiergrenzen in `src/rules.rs` anhand contracts/rules.md ab (alt T013).
- [X] T014 [US1] Vervollständige DOC/RENAME/TEST/STYLE-Regeln in `src/rules.rs`; neutral nur im erkannten Umfang, CSS jede Property/Selektorregion prüfen (alt T014).
- [X] T015 [US1] Vervollständige BUSINESS/DEPENDENCY/QUERY/CACHE in `src/rules.rs`, Manifest-Unterbereiche und mehrzeilige SQL-Operationen ohne unbekannte Nachbarregionen verdecken (alt T015).
- [X] T016 [US1] Schließe AUTH/DESTRUCTIVE/PAYMENT/SECRET-IAM-PROD-Regeln in `src/rules.rs` ab; tatsächliche Operationen/Pfade von Kommentaren/Strings trennen, statische Begründungen ohne Werte (alt T016).
- [X] T017 [US1] Prüfe API_EXTENSION/CONTRACT_BREAK und begrenzte JSON/YAML/OpenAPI-/Signaturvergleiche in `src/rules.rs`, `src/structured.rs`; doppelte Schlüssel/Tags/Referenzen und Rekursion konservativ behandeln (alt T017).
- [X] T018 [US1] Vervollständige Regionsaccounting/Aggregation in `src/rules.rs`: keine Fremdregion durch HIGH abgedeckt; „Stufe absteigend, rule_id, rohe Pfadbytes, Seite, Zeile“, vollständige Fundstellen deduplizieren (alt T018).
- [X] T019 [US1] Schließe Kernintegration/gepufferte Ausgabe in `src/lib.rs`, `src/cli.rs` ab; bestehende 003-Erweiterung kompatibel erhalten, aber kein Testnutzungsabschluss als Kern-Gate (alt T019).
- [X] T020 [US1] Übernimm bereits belegte Kern-E2E-/Wiederholungsprüfungen aus `tests/rules.rs`, `tests/git.rs`, `tests/cli.rs` in `status.md`; spätere neue Regeländerungen gezielt validieren (alt T020).

## Phase 4: US2 — Fehler, Grenzen und Read-only

**Goal**: Kein falsches LOW/Teil-Risk, keine Repositorymutation.
**Independent Test**: Fehler-/Limit-/Helferfälle ohne 003/004-Abschluss.

- [X] T021 [P] [US2] Schließe fehlende Read-only-/Objekt-/Symlink-/Gitlink-/LFS-/Limitmatrix in `tests/git.rs` ab; konkrete Lücken vor erneutem Lauf benennen (alt T030).
- [X] T022 [P] [US2] Schließe Exit/stdout/stderr-/Redaktions-/fehlendes-Git-Fälle in `tests/cli.rs` ab; Release-Smokes separat in 004 (Kernanteil alt T031).
- [X] T023 [US2] Korrigiere verbleibende unsupported/Deadline-/Cleanup-Pfade in `src/git.rs`, `src/rules.rs`, `src/cli.rs` bis T021/T022 vollständig erfüllt sind (alt T032).

## Phase 5: Polish

## Bestätigte Korrektur: Auth-Kontext

- [X] T026 [US1] Ergänze funktionsbezogenen AUTH_CONTEXT_CHANGE in `src/rules.rs` und Regressionen in `tests/rules.rs`: tatsächliche Prüfaufrufe/Funktionsnamen im selben Basis-/Zielsymbol, HIGH bei Logik-/Vertragsänderung, kein Kontextübertrag aus anderen Funktionen, Kommentaren/Literalen; README und status.md berichtigen und FR-011/SC-006 nachweisen.

- [X] T024 Finalisiere Kernbeispiele/-Grenzen in `README.md`, `quickstart.md` und dokumentiere Rest-/Abschlussstatus in `status.md` (Kernanteil alt T035).
- [X] T025 Prüfe 002-FR-001–010/SC-001–005 in `status.md`; nach letzten Funktionsänderungen einmal fmt/Clippy/Kernprüfungen und repräsentativen Zeit/RSS-Nachweis ausführen, keine Vierzielprüfung als Gate (Kernanteil alt T036).

## Dependencies & Execution Order

Setup → Foundation → US1/US2 → Polish. Vorhandene Teilimplementierung wiederverwenden;
konkrete Lücke und fehlenden Testnachweis statt Neuschreiben auswählen. US2-Tests T021/T022
in getrennten Dateien nach Basis parallel vorbereitbar; Auswertung derselben Module sequenziell.
US1-Kernnutzen vor vollständiger Testzuordnung abnehmen; alle Kernfehler bleiben Pflicht.

## Implementation Strategy

MVP zuerst 002 abschließen. 003 kann danach eigene Funktionsmatrix abschließen;
004 baut aktuelles Release am Funktionsmeilenstein. Keine Tests/Docker bei Dokumentenmigration.
25/25 Aufgaben abgeschlossen. Aktuelle Umsetzung, FR-/SC-Matrix, Prüfungen und
konservativer endlicher Regelumfang stehen in status.md. Keine Freigabe von 003/004.
