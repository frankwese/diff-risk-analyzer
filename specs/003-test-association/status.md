# Status: Zugehörige Teständerungen

**Stand**: Übernahme aus eb5de1e am 2026-10-13, ohne neue Codeprüfung.
**Status**: Teilweise implementiert; noch nicht konvergiert.

Vier Parser, direkte Klassen-/Methoden-/Funktionsnutzung, Aliase, statische Aufrufe,
Testcontainer, gemischte Dateien, fehlende/indirekte Nutzung und ausgewählte negative
Grenzen implementiert. Historischer letzter fokussierter Lauf: 20 Zuordnungstests erfolgreich;
kein neuer vollständiger Quellstandtest behauptet. Originalnachweise:
[001 validation.md](../001-diff-risk-analysis/validation.md).

Offen: vollständige negative Matrix je Profil, mehrdeutige Imports/Symbolherkunft,
Klassenfelder/Fixtures, Unsupported-Syntax, vollständiger Testfalländerungs-/Statusnachweis.
T002–T009 und T011/T012 weiterbearbeiten; vorhandene Marker unverändert übernommen.
Kein Zielrelease-Gate für Funktionsabschluss, Integration bleibt abhängig von 002.
