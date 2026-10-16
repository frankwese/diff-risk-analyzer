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
T001–T026 waren vor dieser Konvergenzrunde abgeschlossen. Aktuelle Umsetzung, FR-/SC-Matrix, Prüfungen und
konservativer endlicher Regelumfang stehen in status.md. Keine Freigabe von 003/004.

## Phase 6: Convergence

Konvergenzprüfung: bestehende Erledigtmarker unverändert; sechs verbleibende Lücken.
Die folgenden Aufgaben korrigieren den aktuellen Code, ohne den vereinbarten Umfang
durch schwächere Anforderungen oder geänderte Abschlusskriterien zu ersetzen.

- [X] T027 [US1] CRITICAL: Ersetze den Infrastruktur-Neutralitätsvergleich über `sql_code()` in `src/rules.rs` durch formatgerechte Vergleiche, die Stringwerte erhalten; ergänze in `tests/rules.rs` Regressionen für geänderte IAM Action/Resource/Effect/Principal-Werte und produktive Infrastrukturwerte sowie reine Kommentare. Änderungen dürfen nicht durch das Entfernen von Literalen LOW werden; Geheimniswerte weiterhin nicht ausgeben, per Constitution IV, FR-005/FR-008 und T016 (contradicts; F1).
- [X] T028 [US1] Begrenze den Alt-/Neu-Abgleich von SourceRegion in `src/rules.rs` auf die zugehörige Funktion/Klasse und passende Deklaration statt dateiweitem Text-Multiset; ergänze in `tests/rules.rs` getauschte identische Returns und verschobene Anweisungen zwischen Auth-/Nicht-Auth-Funktionen. Funktionswechsel darf weder Änderungen noch Auth-Kontext als unverändert ausblenden, per FR-006/FR-008/FR-011 und T018/T026 (contradicts; F2).
- [X] T029 [US1] Vervollständige die bestätigte Auth-Kontextregel in `src/rules.rs`: erkennbare nicht neutrale Logik-/Vertragsänderungen innerhalb derselben Auth-Funktion müssen den HIGH-Kontextbefund erhalten, auch wenn kein anderes Signal die Änderung abdeckt; unabhängige Änderungen außerhalb dieser Funktion bleiben unsupported. Prüfe tatsächliche Prüfaufrufe gegenüber bloßen Identifiern, Kommentaren, Strings und verschachtelten Funktionen in `tests/rules.rs`, per FR-011/SC-006 und T026 (partial; F3).
- [X] T030 [US1] Vervollständige begrenzte Manifest-/API-Vertragsabgrenzung in `src/rules.rs`, `src/structured.rs` und `src/syntax.rs`: Dependency-Textmarker dürfen unbekannte Nachbarfelder oder unvollständige Blöcke nicht verdecken; öffentliche Deklarationen ohne Methodenkörper dürfen bei API-Erweiterung/Vertragsbruch nicht übersprungen werden. Ergänze in `tests/rules.rs` einzeilige Maven-Blöcke mit Metadatenänderung, fehlende Blockabschlüsse und geänderte/entfernte deklarierte öffentliche Signaturen; nicht unterstützte Formen ausdrücklich ablehnen, per FR-005/FR-008 und T015/T017 (partial; F4).
- [X] T031 [US2] Stelle beim gesamten CLI-Timeout eine koordinierte Abbruch-/Cleanup-Grenze für Worker und laufende Git-Prozesse in `src/cli.rs`, `src/git.rs` und `src/model.rs` her; ein beendetes Warten darf keinen weiterlaufenden Analyseworker oder Git-Prozess hinterlassen. Ergänze kurze kontrollierte Gesamtdeadlinefälle in `tests/cli.rs` beziehungsweise Unit-Tests mit Exit 1, leerem stdout und belegtem Cleanup, per FR-007/SC-005, plan: 60s Gesamtlaufzeit und T023 (partial; F5).
- [X] T032 Ergänze nach den Korrekturen CLI-End-to-End- und zehnfache Reproduzierbarkeitsnachweise für AUTH_CONTEXT_CHANGE im Regelstand 1.1.0 in `tests/cli.rs`; prüfe geänderten Return mit unveränderter Prüfung, Kommentaränderung und Änderung einer anderen Funktion. Aktualisiere danach die tatsächlichen Prüf-/Abschlussangaben in `status.md`, `quickstart.md` und `README.md`, einschließlich vollständiger Aufgabenzahl und noch offener Konvergenzaufgaben; keine alte Teilprüfung als aktuellen Gesamtabschluss ausgeben, per SC-003/SC-006 und T025/T026 (partial; F6).

Abhängigkeiten: T027 zuerst; T028 vor T029. T030 und T031 können in getrennten
Arbeitsschritten umgesetzt werden, gemeinsame Module nicht gleichzeitig verändern.
T032 folgt allen Codekorrekturen. Danach erneut `/speckit.converge` für Feature 002.

## Phase 7: Convergence

Erneuter Code-/Intent-Abgleich nach Umsetzung von T027–T032; bestehende Aufgaben und
Erledigtmarker bleiben unverändert. Folgende Restabweichungen sind noch zu beheben:

