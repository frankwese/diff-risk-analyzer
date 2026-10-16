# Feature Specification: Git Diff Risk Analyzer MVP

**Feature Branch**: `main` (bestehender Branch; kein Feature-Branch angelegt)

**Created**: 2026-10-13

**Status**: Superseded — aufgeteilt in aktive Features 002, 003 und 004

**Migration**: [Zuordnung und aktive Features](../feature-split.md).
Diese Gesamt-Spec bleibt historische Quelle; Weiterarbeit ausschließlich in den neuen
Feature-Artefakten. Der ursprüngliche Produktumfang wird vollständig übernommen.

**Input**: Lokales Kommandozeilenwerkzeug für Softwareentwickler vor dem Push oder im
Code-Review. Ein Git-Diff wird anhand deterministischer Heuristiken als LOW, MEDIUM oder
HIGH bewertet und mit konkreten Begründungen ausgegeben. Keine LLM- oder Cloud-Nutzung.
Die vom Nutzer vorgegebene Signaltabelle definiert die fachlichen Risikostufen.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Änderungen vor dem Push einschätzen (Priority: P1)

Als Entwickler möchte ich zwei lokale Revisionen vergleichen und eine begründete
Risikoklasse erhalten, damit ich kritische Änderungen vor dem Push gezielt prüfen kann.

**Why this priority**: Die begründete Bewertung ist der zentrale Produktnutzen.

**Independent Test**: Ein lokales Repository mit bekannten Änderungen wird über
`risk-analyzer HEAD~1..HEAD` analysiert; Klasse und Gründe werden mit den erwarteten
Signalen verglichen.

**Acceptance Scenarios**:

1. **Given** ein Diff ausschließlich mit Dokumentationsänderungen, **When** die Analyse
   aufgerufen wird, **Then** lautet das Ergebnis LOW mit einem Dokumentationsgrund.
2. **Given** ein Diff mit einer erkannten Dependency-Aktualisierung, **When** die Analyse
   aufgerufen wird, **Then** lautet das Ergebnis MEDIUM mit betroffener Datei und Grund.
3. **Given** ein Diff mit erkannter Authorization-Änderung und destruktiver Migration,
   **When** die Analyse aufgerufen wird, **Then** lautet das Ergebnis HIGH und beide
   Signale werden mit ihren jeweiligen Fundstellen genannt.
4. **Given** LOW-, MEDIUM- und HIGH-Signale im selben Diff, **When** die Analyse ausgeführt
   wird, **Then** bestimmt HIGH die Gesamtrisikoklasse und alle erkannten Signale bleiben
   in den Gründen sichtbar.
5. **Given** unveränderte Revisionen und Regeln, **When** die Analyse wiederholt wird,
   **Then** sind Klasse, Gründe und deren Reihenfolge identisch.

### User Story 2 - Review-Schwerpunkte nachvollziehen (Priority: P2)

Als Reviewer möchte ich erkennen, welche Änderung ein Signal ausgelöst hat und ob im
Vergleich passende Teständerungen enthalten sind, um meine Prüfung gezielt auszurichten.

**Why this priority**: Konkrete Fundstellen machen die Bewertung überprüfbar und verhindern,
dass eine pauschale Risikoklasse den Review ersetzt.

**Independent Test**: Ein Diff mit einem bekannten Risikosignal wird mit und ohne
zugeordnete Teständerung analysiert; Gründe und Testhinweise werden getrennt verglichen.

**Acceptance Scenarios**:

1. **Given** ein erkanntes Risiko in Produktionscode ohne zugeordnete Teständerung,
   **When** die Analyse ausgeführt wird, **Then** nennt sie das Risikosignal und weist
   auf die fehlende, heuristisch zugeordnete Teständerung hin.
2. **Given** dieselbe Risikoänderung mit zugeordneter Teständerung, **When** die Analyse
   ausgeführt wird, **Then** entfällt der Hinweis auf fehlende Teständerungen; die
   Risikoklasse des Signals bleibt bestehen.
