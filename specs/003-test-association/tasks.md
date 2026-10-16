---
description: "Migrierte Aufgaben für symbolbezogene Testzuordnung"
---

# Tasks: Zugehörige Teständerungen

**Input**: spec/plan/research/data-model.md und contracts/.
**Tests**: Verbindliche eigene vierprofilige Matrix. Marker nur aus 001 übernehmen.

## Format and Path Conventions

Lokale T-IDs; bestehende src/syntax.rs, association.rs und CLI-Module; keine zweite Crate.

## Phase 1: Setup

- [X] T001 Übernimm vorhandene vier Parser-/Crate-Basis in `Cargo.toml`, `src/syntax.rs`; Kern-Git/Modelle besitzt 002 (aus erledigtem alt T002).

## Phase 2: Foundational

002-Kerndaten/CLI vorausgesetzt. Gemeinsame Module nicht parallel ändern; eigene Bindungs-
fixtures können vor Kernabschluss vorbereitet werden. Keine neue Setup-Implementierung nötig.

## Phase 3: US1 — Direkte Nutzungszuordnung

**Goal**: Geänderte Klassen/Methoden zu tatsächlich geänderten Tests mit Belegen zuordnen.
**Independent Test**: Direkte Fälle je Profil inklusive Alias/static/function, ohne Release-Gate.

- [ ] T002 [US1] Vervollständige vierprofilige direkte/negative Nutzungsfixtures in `tests/fixtures/association/`, `tests/association.rs` gemäß Vertrag; bestehende Fälle nicht neu generieren (alt T021).
- [ ] T003 [US1] Schließe geänderte Einzeltest-/Fixturematrix in `tests/association.rs` ab; unveränderte Aufrufzeile im geänderten Fall zählt, andere Tests/Setup/Import/Zeilenverschiebung nicht (alt T022).
- [ ] T004 [US1] Vervollständige Symbol-/Testfallbereiche in `src/syntax.rs`; keine alleinige Namensidentität bei Overloads, kleinste Methode, Klassenkopf/-feld und gelöschte Symbole unterscheiden (alt T023).
- [ ] T005 [US1] Schließe Matching nach Sprache/Container/Name und eigenen Basis-/Zielbytes in `src/association.rs` ab; Kopf/Annotation/Decorator einschließen, Mehrdeutigkeit UNKNOWN (alt T024).
- [ ] T006 [US1] Schließe eindeutige Importherkunft in `src/association.rs` ab: Java-FQN/Modulroot, direkte JS/TS ESM und Python-Module; kein Wildcard/Barrel/Runtimeimport oder Dateinamenbeweis (alt T025).
- [ ] T007 [US1] Vervollständige lokale Konstruktor-/Receiver-/static/function-Bindung in `src/association.rs`; keine Schatten-/Alias-/Factory-/Helper-/Vererbungsbeweise; falsche gleichnamige Klasse nicht USED (alt T026).

## Phase 4: US2 — Grenzen und Status

**Goal**: USED/MISSING/UNKNOWN/NOT_APPLICABLE ehrlich und getrennt von Risk ausgeben.
**Independent Test**: Negativ-/Lösch-/Mehrdeutigkeitsmatrix und unverändertes Kernrisiko.

- [X] T008 [US2] Übernimm Status-/UsageEvidence-Implementierung in `src/association.rs`, `src/model.rs`: gültiger Beleg USED, relevanter unklarer Kandidat UNKNOWN, ohne Nutzung MISSING, gelöschtes Ziel NOT_APPLICABLE (alt T027).
- [X] T009 [US2] Übernimm Tests-CLI-Erweiterung/Fundstellen in `src/lib.rs`, `src/cli.rs`, `tests/cli.rs`; UNKNOWN allein kein Fehler und keine Risikoabsenkung (alt T028).
- [ ] T010 [US2] Schließe komplette eigene Matrix und Integration in `tests/association.rs`, `tests/cli.rs` ab; SC-001–004 inklusive zehn identischer Testhinweise in `status.md` dokumentieren (alt T029).

## Phase 5: Polish

- [ ] T011 Dokumentiere unterstützte Nutzung/Erkennungsgrenzen und Zustände in `README.md`, `quickstart.md`, `status.md` (Anteil alt T035).
- [ ] T012 Prüfe alle 003-FR-001–009/SC-001–004 und Kernkompatibilität in `status.md`; nach Änderungen einmal erforderliche finale Funktionsprüfungen, keine Vierzielbuilds als Gate (Anteil alt T036).

## Dependencies & Execution Order

002-Basis → Fixtures/Symbolmatching → Herkunft/Bindung → Statusintegration → Abschluss.
T002-Fixtures und T004-Syntax nach Basis in getrennten Dateien parallel vorbereitbar;
T005–T009 schreiben gemeinsame Module sequenziell. Beispiel: T003 tests/association.rs
parallel zu T004 src/syntax.rs, nachdem die Fixtures existieren.

## Implementation Strategy

Bestehende Implementierung weiterführen; pro negativer Vertragslücke fokussiert korrigieren.
Eigene Nutzungs-/Statusmatrix abnehmen, ohne Releases zu wiederholen. 12 Aufgaben,
3 erledigte Marker übernommen. Rest teils umgesetzt, noch nicht vollständig nachgewiesen.
