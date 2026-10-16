# Research: Lokale Risikobewertung

**Decision**: Rust 1.99.0, vorhandene Git-/Diff-/Parser-/Serde-Pins und Regelkatalog übernehmen.
**Rationale**: Die Aufteilung verändert weder Architektur noch bestätigte Heuristiken.
**Alternatives considered**: Neue Crate oder neue Sprachwahl würde vorhandene Arbeit duplizieren.

Read-only-Prozesspolitik, rohe NUL-/Blobdaten, ähnliche Zeilendeltas und statische Befunde
gelten weiter. Ursprüngliche technische Recherche und Quellen:
[001 research.md](../001-diff-risk-analysis/research.md), Abschnitte 1, 3, 4 und 6.
Syntax zur Neutralität/Testklassifikation wird gemeinsam genutzt; Nutzungsbindung gehört 003.

Keine neuen technischen Unbekannten; bestehende Risiken sind konkrete offene Aufgaben,
keine erneute Technologieentscheidung. Alte Prüfnachweise sind keine neuen Testläufe.

## T005: Begrenztes Prozesscleanup

**Decision**: Child-Prozessgruppe, nicht blockierende Unix-Pipes und explizite
Abbruchsignale für I/O-Threads, libc 0.2.190 als direkte Dependency.
**Rationale**: Blockierende joins konnten nach Elternexit bei vererbten Pipes hängen;
stdin-Schreibfehler wurden erst nach Prozessende sichtbar. Deadline muss alle I/O-
Phasen begrenzen, erfolgreiche Ausgabe dagegen vollständig lesen.
**Alternatives considered**: Nur Child.kill beendet keine geerbten Pipehalter;
abgetrennte Threads lassen Ressourcen offen. Eigenes Prozessframework nicht erforderlich.
Prüfungen und T005-Abschluss siehe status.md; libc war bereits im vorhandenen Lockfile.
