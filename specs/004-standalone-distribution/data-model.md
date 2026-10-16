# Data Model: Release-Matrix

Dokumentierte Release-Daten, keine neuen Anwendungsmodelle oder persistente Laufzeitdaten.

- **Artifact**: App-Version, Regelstand, Commit/Quellstand, Target, native Datei, gebündelter
  Funktionsumfang, OS-/Git-Voraussetzungen und Verknüpfungsnachweis.
- **Validation**: Artefaktidentität, Prüfumgebung, native/emulierte Ausführung,
  Referenzfallversion, Build-/Runtimeergebnisse und Datum.
- **TargetStatus**: NOT_BUILT/BUILT/RUNTIME_VERIFIED/BLOCKED; fehlender Runner als Ursache.

BUILT allein ist keine Runtimefreigabe. Nach Quelländerung gilt ein alter Nachweis nur
für den alten Quellstand. Kernrelease requires 002; Vollprodukt zusätzlich 003.
Freigabe erst nach aktuellen Nachweisen aller vier Zielkombinationen.