3. **Given** vorhandene Testdateien ohne Änderung im betrachteten Diff, **When** die
   Analyse ausgeführt wird, **Then** behauptet sie weder fehlende bestehende Tests noch
   nachgewiesene Testabdeckung; sie bewertet nur die Teständerungen im Diff.
4. **Given** ein reiner Tests-Diff ohne höheres Signal, **When** die Analyse ausgeführt
   wird, **Then** lautet das Ergebnis LOW mit dem Grund Tests-only.
5. **Given** eine geänderte Methode und eine namens-/pfadmäßig passende Testdatei, deren
   geänderter Test nur eine andere Methode verwendet, **When** die Analyse ausgeführt wird,
   **Then** bleibt der Hinweis auf fehlende zugehörige Teständerungen für die geänderte Methode.
6. **Given** eine geänderte Methode und ein hinzugefügter oder geänderter Test, der sie an
   einer eindeutig zugeordneten Instanz der betroffenen Klasse aufruft, **When** die Analyse
   ausgeführt wird, **Then** wird die Testnutzung mit Fundstelle genannt und der Fehlhinweis
   für diese Methode entfällt.
7. **Given** zwei geänderte Methoden und eine Teständerung, die nur eine davon verwendet,
   **When** die Analyse ausgeführt wird, **Then** bleibt der Fehlhinweis für die andere Methode.
8. **Given** gleichnamige Methoden in mehreren Klassen oder eine nur indirekte Nutzung,
   die nicht eindeutig zugeordnet werden kann, **When** die Analyse ausgeführt wird,
   **Then** wird die Zuordnung als unklar gemeldet, statt passende Tests zu behaupten.

### User Story 3 - Grenzen und Fehler erkennen (Priority: P2)

Als Entwickler möchte ich ungültige Vergleiche und nicht bewertbare Änderungen von einem
erfolgreichen Ergebnis unterscheiden können, damit LOW nicht als falsche Entwarnung dient.

**Why this priority**: Die Aussagekraft des Ergebnisses hängt von einer ehrlichen Angabe
der Analysegrenzen ab.

**Independent Test**: Aufrufe außerhalb eines Repositorys, mit ungültigen Revisionen und
mit nicht unterstützten Änderungen liefern jeweils einen erkennbaren Fehlerzustand.

**Acceptance Scenarios**:

1. **Given** eine unbekannte Revision oder kein lokales Repository, **When** die Analyse
   aufgerufen wird, **Then** endet sie mit einem Fehler und ohne erfolgreiche Risikoklasse.
2. **Given** ein vollständig leerer Diff, **When** die Analyse aufgerufen wird, **Then**
   lautet das Ergebnis LOW mit dem ausdrücklichen Hinweis, dass keine Änderungen vorliegen.
3. **Given** eine Änderung, die von den unterstützten Regeln nicht bewertet werden kann,
   **When** die Analyse aufgerufen wird, **Then** wird die betroffene Datei genannt und
   der Aufruf endet ohne abschließende Gesamtrisikoklasse mit einem Fehlerstatus.
4. **Given** ein Aufruf ohne Netzwerkzugang, **When** lokale Revisionen verfügbar sind,
   **Then** funktioniert die Analyse und verändert weder Repository noch Arbeitsdateien.
5. **Given** ein unterstütztes Nutzersystem mit Git und dem passenden ausgelieferten
   Executable, aber ohne installierte Sprachlaufzeit oder Entwicklungswerkzeuge,
   **When** ein unterstützter Vergleich analysiert wird, **Then** funktioniert die Analyse
   einschließlich aller vier Testzuordnungsprofile ohne Nachladen weiterer Komponenten.

### Edge Cases

- Fehlendes Argument, ungültiger Revisionsvergleich, fehlendes Git oder Lesefehler:
  verständlicher Fehler, keine erfolgreiche Risikoklasse.
