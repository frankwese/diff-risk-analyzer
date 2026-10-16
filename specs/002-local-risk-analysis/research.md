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
