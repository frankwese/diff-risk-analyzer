# Feature Specification: Standalone-Auslieferung

**Feature Branch**: `main` (kein neuer Branch angelegt)

**Created**: 2026-10-13

**Status**: Spec geprüft; frühere Buildnachweise vorhanden, Release nicht freigegeben

**Input**: Native plattformübergreifende Auslieferung aus Gesamt-Spec 001 als separates
Feature; Risikobewertung und Testzuordnung behalten jeweils eigene funktionale Abschlüsse.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Werkzeug ohne Sprachlaufzeit starten (Priority: P1)

Entwickler erhalten das passende Executable und analysieren lokale Revisionen ohne
Sprachlaufzeit, Compiler, Paketmanager oder erstmaliges Nachladen von Analysekomponenten.

**Why this priority**: Dies ist der vereinbarte Nutzen des Standalone-Tools.

**Independent Test**: Aktuelles Executable auf sauberem Zielsystem mit Git offline starten;
alle vereinbarten Analysefälle des freigegebenen Funktionsumfangs ausführen.

**Acceptance Scenarios**:

1. **Given** passendes Executable, unterstütztes System und Git, **When** help/version
   oder Analyse gestartet wird, **Then** keine separate Sprachlaufzeit erforderlich.
2. **Given** kein Netzwerk, **When** lokale Revisionen analysiert werden, **Then** keine
   Laufzeitdownloads und Ergebnisse wie auf dem Entwicklungsziel.

### User Story 2 - Passendes geprüftes Release erhalten (Priority: P1)

Entwickler erkennen unterstützte Plattform/Architektur, Version und Installationsvoraussetzungen.

**Why this priority**: Ein Cross-Build allein garantiert noch keinen nutzbaren Zielrelease.

**Independent Test**: Release-Matrix ordnet jedes aktuelle Artefakt Quellstand und
ausgeführter Zielprüfung zu; fehlende Nachweise verhindern vollständige Freigabe.

**Acceptance Scenarios**:

1. **Given** macOS oder Linux mit x86_64 oder arm64, **When** das passende Artefakt gewählt
   wird, **Then** startet es und erfüllt die vereinbarten Referenzfälle.
2. **Given** vollständiges Produkt aus 002 und 003, **When** Release geprüft wird,
   **Then** funktionieren auch direkte Testnutzungsfälle aller vier Quellsprachen ohne externe Parser.
3. **Given** nur Cross-Build oder ältere Testnachweise, **When** Status dokumentiert wird,
   **Then** kein unbelegter aktueller Zielruntime-Erfolg.

### Edge Cases

- Falsche Architektur, fehlendes/ungeeignetes Git, veraltetes Artefakt oder externe Parser-
  Dateien dürfen nicht als erfolgreich geprüfte Auslieferung erscheinen.
- Linux-OS-Abhängigkeiten und macOS-Systembibliotheken erklären; kein Docker beim Nutzer nötig.
- Erstes Kernrelease aus 002 möglich; vollständiges Produktrelease zusätzlich von 003 abhängig.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Native Executables MÜSSEN für macOS/Linux jeweils x86_64/arm64 geliefert werden.
- **FR-002**: Nutzer DÜRFEN keine separate Sprachlaufzeit, Compiler oder Paketmanager
  benötigen; Git bleibt externe Voraussetzung. Regeln und benötigte Parser sind mitzuliefern.
- **FR-003**: Analyse MUSS offline ohne erstmalige Komponenteninstallation funktionieren;
  unterstützter Funktionsumfang MUSS identisch mit den abgeschlossenen Features sein.
- **FR-004**: Ein vollständiges Produktrelease MUSS 002 und 003 einschließlich aller vier
  Sprachprofile enthalten; ein Kernrelease darf 003 nicht als enthalten/abgeschlossen behaupten.
- **FR-005**: Jedes Artefakt MUSS Version, Plattform/Architektur, Quellstand und belegten
  Prüfumfang besitzen. Veraltete Nachweise oder Cross-Builds ersetzen keine aktuelle Laufzeitprüfung.
- **FR-006**: Installation, Aufruf, Git-/OS-Voraussetzungen, Grenzen und Versionserkennung
  MÜSSEN dokumentiert sein; keine Voraussetzung Docker für Nutzer.

### Key Entities *(include if feature involves data)*

- Releaseartefakt: native Datei, Version, Quellstand, Zielsystem und Funktionsumfang.
- Prüfnachweis: Ziel, Teststand und Ergebnisse; Cross-Build und Runtime getrennt.
- Release-Matrix: vier Zielkombinationen mit Freigabestatus und Blockern.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Auf allen vier Zielen läuft das aktuelle passende Executable offline
  ohne separate Sprachlaufzeit und besteht die Kernreferenzfälle aus 002.
- **SC-002**: Für vollständige Produktfreigabe bestehen zusätzlich die vereinbarten
  Testzuordnungsfälle aus 003 einschließlich direkter Nutzung in allen vier Profilen.
- **SC-003**: Alle vier Artefakte besitzen nachvollziehbare aktuelle Quellstand-/Runtime-
  Nachweise; kein blockiertes oder ungetestetes Ziel wird als freigegeben markiert.
- **SC-004**: Nutzer starten anhand der dokumentierten Schritte help/version und eine
  lokale Analyse ohne Zusatzinstallation außer Git und OS-Systemvoraussetzungen.

## Assumptions

- 002 liefert das Kernprodukt; 003 erweitert es. Releasearbeit kann nach 002 beginnen,
  vollständige Produktfreigabe benötigt beide. Rust-Buildtechnik gehört in den Plan.
- Keine Store-Veröffentlichung, automatische Updates, Telemetrie oder CI-Produktintegration.
- Alte Plattformnachweise aus 001 sind Historie und keine Freigabe des heutigen Quellstands.
- Ursprung und Aufgabenmigration siehe [../feature-split.md](../feature-split.md).