- Nicht eingecheckte Änderungen: werden nicht in einen expliziten Revisionsvergleich einbezogen.
- Neue und gelöschte Dateien: beide Änderungsarten werden berücksichtigt; gelöschte Tests
  gelten nicht als hinzugefügte oder aktualisierte Testabdeckung.
- Reine Dateiumbenennung ohne Inhaltsänderung: LOW, sofern kein höheres Signal greift;
  inhaltliche Refactorings gelten nicht allein wegen ihrer Bezeichnung als semantikneutral.
- Binäränderung oder unbekanntes Dateiformat ohne unterstützte Bewertungsregel:
  explizit nicht bewertbar, keine automatische LOW-Einstufung.
- Pfade mit Leerzeichen oder Sonderzeichen: bleiben in Gründen eindeutig zuordenbar.
- Auth-Begriffe in Dokumentation oder Tests: lösen allein kein Produktionscode-Signal aus.
- Nichtdestruktive Migration: löst nicht allein aufgrund ihres Migrationstyps HIGH aus;
  ihr Inhalt wird anhand der übrigen Regeln bewertet.
- Mehrere Treffer desselben Signals: alle betroffenen Fundstellen bleiben nachvollziehbar;
  doppelte identische Gründe werden nicht mehrfach ausgegeben.
- Klassen-/Methodennamen ausschließlich in Kommentaren, Zeichenketten, Testnamen oder
  Importen: gelten nicht als Nutzung. Ein Import kann die Zuordnung unterstützen, reicht
  aber allein nicht aus.
- Ein unveränderter Test, der die geänderte Methode schon verwendet: zählt nicht als
  Teständerung. Bei einem geänderten Test darf die Verwendung in unveränderten Zeilen
  desselben Testfalls stehen; eine Änderung in einem anderen Testfall genügt nicht.
- Gelöschte Methoden/Klassen sowie reine Dateiänderungen ohne erkennbare Symbole:
  symbolbezogene Testzuordnung wird als nicht anwendbar beziehungsweise unklar ausgewiesen.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Das Werkzeug MUSS als `risk-analyzer <base>..<target>` zwei lokal verfügbare
  Git-Revisionen im aktuellen Repository oder einem seiner Unterverzeichnisse vergleichen.
  `HEAD~1..HEAD` MUSS unterstützt werden. Der Vergleich bezieht sich auf die beiden
  Revisionsstände; nicht auf die Liste dazwischenliegender Commits.
- **FR-002**: Das Werkzeug MUSS ausschließlich lokal und ohne Netzwerkzugriffe arbeiten.
  Es DARF keine LLMs, Cloud-Dienste oder externen Analyse-APIs verwenden und DARF keine
  Repository-Inhalte, Revisionen, Indexeinträge oder Arbeitsdateien verändern.
- **FR-003**: Für einen vollständig bewertbaren Diff MUSS genau eine Gesamtrisikoklasse
  LOW, MEDIUM oder HIGH ausgegeben werden. Das höchste erkannte Signal bestimmt die
  Klasse; ein leerer Diff wird als LOW mit explizitem Leerhinweis ausgegeben.
- **FR-004**: Die Ausgabe MUSS mit `Risk: <CLASS>` beginnen und anschließend unter
  `Reasons:` die konkreten Gründe nennen. Jeder Signalgrund MUSS Signal, fachliche
  Begründung und betroffene Datei enthalten, bei inhaltsbezogenen Treffern zusätzlich
  eine Fundstelle oder eine eindeutige Beschreibung der auslösenden Änderung.
- **FR-005**: Die folgende vom Nutzer vorgegebene Signaltabelle MUSS fachlich abgebildet
  werden. Ein erkanntes niedriges Signal DARF ein höheres Signal nicht unterdrücken.

