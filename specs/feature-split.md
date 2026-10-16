# Migration der Gesamt-Spec 001

**Datum**: 2026-10-13
**Auftrag**: Bestehenden Umfang in drei abschließbare Specs teilen, Spec-Kit-Workflow nutzen.
**Quellstand**: `eb5de1e` — letzter Commit vor Dokumentenmigration.

## Aktive Features

| Feature | Besitzer des Umfangs | Abschluss |
| --- | --- | --- |
| [002 Risikobewertung](002-local-risk-analysis/spec.md) | Git, 14 Regeln, Risk/Reasons, Fehler/read-only | Kernreferenzen auf Entwicklungsziel erfüllt |
| [003 Testzuordnung](003-test-association/spec.md) | Symbol-/Teständerungen, direkte Nutzung, Zustände | Eigene vierprofilige Matrix und CLI-Erweiterung erfüllt |
| [004 Auslieferung](004-standalone-distribution/spec.md) | Vier native Zielartefakte und Runtime-Nachweise | Aktuelle Zielartefakte ohne Sprachlaufzeit geprüft |

002 → 003; 002 → 004-Kernrelease. Vollproduktfreigabe von 004 benötigt 002 und 003.
Eine gemeinsame Rust-Crate bleibt bestehen. Keine Code-/Testmigration oder neue Builds.
001 ist abgelöste Historie, keine parallel aktive Implementierungsaufgabenliste.

## Vollständige Anforderungszuordnung (alte IDs → neue Besitzer)

| Alt 001 | Neu | Erhaltener Umfang |
| --- | --- | --- |
| FR-001 | 002 FR-001 | Zwei-Revisionsvergleich/Unterverzeichnis |
| FR-002 | 002 FR-002; 003 FR-009; 004 FR-003 | Offline/read-only; keine Cloud/LLM oder Codeausführung |
| FR-003 | 002 FR-003 | Drei Stufen, max-Level und leerer Diff |
| FR-004 | 002 FR-004 | Konkrete Signal-/Datei-/Änderungsgründe |
| FR-005 | 002 FR-005 | Alle 14 fachlichen Signale |
| FR-006 | 002 FR-005/006 | Endlicher sprachübergreifender Regelumfang |
| FR-007 | 002 FR-006; 003 FR-007 | Heuristik statt Garantie |
| FR-008 | 003 FR-001/002 | Echte Klassen-/Methodennutzung; Namen allein reichen nicht |
| FR-009 | 002 FR-010; 003 FR-007/008 | Test-/Produktionsabgrenzung, keine Risikoabsenkung |
| FR-010 | 002 FR-004; 003 FR-007 | Reproduzierbare geordnete Ausgabe |
| FR-011 | 002 FR-007 | Exit-/stdout-/stderr-Vertrag |
| FR-012 | 002 FR-008 | Unbekannt nicht als LOW/Teil-Risk |
| FR-013 | 002 FR-009; 003 FR-009; 004 FR-006 | Dokumentation und keine Geheimniswerte |
| FR-014 | 003 FR-003/005 | Geänderte Einzeltests und echte Nutzungsfundstellen |
| FR-015 | 003 FR-004/007/008 | Zustände, unklare Nutzung kein Kernfehler |
| FR-016 | 004 FR-001–006 | Vier Standaloneziele, Parser gebündelt, Git extern |
| SC-001 | 002 SC-001 | Nutzbarer lokaler Einzelaufruf |
| SC-002 | 002 SC-002 | 14 Referenzpaare |
| SC-003 | 002 SC-003; 003 SC-004 | Zehn identische Wiederholungen |
| SC-004 | 002 SC-004 | Nachvollziehbare Fundstelle |
| SC-005 | 002 SC-005 | Fehler ohne Entwarnung |
| SC-006 | 003 SC-004 | Tests verändern Risk nicht |
| SC-007 | 003 SC-001–003 | Vollständige Testzuordnungsmatrix |
| SC-008 | 004 SC-001–003 | Vier laufzeitfreie Zielprüfungen einschließlich 003 |

001 US1 → 002 US1; 001 US2 → 003 US1/US2, Tests-only-Signal zu 002;
001 US3 Szenarien 1–4 → 002 US2, Szenario 5 → 004 US1/US2.
Alle alten Edge Cases sind 002/003/004 oder deren übernommenen Verträgen zugeordnet.

