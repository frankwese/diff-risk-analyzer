# Feature Specification: Zugehörige Teständerungen

**Feature Branch**: `main` (kein neuer Branch angelegt)

**Created**: 2026-10-13

**Status**: Spec geprüft; Implementierung teilweise vorhanden, nicht konvergiert

**Input**: Testzuordnung aus Gesamt-Spec 001 als eigenständig abschließbare Erweiterung
der lokalen Risikobewertung herauslösen. Namen/Pfade plus echte Nutzung geänderter Symbole.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Geänderte Symbole zu Teständerungen zuordnen (Priority: P1)

Reviewer erkennen, ob ein hinzugefügter/geänderter Testfall die betroffene Klasse/Methode
tatsächlich benutzt, und können den Befund zur Fundstelle zurückverfolgen.

**Why this priority**: Passender Testdateiname allein beantwortet die Review-Frage nicht.

**Independent Test**: Zuordnungsbeispiele mit Produktions-/Teständerungen direkt prüfen;
Integration verwendet bereits vorhandene vollständige Risikobewertung aus 002.

**Acceptance Scenarios**:

1. **Given** geänderte Methode und geänderter Test mit eindeutigem passenden Aufruf,
   **When** zugeordnet wird, **Then** USED mit Symbol, Testfall, Datei und Aufrufzeile.
2. **Given** passender Testname, aber Verwendung einer anderen Methode, **When** geprüft
   wird, **Then** MISSING für die betroffene Methode.
3. **Given** zwei geänderte Methoden, aber Nutzung nur einer, **When** geprüft wird,
   **Then** pro Methode getrennte Ergebnisse.

### User Story 2 - Grenzen der Testnutzung erkennen (Priority: P1)

Reviewer unterscheiden fehlende, mehrdeutige und nicht anwendbare Nutzung von Erfolg.

**Why this priority**: Heuristische Nutzung darf keine Coverage-/Korrektheitsgarantie sein.

**Independent Test**: Negativmatrix enthält unveränderte Tests, Alias-/Fixture-/Helper-
Nutzung, Überschattung, mehrdeutige Symbole und Löschungen.

**Acceptance Scenarios**:

1. **Given** nur Test B geändert, Nutzung in unverändertem Test A, **When** geprüft wird,
   **Then** A zählt nicht; unveränderte Aufrufzeile im tatsächlich geänderten Test zählt.
2. **Given** Factory, Fixture, indirekter Aufruf oder mehrdeutige Methode, **When** nicht
   eindeutig zuordenbar, **Then** UNKNOWN mit Ursache statt falschem USED.
3. **Given** gelöschtes Produktionssymbol, **When** geprüft wird, **Then** NOT_APPLICABLE.
4. **Given** gleiche Risikoänderung mit unterschiedlichen Testzuständen, **When** analysiert
   wird, **Then** Risikoklasse unverändert; UNKNOWN allein erzeugt keinen Analysefehler.

### Edge Cases

- Kommentare, Strings, Import oder bloße Namen sind kein Nutzungsbefund.
- Testfallkopf/Annotation/Dekorator zählen zu seiner Änderung; Setup/Imports/Suiteänderung
  allein aktivieren keinen unveränderten Test; Zeilenverschiebung ebenso nicht.
- Gleichnamige Tests in verschiedenen Containern unterscheiden; echte Mehrdeutigkeit UNKNOWN.
- Testsyntax außerhalb üblicher Dateipfade und gemischte Dateien berücksichtigen.
- Reassignment, Überschattung, Overloads, unbekannte Syntax, reflektierte/indirekte Nutzung
  konservativ behandeln; keine Symbolnamen erfinden und keinen analysierten Code ausführen.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Geänderte Klassen, Methoden und unterstützte freie Funktionen MÜSSEN
  einzeln zu hinzugefügten/geänderten Testfällen zugeordnet werden. Bei Methode MUSS
  der passende Aufruf auf der betroffenen Klasse nachweisbar sein.
- **FR-002**: Namen/Pfade MÜSSEN Kandidaten unterstützen, dürfen echte Nutzung nicht
  ersetzen; eindeutig direkte Nutzung ohne passende Testdateinamen darf zählen.
- **FR-003**: Nur geänderte Testfälle im Zielstand zählen. Ihre vollständigen Inhalte
  MÜSSEN betrachtet werden; unveränderte Aufrufzeile im selben geänderten Fall darf zählen.
  Unveränderte/gelöschte Tests dürfen nicht als neue Teständerung gelten.
- **FR-004**: USED/MISSING/UNKNOWN/NOT_APPLICABLE MÜSSEN pro Symbol ausgegeben werden.
  Mehrdeutige/indirekte oder nicht erkannte Bindung MUSS UNKNOWN mit Ursache sein.
- **FR-005**: USED MUSS Testdatei, Testfall, Symbol und Nutzungsfundstelle enthalten;
  Kommentar-/Literal-/Importnamen ohne Aufruf dürfen kein USED liefern.
- **FR-006**: Das vereinbarte endliche Profil MUSS Java, JavaScript, TypeScript und Python
  umfassen; bekannte ausgeschlossene Konstrukte sind als Erkennungsgrenzen zu dokumentieren.
- **FR-007**: Testhinweise MÜSSEN getrennt von Risiko/Gründen sein und reproduzierbar
  geordnet werden. Keine Testausführung/Coveragebehauptung oder Änderung an Risikostufen.
- **FR-008**: UNKNOWN allein DARF einen sonst vollständigen Vergleich aus 002 nicht
  fehlschlagen lassen. Vorhandene Tests außerhalb des Diffs beweisen keine neue Abdeckung.
- **FR-009**: Testzuordnung MUSS lokal/read-only ohne Laufzeitimport analysierter Quellen
  erfolgen; technische Analysegrenzen gelten wie bei 002. Dokumentation MUSS Grenzen erklären.

### Key Entities *(include if feature involves data)*

- Symboländerung: Datei, Container, Signatur und Änderungsart.
- Testfalländerung: eindeutiger Testcontainer/-name und Basis-/Zielbereich.
- Nutzungsbefund: Symbol, Aufruf und eindeutig zugeordnete Herkunft/Instanz.
- Testzuordnung: Status, Ursachen und geordnete Belege.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Alle direkten Nutzungsfälle der vier vereinbarten Sprachprofile liefern USED.
- **SC-002**: Die Referenzmatrix aus falscher Klasse/Methode, Scheinnennung, unverändertem
  Test, Teilzuordnung, indirekter Nutzung, Löschung und Mehrdeutigkeit liefert vereinbarte Zustände.
- **SC-003**: Jeder USED-Befund lässt sich einem geänderten Testfall und Aufruf zuordnen.
- **SC-004**: Testzustandsvarianten verändern weder Risikoklasse noch Erfolg einer
  vollständigen Kernanalyse; zehn Wiederholungen liefern identische Testhinweise.

## Assumptions

- Abhängig von 002 für CLI-/Git-Eingaben und Risikovertrag; Zuordnungslogik separat testbar.
- Keine vollständige Typ-/Vererbungs-/Callgraph-/Coverageanalyse. Endliche Profile stehen
  in contracts/test-association.md; Rust als Umsetzung bleibt im Plan.
- Standalone-Bündelung der Sprachprofile ist Anforderung der Auslieferung 004.
- Ursprung und Aufgabenmigration siehe [../feature-split.md](../feature-split.md).