| Level | Signal | Rationale |
| --- | --- | --- |
| LOW | Dokumentation, Kommentare, Formatierung | Keine relevante Änderung am Runtime-Verhalten. |
| LOW | Lokales Rename/Refactoring ohne Semantikänderung | Begrenzter Scope; bestehende Verträge bleiben erhalten. |
| LOW | Tests-only Änderung | Produktionscode bleibt unverändert; Testabdeckung wird erweitert. |
| LOW | Unkritische UI-/Styling-Änderung | Lokaler visueller Effekt ohne Daten- oder Sicherheitsauswirkung. |
| MEDIUM | Neue Business-Logik | Neues Verhalten kann Fehler oder unerwartete Edge Cases einführen. |
| MEDIUM | API-Contract erweitert | Bestehende Consumer könnten betroffen sein; Kompatibilität prüfen. |
| MEDIUM | Dependency-Update | Indirekte Breaking Changes oder transitive Änderungen möglich. |
| MEDIUM | DB-Query-/Performance-Änderung | Kann Verhalten, Last oder Laufzeit beeinflussen. |
| MEDIUM | Concurrency-/Caching-Änderung | Fehler sind oft schwer reproduzierbar. |
| HIGH | Authorization/AuthN verändert | Direkte Security-Auswirkung und potenziell großer Blast Radius. |
| HIGH | Destruktive DB-Migration | Potenzieller Datenverlust bzw. schwer reversibel. |
| HIGH | Payment/Billing verändert | Direkte finanzielle Auswirkungen möglich. |
| HIGH | Secrets/IAM/Production Infrastructure | Sicherheits- oder Verfügbarkeitsauswirkung. |
| HIGH | Zentraler Domain-/API-Vertrag gebrochen | Viele Consumer können gleichzeitig betroffen sein. |

- **FR-006**: Für jedes Signal MÜSSEN unterstützte Eingaben, auslösende Indizien und
  Ausschlüsse dokumentiert und mit positiven sowie negativen Beispielen überprüfbar sein.
  Der MVP MUSS sprachübergreifende Pfad- und Textheuristiken verwenden; ein einzelnes
  Sprachprofil DARF keine Voraussetzung für die Risikoanalyse sein. Fachliche Indizien
  sind in der folgenden Tabelle festgelegt. Die konkreten Muster und Ausschlüsse MÜSSEN
  im Plan als endlicher, versionierter Regelkatalog mit Referenzbeispielen festgelegt werden.
- **FR-007**: Die Bewertung MUSS als heuristische Review-Hilfe erkennbar sein. Sie DARF
  keine nachgewiesene Semantikgleichheit, Fehlerfreiheit, Vertragskompatibilität oder
  Sicherheitsgarantie behaupten. Begriffe wie „Vertrag gebrochen“ MÜSSEN in der Ausgabe
  als erkanntes Indiz mit Fundstelle statt als bewiesene Wirkung formuliert werden.
- **FR-008**: Für Produktionscode-Änderungen MUSS das Werkzeug einen getrennten Hinweis
  ausgeben, wenn keine zugehörigen hinzugefügten oder aktualisierten Tests im Diff erkannt
  werden. Bei erkennbaren geänderten Klassen und Methoden MUSS die Zuordnung pro Symbol
  erfolgen: Ein hinzugefügter oder geänderter Testfall MUSS die geänderte Klasse verwenden
  und bei einer Methodenänderung die betroffene Methode aufrufen. Namen und Pfade dienen
  der Kandidatenzuordnung, ersetzen aber keinen Nutzungsbefund. Eindeutige direkte
  Symbolnutzung darf auch ohne passenden Testdateinamen als zugehörig erkannt werden.
  Ein gleichnamiger Aufruf auf einer nicht zugeordneten Klasse reicht nicht aus.
- **FR-009**: Teständerungen MÜSSEN getrennt von Produktionscode bewertet werden. Sie
  DÜRFEN allein keine Änderung an produktiver Auth-, Zahlungs- oder Infrastruktur-Logik
  suggerieren. Vorhandene Teständerungen DÜRFEN die Signalklasse nicht herabsetzen;
  fehlende Teständerungen erhöhen die Klasse in der Erstfassung nicht automatisch.
