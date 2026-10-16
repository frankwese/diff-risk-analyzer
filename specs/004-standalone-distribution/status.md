# Status: Standalone-Auslieferung

**Stand**: Übernahme aus eb5de1e, keine neuen Builds/Containerläufe.
**Status**: Release nicht freigegeben; aktuelle Vierziel-Runtime-Nachweise fehlen.

| Target | Historischer Nachweis | Aktuelle Freigabe |
| --- | --- | --- |
| macOS arm64 | Native Releasebuild, nur libSystem; vor letzten Quelländerungen | Offen, aktuelles Artefakt prüfen |
| macOS x86_64 | Cross-Build erfolgreich, Start auf arm64-Host blockiert | Blockiert: geeigneter Runtime-Runner fehlt |
| Linux arm64 musl | Frühere Docker-Tests/Build, statische Verknüpfung | Offen, aktueller Stand/saubere Runtime fehlt |
| Linux x86_64 musl | Frühere Tests/Build unter Emulation | Offen, aktuelle Zielruntimeprüfung fehlt |

Originalnachweise: [001 validation.md](../001-diff-risk-analysis/validation.md).
Container beendet, Laufzeitprüfungen pausiert. Erst 002 und ggf. 003 abschließen, dann
gezielt aktuelle Releaseartefakte prüfen. Cross-Build/Emulation nicht als native Freigabe
übernehmen. Git bleibt Nutzerabhängigkeit, kein Docker/Sprachruntime erforderlich.
