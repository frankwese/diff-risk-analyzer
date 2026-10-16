# Status: Lokale Risikobewertung

**Stand**: Dokumentenmigration am 2026-10-13, Basiscommit eb5de1e.
**Status**: Teilweise implementiert; noch nicht konvergiert.

Cargo/Git/CLI und 14 Regelkategorien vorhanden; Referenzpaare, max-Level, deterministische
Gründe, HIGH-plus-unknown, SQL-Kommentare/Strings, mehrzeilige SQL/CSS, Manifestgrenzen und
mehrere Git-/Limitfehler geprüft. Nachweise aus
[001 validation.md](../001-diff-risk-analysis/validation.md) und
[Tagesabschluss](../001-diff-risk-analysis/implementation-status.md).

Keine neue Codeprüfung beim Split. Letzte volle Gesamtsuite 41 Fälle; heutiger historischer
Quellstand enthält 45 Testfunktionen mit späteren fokussierten Nachweisen.

Offen: Raw-Framing/Pfadvalidierung, vollständiges Regionsaccounting/Manifest-Randfälle,
Timeout-/Cleanup-/kumulative Limits, vollständige Kern-Vertragsmatrix. Aufgaben T004–T009,
T013–T018 und T021–T025 konkret abschließen; kein Testzuordnungs-/Vierziel-Blocker für 002.
