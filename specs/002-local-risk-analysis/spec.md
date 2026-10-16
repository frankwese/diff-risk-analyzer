# Feature Specification: Lokale Risikobewertung

**Feature Branch**: `main` (kein neuer Branch angelegt)

**Created**: 2026-10-13

**Status**: Kernumfang implementiert und lokal validiert; Abschlussnachweise in status.md

**Input**: Bestehenden Git Diff Risk Analyzer in unabhängig abschließbare Features teilen.
Dieses Feature besitzt den lokalen Revisionsvergleich, 14 Risikosignale, begründete
Risikoklassen und das Fehler-/Read-only-Verhalten. Herkunft: Gesamt-Spec 001.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Änderungen vor Push oder Review einschätzen (Priority: P1)

Entwickler vergleichen zwei lokale Revisionen und erhalten eine verständliche Risikoklasse
mit konkreten Gründen, um ihre Prüfung auf relevante Änderungen auszurichten.

**Why this priority**: Dies ist der eigenständige Kernnutzen des Werkzeugs.

**Independent Test**: Vergleich bekannter lokaler Revisionen liefert die vereinbarten
Klassen und Fundstellen ohne Testzuordnung oder plattformübergreifenden Release-Nachweis.

**Acceptance Scenarios**:

1. **Given** ausschließlich Dokumentation geändert, **When** der Vergleich ausgeführt wird,
   **Then** LOW mit Dokumentationsgrund.
2. **Given** Dependency-Update, **When** der Vergleich ausgeführt wird, **Then** MEDIUM mit
   betroffener Datei; bei Auth-Änderung oder destruktiver Migration HIGH mit Fundstelle.
3. **Given** mehrere Signalstufen, **When** analysiert wird, **Then** höchste Stufe als
   Gesamtrisiko und alle erkannten Gründe ohne identische Duplikate.
4. **Given** unveränderte Endpunkte und Regeln, **When** zehnmal analysiert wird,
   **Then** byteidentische Risikoklasse und geordnete Gründe.

### User Story 2 - Fehler und Grenzen von Entwarnung unterscheiden (Priority: P1)

Entwickler erkennen ungültige oder nicht bewertbare Vergleiche ohne falsche LOW-Ausgabe.

**Why this priority**: Fehlende Bewertung darf nicht als niedriges Risiko erscheinen.

**Independent Test**: Fehler-/Grenzfallbeispiele liefern Status und betroffene Dateien
ohne erfolgreiche Risikoklasse; ein leerer Vergleich bleibt erfolgreich unterscheidbar.

**Acceptance Scenarios**:

1. **Given** ungültige Revision oder kein Repository, **When** aufgerufen wird, **Then**
   Exit 2, stdout leer und verständliche Fehlermeldung auf stderr.
2. **Given** leerer Diff, **When** analysiert wird, **Then** LOW mit explizitem Leerhinweis.
3. **Given** unbekannter Änderungsbereich neben HIGH, **When** analysiert wird, **Then**
   Exit 1 ohne abschließende Risk-Ausgabe und mit betroffenem Pfad.
4. **Given** lokal verfügbare Objekte und Dirty Worktree, **When** offline analysiert wird,
   **Then** keine Änderung an HEAD, Index oder Dateien und keine Git-Helferausführung.

### Edge Cases

- Neue/gelöschte Dateien, exakte Renames und Mode-/Typänderungen berücksichtigen.
- Nicht eingecheckte Änderungen nicht in den expliziten Revisionsvergleich aufnehmen.
- Ungewöhnliche Pfadbytes eindeutig darstellen; unbekannte Binär-/LFS-/Gitlink-Inhalte
  nicht stillschweigend als LOW klassifizieren.
- Auth-Begriffe nur in Dokumentation/Tests nicht als produktive Auth-Änderung behandeln;
  gemischte Dateien nach Änderungsbereichen trennen.
- Additive Migrationen nicht allein als destruktiv werten. SQL-Kommentare und Strings
  sind keine SQL-Operationen; unbekannte Nachbaranweisung bleibt unbekannt.
- Fehlende Objekte, Ressourcenlimits und Timeout als Fehler, niemals als Teilerfolg.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `risk-analyzer BASE..TARGET` MUSS zwei lokal verfügbare Revisionsstände
  vergleichen; `HEAD~1..HEAD` und Aufruf im Repository-Unterverzeichnis sind unterstützt.
- **FR-002**: Analyse MUSS lokal, deterministisch und offline erfolgen; keine LLM-/Cloud-
  API, keine Repositorymutation, kein Checkout/Fetch/Hook/Textconv/External-Diff/Testlauf.
- **FR-003**: Vollständig bewertbare Änderungen MÜSSEN genau LOW/MEDIUM/HIGH liefern;
  höchste Signalstufe gewinnt. Leerer Diff MUSS ausdrücklich als LOW/keine Änderungen erscheinen.