## Aufgabenmigration (keine neue Erfolgsbehauptung)

| Alt T-ID | Aktive Aufgaben |
| --- | --- |
| T001–T003 | 002 T001–T003, 004 T001/T008 (Release-Dokumentation) |
| T004–T009 | 002 T004–T009; 003 T001 (genutzte Syntaxbasis) |
| T010–T020 | 002 T010–T020 |
| T021–T029 | 003 T002–T010 |
| T030 | 002 T021 |
| T031 | 002 T022 (Kernfehler), 004 T002/T003 (Release-Smokes) |
| T032 | 002 T023 |
| T033 | 004 T004 |
| T034 | 004 T005–T007 |
| T035 | 002 T024, 003 T011, 004 T008 |
| T036 | 002 T025, 003 T012, 004 T009 |

Erledigtmarker werden ausschließlich aus 001 übernommen, nicht anhand neuer Testläufe
erhöht. Alte Sammelaufgaben werden bei geteiltem Umfang konservativ offen übernommen.
Neue IDs gelten nur innerhalb ihrer Feature-Aufgabenliste; Referenzen immer mit Feature.

## Workflow und aktive Auswahl

Jedes Feature separat: specify → Anforderungscheckliste → plan mit research/data-model/
contracts/quickstart → tasks mit Vorzustand → Dokumentenkonsistenzprüfung.
Keine extensions.yml vorhanden, daher keine registrierten Pre-/Post-Hooks.
Templates über resolve-template.sh aufgelöst; keine Templatequellen geändert.
Aktuell ausgewählt: `specs/002-local-risk-analysis` in `.specify/feature.json` (lokaler Pointer).

Auswahl für spätere Kommandos über den vorhandenen Featurepfad, etwa:

```sh
SPECIFY_FEATURE_DIRECTORY=specs/003-test-association .specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
```

Im Agenten ebenfalls das gewünschte Feature nennen; ohne Override arbeitet er auf 002.
Vor Implementierung Checkliste und tasks.md des ausgewählten Features prüfen.
Die Setup-/Prerequisite-Skripte können den lokalen Pointer auf das übergebene Feature
aktualisieren. Nach der Migration wurde er deshalb abschließend wieder auf 002 gesetzt.

## Konsistenzprüfung der Migration

- Alle 16 alten FR und acht SC besitzen explizite Zielzuordnungen; kein Signal entfernt.
- Alle 36 alten Tasks besitzen Zielaufgaben. Aufspaltung der Sammel-/Release-/Polish-
  Aufgaben ergibt 46 lokale Tasks: 002 mit 25 (7 übernommen erledigt), 003 mit 12
  (3 übernommen erledigt), 004 mit 9 (alle offen). Keine 46 neuen Implementierungsarbeiten.
- Drei Anforderungschecklisten je 16/16 anhand der Dokumente geprüft.
- Setup-plan/setup-tasks und require-tasks-Prerequisiteprüfung je Feature erfolgreich;
  Git-Whitespaceprüfung ohne Befund. Neue Pläne folgen der aufgelösten Template-Hierarchie.
- Gemeinsame Daten-/CLI-/Syntaxmodule bleiben eine Crate; Änderungen daran brauchen
  Kompatibilitätsprüfung anderer Features, aber keine fremden Abschlussgates.
- 002 CLI besitzt Risk/Reasons/Fehler, 003 nur Tests-Erweiterung, 004 nur Releaseprüfung.
  Kernabschluss ist keine Vollproduktfreigabe; ursprüngliche Test-/Vierzielanforderungen
  werden weiter durch 003/004 verlangt. 001 ausdrücklich als abgelöst gekennzeichnet.
- Keine neue Implementierungsvalidierung oder Anwendungstests behauptet. Historischer
  Pause-/Quellstand und unvollständige Prüfungen pro Feature ausgewiesen.

## Prüfnachweise und Pause

Historische Tagesabschlüsse unter 001 bleiben erhalten. Featurebezogene status.md-Dateien
verweisen auf diese Nachweise, unterscheiden ältere Releasebuilds und aktuelle Quellen.
Der dokumentierte Implementierungsstand bleibt teilweise abgeschlossen. Keine erneute
Anwendungsausführung, keine Docker-/Releaseprüfung im Rahmen dieser Dokumentenmigration.