- **FR-010**: Gleiche Eingaben mit demselben Regelstand MÜSSEN identische Klasse, Gründe
  und Reihenfolge liefern. Gründe MÜSSEN stabil nach Risikostufe (HIGH zuerst), Signal
  und Dateipfad/Fundstelle geordnet werden.
- **FR-011**: Erfolgreiche Analysen MÜSSEN unabhängig von ihrer Risikoklasse mit Exitstatus
  0 enden. Ungültige Eingaben, Verarbeitungsfehler und nicht vollständig bewertbare Diffs
  MÜSSEN mit einem von 0 verschiedenen Status und Fehlermeldung enden; Fehler gehören
  in die Fehlerausgabe, erfolgreiche Ergebnisse in die Standardausgabe.
- **FR-012**: Nicht bewertbare Änderungen MÜSSEN ausdrücklich mit ihren Dateien genannt
  werden. Eine fehlende Regel DARF NICHT als bestätigte LOW-Änderung behandelt werden.
- **FR-013**: Die Benutzerdokumentation MUSS Aufruf, unterstützte Regeln, Klassenbildung,
  Testzuordnung, Fehlerverhalten und bekannte Erkennungsgrenzen mit Beispielen erklären.
  Geheimniswerte DÜRFEN in Gründen oder Fehlerausgaben nicht wiedergegeben werden.
- **FR-014**: Die Testzuordnung MUSS geänderte Testfälle und deren Inhalte im Zielstand
  berücksichtigen, nicht nur hinzugefügte Diff-Zeilen. Unveränderte oder gelöschte Testfälle
  zählen nicht. Kommentare, Zeichenketten, Imports und bloße Namensähnlichkeit DÜRFEN
  NICHT als Nutzung gewertet werden. Für jeden Nutzungsbefund MÜSSEN Testdatei, Testfall,
  Produktionssymbol und Fundstelle nachvollziehbar angegeben werden.
- **FR-015**: Fehlende, unklare und nicht anwendbare symbolbezogene Testzuordnung MÜSSEN
  getrennt benannt werden. Nicht auflösbare indirekte Aufrufe, dynamische Nutzung und
  mehrdeutige Symbolnamen MÜSSEN als unklar gelten. Diese Grenze der Testzuordnung allein
  DARF eine sonst bewertbare Risikoanalyse nicht zum Fehler machen. Die Analyse DARF
  aus einer erkannten Nutzung weder ausgeführte Tests noch wirksame Assertions oder
  vollständige Testabdeckung ableiten.
- **FR-016**: Das Werkzeug MUSS als natives Standalone-Executable für macOS und Linux,
  jeweils x86_64 und arm64, ausgeliefert werden. Auf dem Nutzersystem DARF keine separate
  Sprachlaufzeit, kein Compiler oder Paketmanager nötig sein. Parser und feste Regeln
  MÜSSEN mitgeliefert sein; Git bleibt eine externe Voraussetzung. Die Auslieferung
  MUSS ohne erstmaliges Herunterladen von Analysekomponenten funktionieren.

### Heuristische Erkennungsbedingungen

Pfadbefunde beziehen sich auf die geänderte Datei; Inhaltsbefunde auf hinzugefügte oder
gelöschte Inhalte. Unveränderte Begriffe im Kontext allein lösen kein Inhaltssignal aus.
Dokumentation und Tests werden vor Produktionscode-Signalen abgegrenzt. LOW für
Semantikneutralität ist nur bei einem konkret unterstützten Neutralitätsmuster zulässig.

