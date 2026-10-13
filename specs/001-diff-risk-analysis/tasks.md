---
description: "Ausführbare Aufgaben für die native Rust-CLI Git Diff Risk Analyzer"
---

# Tasks: Git Diff Risk Analyzer MVP

**Input**: Design documents from `specs/001-diff-risk-analysis/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md und contracts/.

**Tests**: Verbindlich gemäß Constitution III und SC-002–008; Regelbeispiele und
Testzuordnung sind ausdrücklich überprüfbare Anforderungen. Verhaltenstests zuerst
erstellen und ihren fehlenden Implementierungsstand nachweisen, dann implementieren.

**Organization**: Aufgaben nach Nutzerstory; gemeinsame Modelle/Git-/Syntaxbasis vorab.
Checkboxen erst nach Umsetzung und erforderlicher Prüfung abhaken. Kein automatisches
Committen oder Pushen durch die Aufgabenliste.

## Format: `[ID] [P?] [Story] Description`

- `[P]`: Nach den genannten Voraussetzungen parallel in getrennten Dateien bearbeitbar.
- `[US1]`, `[US2]`, `[US3]`: Zuordnung zu den Nutzerstorys in spec.md.
- Pfade sind relativ zur Repositorywurzel. Alle Fixtures enthalten harmlose Beispieldaten.

## Path Conventions

Eine Cargo-Crate: `Cargo.toml`, `src/*.rs`, `tests/*.rs`, `tests/fixtures/`.
CLI-Binary `risk-analyzer`; Bibliotheksmodule nur für interne Wiederverwendung/Tests.
Release-Validierungsprotokoll in `specs/001-diff-risk-analysis/validation.md`.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reproduzierbar baubares Rust-Grundgerüst und Parser-Kompatibilitätsprüfung.

- [X] T001 Erzeuge Cargo-Crate mit Binary `risk-analyzer`, Edition 2024 und Modulgerüst in `Cargo.toml`, `src/main.rs` und `src/lib.rs`; pinne Rust 1.99.0 mit rustfmt/Clippy in `rust-toolchain.toml` und ergänze `/target/` in `.gitignore`.
- [X] T002 Ergänze die exakten Crate-Pins aus plan.md in `Cargo.toml`, `tempfile` nur als dev-dependency; erzeuge und prüfe `Cargo.lock`, kompiliere alle vier Tree-sitter-Grammatiken und verifiziere LANGUAGE-/ABI-Kompatibilität mit minimalen Parse-Proben in `src/syntax.rs`.
- [X] T003 [P] Dokumentiere Build, Cargo-Attach in IntelliJ, Nutzer-/Buildvoraussetzungen und die vier Releaseziele in `README.md`; funktionierende native Toolchain nicht mit erfolgreichem Anwendungstest gleichsetzen.

**Checkpoint**: `cargo check --locked` und vier Parserproben erfolgreich; Cargo-Projekt
kann in IntelliJ angehängt werden. T003 kann nach T001 parallel zu T002 erfolgen.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Gemeinsame Eingaben, Rohdaten, Syntax und Fehlerpfade für alle Storys.

- [ ] T004 Implementiere gemeinsame structs/enums in `src/model.rs`: optionale Werte als `Option<T>`, Rohdaten als `Vec<u8>`, Ergebnisse als `Result<T, AnalysisError>`; `root` absolut, OIDs vollständig SHA-1/SHA-256; „beide Endpunkte nicht leer, keine Option/Revisionliste/Dreipunktnotation“; Pfade „niemals zur Symlinkverfolgung oder Checkout verwenden“; fehlende Blobseite leer und Zustände USED/MISSING/UNKNOWN/NOT_APPLICABLE gemäß data-model.md.
- [ ] T005 Implementiere begrenzten read-only Git-Prozesszugriff in `src/git.rs`: Git >=2.49, Argumentlisten ohne Shell, feste Schutzoptionen/Umgebung aus research.md, Deadline und begrenztes stdout/stderr-Lesen; Raw-NUL-Status inklusive R100 und Batch-Blobheader/exakte Bytezahl validieren; nie Checkout/Fetch/Filter/Hooks starten.
- [ ] T006 Implementiere reproduzierbare Alt-/Neu-Zeilenregionen mit `similar::Algorithm::Myers` in `src/git.rs`; Grenzen „1.000 Dateien, 2 MiB je Blob, 32 MiB insgesamt gelesene Blobdaten, 100.000 hinzugefügte/gelöschte Zeilen, 60 Sekunden insgesamt“ vor teuren Operationen durchsetzen; Deadline nie als approximativen Teildiff ausgeben.
- [ ] T007 Implementiere den vierprofiligen Syntaxzugriff mit Byte-/Zeilenbereichen, Kommentar-/Literalabgrenzung und ERROR/MISSING-Markern in `src/syntax.rs`; Python-Einrückung, Docstrings und JS-Zeilengrenzen erhalten; Java-Unicode-Escapes für Neutralität/Bindung konservativ sperren.
- [X] T008 Implementiere Aufrufparser und gemeinsame Fehlerausgabe in `src/cli.rs` und `src/main.rs`: genau ein BASE..TARGET, kein Dreipunkt/Pathspec, help/version ohne Repositoryzugriff; Exit 2 für Aufruf/Revision, Exit 1 für Verarbeitung, stdout bei Fehler leer; kein vollständiger Git-stderr-/Quelltextdurchgriff.
- [ ] T009 Prüfe die Basis in `tests/git.rs` mit temporären Repositorys: Root/Unterverzeichnis, neue/gelöschte Dateien, exakter Rename, ungewöhnliche Bytepfade, fehlende Objekte, Dirty Worktree und Limits; prüfe Syntaxprobe und Fehlerpfade zusätzlich in `src/syntax.rs` und `tests/cli.rs`.

**Checkpoint**: Gemeinsame Daten-/Git-/Syntaxbasis und Fehlerpfade geprüft; keine
erfolgreiche LOW-Ausgabe als Platzhalter für noch fehlende Bewertungsregeln.

## Phase 3: User Story 1 - Änderungen vor dem Push einschätzen (Priority: P1)

**Goal**: Alle 14 Signaltypen und nachvollziehbare Gesamtrisikoklasse für unterstützte Diffs.

**Independent Test**: Temporärer Zwei-Commit-Vergleich liefert für Dokumentation LOW,
Dependency-Update MEDIUM und Auth/destruktive Migration HIGH mit konkreten Fundstellen;
gemischte Stufen erhalten alle Gründe, Wiederholungen sind identisch.

### Tests for User Story 1

- [X] T010 [P] [US1] Erzeuge alle 14 positiven/negativen Referenzpaare aus contracts/rules.md in `tests/fixtures/rules/` mit Basis-/Zielinhalt, Pfad, erwarteten IDs und Klasse oder UNSUPPORTED_CHANGE; ergänze Scope-/Neutralitäts-/Rename-Grenzfälle ohne Secrets.
- [ ] T011 [P] [US1] Schreibe CLI-Vertragstests in `tests/cli.rs` für LOW/MEDIUM/HIGH, leeren Vergleich, konkrete BASE/TARGET/METADATA-Fundstellen, englisches LF-Format, deduplizierte/stabile Gründe und zehn byteidentische Wiederholungen gemäß contracts/cli.md.
- [X] T012 [US1] Erzeuge den Fixture-Runner und Aggregationstests in `tests/rules.rs` nach T010; teste unbekannte Region neben HIGH, Auth in Dokumentation/Tests, additive Migration und fehlende Fallbackbewertung ausdrücklich.

### Implementation for User Story 1

- [ ] T013 [US1] Implementiere DOC/TEST/SOURCE/STYLE/DEPENDENCY/MIGRATION/CONTRACT/INFRA-Klassifikation in `src/rules.rs`; feste Segment-/Basenamemuster, CamelCase-/underscore-Grenzen, DOC zuerst und gemischte Test-/Produktionsscopes getrennt behandeln.
- [ ] T014 [US1] Implementiere DOC_CHANGE, NEUTRAL_RENAME, TEST_CHANGE und VISUAL_STYLE in `src/rules.rs` mit Tokenneutralität, unverändertem Selektor/erlaubten CSS-Properties und kritischem Pfadrollenwechsel gemäß Regelvertrag; kein allgemeiner Semantikbeweis.
- [ ] T015 [US1] Implementiere BUSINESS_LOGIC, DEPENDENCY_UPDATE, QUERY_PERFORMANCE und CONCURRENCY_CACHE in `src/rules.rs`; Manifest-Unterbereiche/Lockfiles, SQL-Operationsgrenzen und unveränderten Kontext gemäß contracts/rules.md abgrenzen.
- [ ] T016 [US1] Implementiere AUTH_CHANGE, DESTRUCTIVE_MIGRATION, PAYMENT_CHANGE und SECRET_IAM_PROD in `src/rules.rs`; Dokumentation/Tests ausschließen, kritische Pfade und ausführbare Operationsindizien unterscheiden, nur statische Gründe ohne Werte erzeugen.
- [ ] T017 [US1] Implementiere API_EXTENSION und CONTRACT_BREAK in `src/rules.rs` mit begrenztem öffentlichem Signatur-/OpenAPI-Vergleich; JSON/YAML doppelte Schlüssel vor Map-Konvertierung ablehnen, Rekursion/Aliasexpansion begrenzen und externe $ref/allOf/oneOf nicht stillschweigend auswerten.
- [ ] T018 [US1] Implementiere vollständiges Region accounting und max-Level-Aggregation in `src/rules.rs`: jede substanzielle Region neutral/abgedeckt/unsupported, kein HIGH als Abdeckung fremder Regionen; „Sortierung: absteigende Risikostufe, rule_id, rohe Pfadbytes, Seite, Zeile“ und „Deduplizierung nach kompletter Fundstellenidentität, nicht bloß Regel-ID“ gemäß data-model.md.
- [ ] T019 [US1] Verbinde Git/Syntax/Regeln und vollständig gepufferte Erfolgsdarstellung in `src/lib.rs` und `src/cli.rs`; Risk/Reasons/Rules/Heuristikhinweis, escaped Pfade und leere Diff-Ausgabe gemäß Vertrag; Testzuordnung bis US2 ausdrücklich als noch nicht analysiert behandeln, nie USED/MISSING erfinden.
- [X] T020 [US1] Führe die Regel-/CLI-Tests und Git-End-to-End-Fälle in `tests/rules.rs`, `tests/cli.rs` und `tests/git.rs` aus; dokumentiere SC-001–004 samt zehn Wiederholungen in `specs/001-diff-risk-analysis/validation.md` und behebe Befund-/Formatabweichungen.

**Checkpoint**: Risikobewertung als interner Demo-Meilenstein unabhängig überprüfbar.
Die vollständige MVP-Ausgabe einschließlich Testzuordnung erst nach US2 freigeben.

## Phase 4: User Story 2 - Review-Schwerpunkte nachvollziehen (Priority: P2)

**Goal**: Pro geänderter Klasse/Methode direkte Nutzung durch einen geänderten Testfall
erkennen; fehlende/unklare Nutzung transparent ausweisen, Risikoklasse unverändert lassen.

**Independent Test**: Derselbe Produktionsdiff mit direkter Nutzung, nur passendem
Dateinamen, anderer Methode, unverändertem Test und indirektem Receiver liefert die
vereinbarten USED/MISSING/UNKNOWN-Hinweise; zwei geänderte Methoden getrennt behandeln.

### Tests for User Story 2

- [ ] T021 [P] [US2] Erzeuge vierprofilige Bindungsfixtures in `tests/fixtures/association/` für die gesamte Matrix aus contracts/test-association.md: direkte/statische Nutzung, Aliase, falsche gleichnamige Klasse, Scheinnennung, Overloads, Fixtures/Factory/Helper, Shadowing, Löschungen und Parserfehler.
- [ ] T022 [US2] Implementiere Fixture-Runner und Testfall-Änderungsprüfungen in `tests/association.rs` nach T021; unveränderte Nutzungszeile im geänderten Test zählt, Änderung in anderem Test/Setup/Import/Suite und reine Zeilenverschiebung zählen nicht; teilweise Zuordnung pro Methode prüfen.

### Implementation for User Story 2

- [ ] T023 [US2] Extrahiere geänderte Klassen/Methoden/Funktionen und Testfallbereiche aus den vier Syntaxprofilen in `src/syntax.rs`; „keine alleinige Namensidentität bei Overloads“, kleinste umgebende Methode statt zusätzlichem Klassenhinweis; Klassensignatur-/Feldänderungen und gelöschte Symbole gemäß data-model.md unterscheiden.
- [ ] T024 [US2] Implementiere eindeutiges Testfallmatching nach Sprache/Container/Name und Vergleich eigener Basis-/Zielbytes in `src/association.rs`; Kopf/Annotation/Decorator einschließen, mehrfachen Namen/unklare Grenzen UNKNOWN und gelöschte Tests nicht als Änderung im Ziel zählen.
- [ ] T025 [US2] Implementiere Importherkunft und Kandidatenauflösung in `src/association.rs`: Java-FQN/Einzelimport/Modulroot, JS/TS direkte relative ESM-Dateien, Python eindeutiger Root/src-Modulpfad; nur geänderte Produktionsdateien, keine Barrel-/Wildcard-/Runtime-Importauflösung oder bloße Dateinamenbindung.
- [ ] T026 [US2] Implementiere lokale direkte Konstruktor-/Receiver-/Static-/Function-Bindung in `src/association.rs`; gleiche Klasse und direkt deklarierte Methode erforderlich, keine Aliasfortpflanzung/Reassignment/Shadowing/Vererbung/Helperverfolgung; Assertionargumente zulässig, verschachtelte Lambdas/indirekte Nutzung UNKNOWN.
- [X] T027 [US2] Implementiere TestAssociation-Status und UsageEvidence in `src/association.rs` und `src/model.rs`: „valider positiver Befund → USED; andernfalls relevanter offener Kandidat → UNKNOWN; vollständig untersuchter unterstützter Umfang ohne Nutzung → MISSING“; gelöschtes Ziel NOT_APPLICABLE, kein Symbolname aus unbekannter Syntax erfinden.
- [X] T028 [US2] Integriere sortierte Testhinweise und USED-Fundstellen in `src/lib.rs` und `src/cli.rs`; unbekannte Bindung allein kein Fehler, keine Risikoabsenkung oder Coveragebehauptung; erweitere `tests/cli.rs` auf vollständigen Erfolgsvertrag.
- [ ] T029 [US2] Führe die gesamte Bindungs-/CLI-Matrix in `tests/association.rs` und `tests/cli.rs` aus und dokumentiere SC-006/007 in `specs/001-diff-risk-analysis/validation.md`; bestätige unveränderte Risk-Klasse bei variierenden Testzuständen.

**Checkpoint**: Risikoklasse plus vollständige ehrliche Testzuordnung auf lokalem Ziel.

## Phase 5: User Story 3 - Grenzen und Fehler erkennen (Priority: P2)

**Goal**: Fehler statt falscher Entwarnung; offline/read-only und native Auslieferung.

**Independent Test**: Ungültige Revisionen, unbekannte Änderungen und Limits liefern
leeres stdout/Fehlerstatus. Erfolgsfälle verändern keine Repositorydaten und starten
keine Helfer. Releasebinary läuft ohne Sprachlaufzeit samt allen vier Grammatiken.

### Tests for User Story 3

- [ ] T030 [P] [US3] Ergänze adversarielle Git-/Read-only-Fälle in `tests/git.rs`: Textconv/external diff/fsmonitor als harmlose Markerprobes, fehlende Partial-Clone-Objekte ohne Fetch, Symlink/Gitlink/LFS/Binärbytes, Source/Index/HEAD vor-nach unverändert und Ressourcenlimits.
- [ ] T031 [P] [US3] Ergänze Fehler-/Release-Smoke-Verträge in `tests/cli.rs`: stdout vollständig leer bei Exit 1/2, keine Quell-/Geheimniswerte in stderr, UNKNOWN-Testhinweis bei erfolgreichem Risk, fehlendes/ungeeignetes Git und vier eingebettete Parserprofile.

### Implementation for User Story 3

- [ ] T032 [US3] Schließe sämtliche Unsupported-/Limit-/Prozessfehlerpfade in `src/git.rs`, `src/rules.rs` und `src/cli.rs`; keine gefilterten Blobs/Symlinkverfolgung/Lazy-Fetch, Deadline abbrechen und Prozesse aufräumen, keine Teil-Risk-Ausgabe; T030/T031 vollständig erfüllen.
- [ ] T033 [US3] Baue native Releasebinary mit allen eingebetteten Grammatiken gemäß `Cargo.toml` und prüfe macOS-arm64-Ausführung außerhalb Quellbaum offline ohne Sprachlaufzeit; dokumentiere Verknüpfung und Parser-Smokes in `specs/001-diff-risk-analysis/validation.md`.
- [ ] T034 [US3] Baue und prüfe zusätzlich x86_64-apple-darwin, aarch64-unknown-linux-musl und x86_64-unknown-linux-musl gemäß `specs/001-diff-risk-analysis/quickstart.md`; benötigte native/musl-Umgebungen, statische Linux-Verknüpfung und sämtliche Referenzfälle je Ziel in `specs/001-diff-risk-analysis/validation.md` belegen, unzugängliche Ziele explizit offen lassen.

**Checkpoint**: SC-005 und Offline/Read-only lokal erfüllt; SC-008 erst nach belegter
Prüfung aller vier Releaseziele. Ein Cross-Build allein ist kein Zielsystem-Laufzeittest.

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Dokumentation und finaler Nachweis des vereinbarten Umfangs.

- [ ] T035 [P] Finalisiere Nutzer-/IntelliJ-/Builddokumentation in `README.md` und ausführbare Schritte in `specs/001-diff-risk-analysis/quickstart.md`; unterstützte Muster, Grenzen, Testzustände, Versions-/Exitvertrag, Git-Voraussetzung und Standaloneinstallation mit Beispielen erklären.
- [ ] T036 Führe `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` und `cargo build --release --locked` aus; einmaligen 100-Dateien-/10.000-Zeilen-Zeit/RSS-Smoke samt Maschine sowie FR-001–016/SC-001–008-Abgleich in `specs/001-diff-risk-analysis/validation.md` dokumentieren; blockierte Zielprüfungen nicht als Erfolg markieren.

## Dependencies & Execution Order

### Phase Dependencies

Setup T001 → T002; T003 parallel zu T002 nach T001. Foundation T004 → T005 → T006;
T007 und T008 folgen dem gemeinsamen Modell, T009 prüft die fertige Basis.
Storys erst nach T009. Finalprüfung erst nach lokalen Storyprüfungen und Releasearbeit.

### User Story Dependencies

```text
Setup → Foundation → US1 (T010–T020) → US2 (T021–T029)
                  └→ US3 Tests (T030/T031), Integration nach US1/US2
US1 + US2 → US3 Abschluss/Release (T032–T034) → Polish (T035/T036)
```

US1 braucht keine Testnutzungsimplementierung für die Risikobewertungsprüfung.
US2 kann als Bindungsfunktion unabhängig getestet werden, seine CLI-Integration benötigt
US1. US3 prüft die gemeinsame Basis unabhängig; vollständiger Release-Smoke benötigt US2.
Innerhalb einer Story: Fixture/Test → Modell/Syntax → Auswertung → CLI → Prüfung.
Aufgaben im selben Modul nacheinander durchführen; `[P]` erlaubt keine Dateikonflikte.

### Parallel Opportunities and Examples

- US1: nach Foundation T010 (`tests/fixtures/rules/`) und T011 (`tests/cli.rs`) parallel;
  T012 folgt T010. T013–T018 schreiben dasselbe Modul und sind sequenziell.
- US2: T021 (`tests/fixtures/association/`) kann nach Foundation unabhängig von US1
  vorbereitet werden; nach T021 können T022 (`tests/association.rs`) und T023
  (`src/syntax.rs`) parallel erfolgen. T024–T027 sequenziell in association.rs.
- US3: nach US2 T030 (`tests/git.rs`) und T031 (`tests/cli.rs`) parallel. Zielbuilds
  in T034 in getrennten Buildumgebungen; Validierungsprotokoll anschließend konsolidieren.
- T035 Dokumentation parallel zu abschließenden Prüfvorbereitungen, T036 danach.

## Implementation Strategy

### MVP First

Setup/Foundation und US1 liefern einen überprüfbaren Risikoklassen-Demo-Meilenstein.
Kein Release als vollständiger MVP ohne die bestätigte Testnutzung und Fehlerverträge.
Danach US2, US3 lokal und die vier Zielprüfungen; jede Ergänzung muss US1 weiter bestehen lassen.

### Incremental Delivery

Erst Parser-/Crate-Kompatibilität, dann einzelne Signalgruppen anhand fester Fixturepaare;
danach Bindungsprofile und CLI-Integration. Releaseziele separat nachweisen. Die zugehörigen
Tasks markieren den realen Fortschritt; bei fehlender Buildumgebung T034 offen lassen.

## Notes

### Implementation status

**Tagesabschluss 2026-10-13: auf Nutzerwunsch pausiert.**
Verbindlicher Wiedereinstiegspunkt: [implementation-status.md](implementation-status.md).

Cargo-Grundgerüst und native CLI implementiert; Testsuite umfasst 45 Testfunktionen.
Letzte volle Suite: 41 erfolgreich; spätere Ergänzungen nur fokussiert geprüft.
Alle 14 Kategorien besitzen Referenzpaare. Direkte Testnutzung, statische Aufrufe und
ausgewählte negative Grenzen sind geprüft. Offene Checkboxen umfassen teilweise bereits
implementierte Aufgaben, deren vollständige Vertragsmatrix noch nicht nachgewiesen ist.
Sie werden erst nach vollständiger Validierung abgeschlossen. Details: validation.md.

T034: Linux-arm64-Test/Build erfolgreich, Linux-x86_64-Tests unter Docker-Emulation
und Releasebuild unter Emulation erfolgreich für einen früheren Quellstand. macOS-x86_64-Build
erfolgreich, Ausführung auf diesem arm64-Mac wegen fehlender x86_64-Unterstützung blockiert.
Kein Zielruntime-Erfolg für Intel-macOS behauptet.
Der Prüfcontainer wurde beendet; keine erneuten Docker-Läufe während der Funktionsarbeit.
Neue Tests sichern Testsyntax ohne Dateinamenskonvention, gemischte Dateien, Symbol-/
Frameworküberschattung, Suitecontainer, Dekoratoren, SQL-Kommentare/Strings, mehrere
CSS-Regeln, direkte Funktionsimports und Datei-/Zeilenlimits ab. Offene Aufgaben bleiben
bis zum vollständigen Nachweis offen; validation.md listet den verbleibenden Umfang.
Keine weiteren Implementierungs-/Test-/Containerläufe heute. Die offene Aufgabenliste
wird beim Wiedereinstieg anhand konkreter Funktionslücken abgearbeitet.

- 36 Aufgaben: Setup 3, Foundation 6, US1 11, US2 9, US3 5, Polish 2.
- Sämtliche FR-001–016 und SC-001–008 sind Aufgaben und Vertragsprüfungen zugeordnet.
- Keine produktive Testausführung durch risk-analyzer; die Entwicklungstests prüfen nur
  das Werkzeug und harmlose synthetische Repositorys.
- Testfälle müssen fehlende Funktionen zeigen; fehlende Tests nicht durch reine
  Implementierungsdetails ersetzen. Keine Behauptung semantischer Sicherheit.