- [X] T033 [US1] CRITICAL: Ersetze den ungeordneten Regions-Multiset-Abgleich innerhalb einer Funktion in `src/rules.rs` durch einen geordneten, kontrollstrukturbezogenen Vergleich. Das Verschieben einer Auth-Prüfung hinter eine Operation oder zwischen unterschiedlichen Zweigen darf nicht als unveränderte beziehungsweise nicht ausführbare Änderung LOW liefern. Ergänze in `tests/rules.rs` Regressionen für vertauschte Anweisungen derselben Funktion, verschobene Auth-Prüfung und geänderte Zweigzugehörigkeit; unveränderte Formatierung weiterhin getrennt behandeln, per Constitution IV/V, FR-006/FR-008/FR-011 und T028 (contradicts; F7).
- [X] T034 [US1] Ergänze eindeutige Deklarations-/Signaturidentität für SourceRegion-Herkunft und Auth-Kontext-Gegenparts in `src/rules.rs` und bei Bedarf `src/syntax.rs`; gleichnamige überladene Methoden dürfen keine Regionen gegenseitig ausgleichen oder den Auth-Kontext eines beliebigen ersten Gegenparts übernehmen. Prüfe in `tests/rules.rs` getauschte Returns zwischen Überladungen und unterschiedliche Auth-Kontexte gleicher Methodennamen; nicht eindeutig zuordenbare Formen ausdrücklich als nicht bewertbar behandeln, per FR-008/FR-011 und T028/T029 (partial; F8).
- [X] T035 [US2] Verhindere in `src/cli.rs`, dass der öffentlich direkt aufrufbare Schalter `--internal-analysis` die Gesamttimeout-/Cleanup-Grenze umgeht. Binde den internen Einstieg an die tatsächlich überwachte Prozessausführung oder stelle dieselbe harte Grenze für jeden unterstützten Analysepfad sicher; ergänze kurze kontrollierte Regressionen in `tests/cli.rs` beziehungsweise CLI-Unit-Tests für direkten internen Aufruf, blockierte CPU-/Prozessanalyse, Exitstatus und Cleanup. Kein erfolgreicher Analyseweg darf ohne die vereinbarte 60s-Grenze laufen, per FR-007/SC-005, plan: 60s Gesamtlaufzeit und T031 (contradicts; F9).
- [X] T036 [US1] Ersetze die verbliebene zeilenbasierte Gradle-Dependency-Abgrenzung in `src/rules.rs` durch literal-/kommentarbewusste Token- beziehungsweise Bytebereichsgrenzen des tatsächlichen dependencies-Blocks. Änderungen an anderen Einstellungen vor/nach dem Block auf derselben Zeile dürfen nicht als Dependency-Update mitbewertet werden. Ergänze in `tests/rules.rs` einzeilige Blocks mit veränderter Nachbareinstellung, verschachtelten Blocks, Klammern in Strings/Kommentaren und reinem Dependency-Update; mehrdeutige Konstrukte konservativ ablehnen, per FR-008 und T015/T030 (partial; F10).

T033 vor T034 im gemeinsamen Regionsvergleich bearbeiten. T035 betrifft die
CLI-Prozessgrenze; T036 die Manifestabgrenzung. Nach den Korrekturen gezielte
Regressions-/Kompatibilitätsprüfungen dokumentieren und erneut für 002 konvergieren.

## Phase 8: Convergence

Erneuter Abgleich nach T033–T036: drei konkrete Restvarianten bestehender Anforderungen.
Keine Änderung vorhandener Aufgaben oder Erledigtmarker durch diese Prüfung.

- [X] T037 [US1] CRITICAL: Ergänze die AST-bezogene Then-/Else-/Zweigidentität in der SourceRegion-Herkunft in `src/rules.rs`. Die gleiche if-Bedingung darf Konsequenz und Alternative nicht zu demselben Vergleichsraum machen; Anweisungen zwischen diesen Zweigen dürfen nicht gegenseitig als unverändert ausgeglichen werden. Ergänze in `tests/rules.rs` Java-/JS-/TS-Fälle mit gleicher Bedingung und zwischen Then/Else getauschten Auth-/Zahlungsoperationen sowie unverändertem Kontrollfall; fordere Auth-Kontext statt LOW oder verschwundenem Befund, per Constitution IV/V, FR-008/FR-011 und T033 (contradicts; F11).
- [X] T038 [US1] Trenne direkte Auth-Prüfaufrufe und geänderte Auth-Deklarationen von bloßen Identifiern in `src/rules.rs` und bei Bedarf `src/syntax.rs`. Eine Variable, Methodenreferenz oder sonstige Namensnennung authorize/authenticate/checkPermission/hasRole/verifyToken darf allein keine direkt geänderte Auth-Operation behaupten. Ergänze in `tests/rules.rs` positive tatsächliche Aufrufe und negative Variablen-/Referenzfälle; funktionsbezogenen AUTH_CONTEXT_CHANGE und Auth-Pfadregeln erhalten, per FR-006/FR-011 und T029 (partial; F12).
- [X] T039 [US1] Berücksichtige implizit öffentliche Java-Interface-Methoden ohne geschriebenen public-Modifier in der Vertrags-Symbolerkennung in `src/syntax.rs` und `src/rules.rs`. Ergänze in `tests/rules.rs` hinzugefügte, entfernte und geänderte Interface-Signaturen wie int get(); und private Interface-Methoden als Gegenbeispiele; API_EXTENSION/CONTRACT_BREAK im vereinbarten API-Vertragsbereich korrekt ausgeben, per FR-005/FR-008 und T030 (partial; F13).

T037 zuerst; T038 und T039 danach mit getrennten Regressionen. Anschließend den
betroffenen Regel-/CLI-Vertrag prüfen und erneut für Feature 002 konvergieren.