| Signal | Positives Referenzindiz | Negativbeispiel / Grenze |
| --- | --- | --- |
| Dokumentation, Kommentare, Formatierung | Dokumentationsdatei geändert oder ausschließlich erkannte Kommentare/Leerraum geändert | Eine ausführbare Anweisung neben einer Kommentaränderung ist nicht ausschließlich LOW. |
| Lokales Rename/Refactoring ohne Semantikänderung | Reine Dateiumbenennung bei unverändertem Inhalt oder erkanntes lokales Neutralitätsmuster | Eine geänderte öffentliche Signatur ist kein neutrales Rename. Unbekannte Refactorings werden nicht als neutral angenommen. |
| Tests-only Änderung | Ausschließlich erkannte Testdateien geändert | Eine zusätzliche Produktionscode-Änderung hebt Tests-only auf; Assertions und tatsächliche Abdeckung werden nicht bewiesen. |
| Unkritische UI-/Styling-Änderung | Nur lokale visuelle Eigenschaften wie Farbe oder Abstand geändert | Änderungen an Sichtbarkeit von Zugriffskontrollen, Datenverarbeitung oder ausführbarer UI-Logik sind nicht allein visuell. |
| Neue Business-Logik | Neue ausführbare Entscheidung, Berechnung oder Verhaltensregel im Produktionscode | Derselbe Text in Dokumentation oder Tests löst das Produktionssignal nicht aus. |
| API-Contract erweitert | Erkennbar neues öffentliches Vertragselement bei Erhalt bestehender Elemente | Entfernte oder inkompatibel geänderte Elemente gelten nicht als reine Erweiterung. |
| Dependency-Update | Abhängigkeit oder Version in erkanntem Manifest beziehungsweise Lockfile geändert | Die Erwähnung einer Version in Dokumentation ist kein Update. |
| DB-Query-/Performance-Änderung | Abfrage, Index oder erkennbar laufzeitrelevante Operation geändert | Eine Abfrage in einer Dokumentationsdatei zählt nicht als produktive Abfrageänderung. |
| Concurrency-/Caching-Änderung | Sperren, parallele Verarbeitung, Cache-Zugriff oder Invalidierung geändert | Ein unveränderter Cache-Begriff im Kontext löst keinen Inhaltsbefund aus. |
| Authorization/AuthN verändert | Zugriffskontrolle, Identitätsprüfung oder produktiver Auth-Bereich geändert | Auth-Begriffe nur in Dokumentation oder Tests genügen nicht. |
| Destruktive DB-Migration | Daten-/Schemaentfernung wie DROP oder TRUNCATE in einer erkannten Migration geändert | Eine rein additive Migration löst dieses HIGH-Signal nicht aus. |
| Payment/Billing verändert | Zahlungs-, Abrechnungs- oder produktive Preisberechnungslogik geändert | Ein Preisbeispiel in Dokumentation ist keine Zahlungslogik. |
| Secrets/IAM/Production Infrastructure | Geheimnisverwaltung, Berechtigungsregel oder erkennbar produktive Infrastruktur geändert | Eine ausdrücklich lokale Entwicklungsumgebung ist nicht allein produktive Infrastruktur. |
| Zentraler Domain-/API-Vertrag gebrochen | Entferntes erforderliches Vertragsfeld oder inkompatibel geänderte öffentliche Signatur in erkanntem zentralem Vertrag | Ein neues optionales Feld ist keine nachgewiesene Inkompatibilität. Zentralität und Bruch bleiben heuristische Indizien. |

### Key Entities *(include if feature involves data)*

- **Revisionsvergleich**: Repository sowie Basis- und Zielrevision, die den Diff bestimmen.
- **Dateiänderung**: Alter/neuer Pfad, Änderungsart und verfügbare hinzugefügte/gelöschte Inhalte.
- **Risikoregel**: Signalname, Stufe, Rationale, Erkennungsbedingungen und Ausschlüsse.
- **Signalbefund**: Getroffene Regel mit Datei, Fundstelle und konkretem Änderungsindiz.
- **Analyseergebnis**: Gesamtrisikoklasse, geordnete Gründe und separate Testhinweise;
  eine vollständige Bewertung ist Voraussetzung für eine Gesamtrisikoklasse.