- **FR-004**: `Risk:` und `Reasons:` MÜSSEN Klasse, Signal, Rationale und Datei enthalten;
  Inhaltsbefunde benötigen Fundstelle oder eindeutige Änderungsbeschreibung. Löschungen
  beziehen sich auf Basisstand. Sortierung/Deduplizierung MUSS reproduzierbar sein.
- **FR-005**: Die folgenden 14 fachlichen Signale MÜSSEN als fester versionierter Katalog
  mit definierten Indizien, Ausschlüssen und positiven/negativen Referenzfällen gelten.

| Level | Signal | Rationale |
| --- | --- | --- |
| LOW | Dokumentation, Kommentare, Formatierung | Keine relevante Runtime-Änderung im erkannten Umfang |
| LOW | Lokales Rename/Refactoring ohne Semantikänderung | Begrenzter Scope, bestehende Verträge bleiben erhalten |
| LOW | Tests-only | Produktionscode unverändert; Tests geändert |
| LOW | Unkritische UI-/Styling-Änderung | Lokaler visueller Effekt ohne Daten-/Sicherheitswirkung |
| MEDIUM | Neue Business-Logik | Neues Verhalten und Edge Cases |
| MEDIUM | API-Contract erweitert | Kompatibilität bestehender Consumer prüfen |
| MEDIUM | Dependency-Update | Indirekte/transitive Änderungen möglich |
| MEDIUM | DB-Query-/Performance-Änderung | Verhalten, Last oder Laufzeit beeinflusst |
| MEDIUM | Concurrency-/Caching-Änderung | Schwer reproduzierbare Fehler möglich |
| HIGH | Authorization/AuthN verändert | Security-Auswirkung und großer Blast Radius |
| HIGH | Destruktive DB-Migration | Datenverlust/schwere Reversibilität möglich |
| HIGH | Payment/Billing verändert | Finanzielle Auswirkungen möglich |
| HIGH | Secrets/IAM/Production Infrastructure | Sicherheits-/Verfügbarkeitswirkung |
| HIGH | Zentraler Domain-/API-Vertrag gebrochen | Viele Consumer potenziell betroffen |

- **FR-006**: Pfad-/Textheuristiken MÜSSEN sprachübergreifend sein. Kein bewiesener
  Vertragsbruch, keine Semantikgleichheit oder Sicherheitsgarantie darf behauptet werden.
  Unveränderter Kontext und reine Kommentarnennungen sind keine Inhaltsänderungsbefunde.
- **FR-007**: Ungültige Eingaben MÜSSEN Exit 2, Verarbeitungs-/nicht bewertbare Änderungen
  Exit 1 liefern, jeweils stdout leer. Erfolgreiches HIGH MUSS Exit 0 liefern.
- **FR-008**: Jede substanzielle Änderungsregion MUSS im vereinbarten Regelumfang bewertet
  werden; fehlende Regel oder unbekannte Nachbarregion darf nicht durch HIGH verdeckt werden.
- **FR-009**: Dokumentation MUSS Aufruf, Grenzen, Klassenbildung und Fehler erklären.
  Geheimniswerte dürfen nicht in Gründen/Fehlern erscheinen.
- **FR-010**: Tests MÜSSEN von Produktionsänderungen getrennt klassifiziert werden;
  ihre bloße Anwesenheit darf keine Risikoklasse senken. Symbolbezogene Testhinweise
  werden ausschließlich durch Feature 003 hinzugefügt und blockieren diesen Kernabschluss nicht.

### Key Entities *(include if feature involves data)*

- Revisionsvergleich: Repository und lokale Basis-/Zielrevision.
- Dateiänderung: Pfade, Typ und geänderte Inhalte in beiden Ständen.
- Risikoregel/-befund: Stufe, Indiz, Rationale und Fundstelle.
- Analyseergebnis/-fehler: vollständige geordnete Klasse/Gründe oder expliziter Fehler.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Mit einem Aufruf erhalten Entwickler Klasse und konkrete Gründe für
  unterstützte Vergleiche ohne externe Datenübertragung.
- **SC-002**: Alle 14 positiven/negativen Referenzpaare liefern die vereinbarten Befunde.
- **SC-003**: Zehn Wiederholungen aller Referenzvergleiche liefern identische Ausgabe.
- **SC-004**: Jeder Referenzbefund lässt sich einer Datei und Änderung eindeutig zuordnen.
- **SC-005**: Alle vereinbarten Fehler-/Limit-/Read-only-Fälle liefern den definierten
  Status ohne falsche Risk-Ausgabe und ohne Repositorymutation.

## Assumptions

- Eigenständiger Abschluss auf dem Entwicklungsziel; Vier-Plattform-Auslieferung gehört 004.
- Kein Staging-/Worktree-Modus, Dreipunktvergleich, Score, Webdienst, JSON-Report oder
  konfigurierbare Regelplugins. CLI-Ausgabe bleibt Englisch.
- Die Erkennungsgrenzen und Beispiele werden in contracts/rules.md präzisiert; bestehende
  Nutzerentscheidungen aus 001 gelten unverändert.
- Migration der Anforderungen siehe [../feature-split.md](../feature-split.md).