- **Testzuordnung**: Heuristische Beziehung einer Teständerung zu Produktionsänderungen;
  geänderte Klasse/Methode, geänderter Testfall, Nutzungsfundstelle und Status
  (Nutzung erkannt, Nutzung fehlt, unklar oder nicht anwendbar); kein Nachweis tatsächlicher
  Testabdeckung oder erfolgreicher Testausführung.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Ein Entwickler erhält für einen unterstützten lokalen Vergleich mit einem
  Aufruf eine Klasse und konkrete Gründe, ohne Daten an externe Dienste zu übertragen.
- **SC-002**: Jede der 14 Signalkategorien besitzt mindestens ein abgestimmtes positives
  und ein negatives Beispiel; alle Beispiele liefern die vereinbarten Signalbefunde und
  Gesamtrisikoklassen.
- **SC-003**: Zehn Wiederholungen jedes abgestimmten Beispiels liefern jeweils dieselbe
  Klasse und dieselben geordneten Gründe.
- **SC-004**: Jeder Signalgrund der abgestimmten Beispiele lässt sich eindeutig einer
  betroffenen Datei und einer auslösenden Änderung zuordnen.
- **SC-005**: Alle vereinbarten Beispiele für ungültige oder nicht bewertbare Eingaben
  liefern einen erkennbaren Fehler ohne erfolgreiche Gesamtrisikoklasse.
- **SC-006**: Beispiele mit derselben Produktionsänderung und variierenden Teständerungen
  liefern die vereinbarten Testhinweise, ohne die zugrunde liegende Signalklasse abzusenken.
- **SC-007**: Referenzfälle für direkte Nutzung, passenden Namen ohne Nutzung, falsche
  gleichnamige Methode, unveränderten Testfall, teilweise Zuordnung und indirekte Nutzung
  liefern jeweils den vereinbarten symbolbezogenen Status und nachvollziehbare Fundstellen.
- **SC-008**: Auf jedem der vier unterstützten Plattform-/Architekturziele liefert das
  Release-Executable die Referenzfälle ohne separate Sprachlaufzeit und offline korrekt,
  einschließlich je eines direkten Testnutzungsbefunds für Java, JS, TS und Python.

## Assumptions

- Zielgruppe sind Softwareentwickler mit lokalem Git-Repository und verfügbaren Revisionen.
- Der MVP analysiert explizite Zwei-Revisions-Vergleiche. Arbeitsverzeichnis-/Staging-Modus,
  Dreipunkt-Vergleiche, automatische Git-Hooks, CI-Integration, Weboberfläche, gespeicherte
  Reports, JSON-Ausgabe und benutzerdefinierte Regelkonfiguration gehören nicht zum MVP.
- Englisch ist die Sprache der CLI-Ausgabe, entsprechend dem Nutzerbeispiel.
- Die höchste Signalstufe bestimmt die Gesamtrisikoklasse; ein kumulativer Score und
  zusätzliche Eskalation wegen fehlender Teständerungen sind nicht vorgesehen.
- Die Tabelle ersetzt die pauschale Beispielaussage „A database migration is included“:
  HIGH gilt für eine als destruktiv erkannte Migration, nicht für jede Migration.
- LOW bedeutet lediglich ein niedriges erkanntes Änderungsrisiko im unterstützten
  Regelumfang. Eine Ausführung der Anwendung oder ihrer Tests gehört nicht zur Analyse.
- Technologiewahl und interne Umsetzung werden im anschließenden Plan entschieden.
  Das Erkennungsprofil ist sprachübergreifend und mustergebunden. Unterstützte Muster
  zur Klassen-, Methoden- und Testfallerkennung werden im Plan ausdrücklich aufgelistet;
  nicht erkannte Strukturen werden nicht als erfolgreiche Testzuordnung behandelt.
- Nutzerklärung vom 2026-10-13: Erkennung nach Option 1A; Testnamen-/Pfadzuordnung nach
  Option 2B mit zusätzlichem zwingendem Nutzungsbefund für geänderte Klassen und Methoden.
- Nutzerentscheidung vom 2026-10-13: natives Standalone-Tool; technische Umsetzung in
  Rust wird im Plan festgelegt. Python bleibt unterstützte analysierte Quellsprache.
