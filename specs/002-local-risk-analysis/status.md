# Status: Lokale Risikobewertung

**Stand**: Dokumentenmigration am 2026-10-13, Basiscommit eb5de1e.
**Status**: Bestätigte Auth-Kontextkorrektur T026 umgesetzt und gezielt validiert.

## Zweite Konvergenzkorrektur T033–T036

T033: Geordneter Myers-Abgleich je Regionsherkunft statt Multiset; Zweig-/Schleifen-
zugehörigkeit Teil der Herkunft. Verschobene Auth-Prüfung oder getauschte Operations-
reihenfolge bleibt eine Änderung. T034: Deklarationsheader/Signatur Teil der Herkunft;
Auth-Gegenparts benötigen dieselbe Signatur statt beliebiger gleichnamiger Überladung.

T035: Interner CLI-Einstieg erfordert Supervisor-PID als unmittelbaren Elternprozess
und die eigene Prozessgruppenführerschaft. Normaler direkter Aufruf wird Exit 2 und
stdout leer; Supervisor startet Worker passend. CPU-Endlosschleife im kontrollierten
Prozessdummy wird nach 1s beendet und eingesammelt. Keine Nutzeroption zum Budget-Bypass.

T036: Eigener literal-/kommentarbewusster Gradle-Tokenstrom; genau abgegrenzte top-level
dependencies-Blöcke inklusive verschachtelter Konfigurationsblöcke. Nachbareinstellungen
derselben Zeile bleiben im separaten Restvergleich. Mehrdeutige verschachtelte
dependencies-Blöcke abgelehnt; Klammern in Strings/Kommentaren verfälschen Grenzen nicht.

Finale lokale Validierung: cargo test --locked erfolgreich mit 22 Unit-, 20 bestehenden
Zuordnungskompatibilitäts-, 10 CLI-, 12 Git- und 32 Regeltests (96 Testfunktionen plus
Datenfälle). Clippy -D warnings, Whitespaceprüfung und IDE-Build erfolgreich.
T033–T036 erledigt; alle 36 derzeitigen 002-Aufgaben abgearbeitet. Keine Commits/Docker-
oder Releasebuilds. Erneute Konvergenzprüfung ist nicht durch diese Umsetzung ersetzt.
Der ursprüngliche Abschluss hatte diesen Kontext nicht ausreichend abgesichert.

## Konvergenzkorrekturen T027–T032 umgesetzt

- T027: JSON/YAML-Infrastruktur wird strukturell mit erhaltenen Werten verglichen;
  Terraform-artige Texte über literal-erhaltende Kommentarabgrenzung. Geänderte IAM-
  Action/Resource/Effect/Principal- und Produktionswerte erzeugen HIGH statt falschem LOW.
- T028: SourceRegion besitzt die Funktion-/Klassenherkunft. Identische Returns in anderen
  Funktionen gleichen die Änderung nicht mehr aus; getauschte Returns erhalten Auth-Kontext.
- T029: Auth-Kontext bewertet auch neue Logik ohne anderes Signal innerhalb derselben
  Funktion als HIGH. Die bisherige gegenteilige Regression wurde der bestätigten neuen
  Anforderung angepasst; unbekannte Logik in anderen Funktionen bleibt unsupported.
- T030: quick-xml 0.38.3 und toml 0.9.8 ersetzen fehleranfällige Marker für Maven/TOML;
  XML-Metadaten außerhalb Dependencies bleiben getrennt, unvollständige XML/TOML/Gradle-
  Blöcke werden abgelehnt. Öffentliche Java-Interface-/abstrakte Methodendeklarationen
  ohne Körper werden für API-Erweiterung/-Bruch mit erfasst.
- T031: Analyseprozess statt abgetrenntem Thread. Supervisor besitzt eine eigene
  Prozessgruppe und beendet sie bei Gesamttimeout samt Git-Kindprozessen; Git im
  Worker bleibt in derselben Gruppe. Ausgabe wird erst nach Prozessabschluss ausgegeben.
- T032: Echter CLI-Revisionsvergleich für Auth-Return, Kommentar und fremde Funktion;
  jedes Ergebnis zehnmal byteidentisch mit Rules 1.1.0. Hilfe-/Fehler-/Kernverträge erhalten.

Finaler lokaler Lauf: cargo test --locked erfolgreich mit 21 Unit-, 12 Git-, 9 CLI-,
30 Regel- und 20 Zuordnungskompatibilitätstests, insgesamt 92 Testfunktionen plus
datengetriebene Fälle. Clippy erfolgreich vor dem letzten reinen Erwartungsupdate des
Kompatibilitätstests. Format-/IDE-Abschlussprüfung nachfolgend dokumentiert.
Keine Docker-/Vierzielreleaseprüfung, keine Commits. Alle 32 Aufgaben wurden bearbeitet;
eine erneute unabhängige Konvergenzprüfung ist vor endgültiger Freigabe erforderlich.

Abschluss dieser Runde: fmt --check, Clippy -D warnings, IDE-Build und Whitespaceprüfung
erfolgreich. Aktueller Debug-Smoke: 6.68s real, 6.70s Wandzeit, max RSS 5.079.040 Bytes;
interner 5s-Wunsch weiterhin nicht erreicht, kein zusätzlicher Produkt-Akzeptanzfehler.
T027–T032 als erledigt markiert. Keine offenen Tasks in dieser Runde, aber keine neue
Converged-Aussage ohne anschließende Konvergenzprüfung.

## Auth-Kontextkorrektur T026

Regelstand 1.1.0: AUTH_CONTEXT_CHANGE als eigenständiger HIGH-Grund innerhalb der
Auth-Kategorie. Auth-Pfad, exakter Auth-Funktionsname oder tatsächlicher Auth-Aufruf
im selben Basis-/Zielsymbol begründet den Kontext für geänderte Logik/Verträge.
Unveränderte Prüfaufrufe sind keine direkt geänderten AUTH_CHANGE-Befunde.
Kommentare/Literale, andere Funktionen und verschachtelte Funktionen übertragen
keinen Kontext. Unbekannte Nachbaranweisungen bleiben weiterhin unsupported;
der Kontextbefund dient nicht zum Überdecken unbewerteter Änderungen.

25 Regeltests und Clippy erfolgreich; neue Fälle decken Java/JS/TS/Python-Returns,
Auth-Namen, Vertragsänderung, entfernte Prüfung, Kommentare/Strings, andere Funktionen
und verschachtelte Helper ab. FR-011/SC-006 damit gezielt nachgewiesen; README korrigiert.
Kein neuer Release-/Dockerlauf oder Commit. Ältere Abschlussnachweise beziehen sich
weiterhin auf ihren früheren Regelstand.

## Abschluss der verbleibenden Aufgaben T011–T025

Quellcodebewertung von der bisherigen Zeilenheuristik auf unabhängig verglichene
Anweisungs-/Deklarationsbereiche umgestellt. Erhalten bleiben die 14 fest vereinbarten
Signale und konservative unsupported-Behandlung. Wesentliche Korrekturen:
- Unveränderter Auth-Aufruf neben verändertem Return erzeugt keinen neuen Auth-Befund.
- Unbekannte Nachbaranweisung bleibt unsupported, auch neben einem HIGH-Signal oder
  einer Vertragsänderung. Deklarationshüllen erlauben keine unbewerteten Methodenrümpfe.
- Testpfad/-dateiname verdeckt keinen geänderten Produktionsrumpf oder Klassenfeld.
- Kritische Quellpfade verlangen lesbare Syntax; Kommentare/Strings allein lösen keine
  produktive Operation aus. Literalinterpolation bleibt Code. Identifierbestandteile
  werden nach CamelCase/underscore abgegrenzt.
- Manifest-Unterbereiche, mehrere CSS-Regeln und mehrzeilige SQL-Statements sind
  voneinander abgegrenzt; unbekannte Metadaten/Operationen werden nicht mitbewertet.
- OpenAPI-Erweiterung verdeckt keine unabhängige info-/Metadatenänderung; required-
  Erweiterungen/Signaturentfernung führen zu Vertragsbruchindizien. JSON/YAML-Duplikate,
  Tags/Aliase/Referenzen und komplexe Schemas bleiben ausdrücklich unsupported.
- Ausgabeformat, BASE/TARGET/METADATA-Fundstellen, escaping, stabile Reihenfolge und
  vollständige Deduplizierung besitzen zusätzliche Kernvertragstests.

## Finale lokale Nachweise

- Rust/Cargo 1.99.0, macOS arm64, Git 2.54.0.
- 20 Unit-Tests, 12 Git-Integrationstests, 8 CLI-Tests und 22 Regeltests erfolgreich;
  zusätzlich 20 bestehende Zuordnungstests als Kompatibilitätsprüfung erfolgreich.
- Alle 14 Signale haben positive/negative Referenzen (26 Datenfälle plus Renamepaar).
  Positive Referenzen zehnmal identisch; CLI-E2E zehnmal byteidentisch.
- Fehler-/Helfer-/Objekt-/SHA-256-/Dirty-Index-/Blob-/Datei-/Zeilen-/Deadlinefälle aus
  T004–T009 plus neue Regions-/Formatfälle vollständig für den vereinbarten Kernumfang.
- 100 Dateien, 10.000 geänderte Zeilen: 6.98s real / 7.01s Wandzeit im Debug-E2E,
  max RSS 5.079.040 Bytes; interner <=5s-Wunsch verfehlt, <=256 MiB deutlich eingehalten.
  Dies ist kein zusätzliches Produkt-Akzeptanzkriterium. Keine Performancegarantie.
- Formatprüfung/Clippy und IDE-Build werden im Abschluss geprüft; Ergebnis siehe
  nachfolgende Abschlussnotiz. Kein Release-/Dockerlauf, kein Commit oder Push.

**Abschlussnotiz**: cargo fmt --check, Clippy -D warnings, git diff --check und
IDE-Build erfolgreich. Alle T001–T025 sind im dokumentierten Kernumfang abgeschlossen.

## FR-/SC-Abgleich

| Kriterium | Nachweis |
| --- | --- |
| FR-001 / SC-001 | Lokaler Zwei-Revisions-/Unterverzeichnis-/SHA-256- und CLI-E2E |
| FR-002 / SC-005 | Read-only HEAD/Index/Worktree, no-helper, missing-object/no-fetch; Prozess-/Budgettests |
| FR-003 | LOW/MEDIUM/HIGH max-Level und leerer Vergleich |
| FR-004 / SC-003/004 | Fundstellen-/Format-/Deduplizierungs- und zehn Wiederholungsprüfungen |
| FR-005 / SC-002 | 14 positive/negative Referenzpaare |
| FR-006 | Endliche Regionen/Identifier, Kommentare/Literale und konservative Grenzen |
| FR-007 | Exit 0/1/2, stdout bei Fehler leer, statische stderr-Ausgabe |
| FR-008 | Unknown-neben-HIGH, unbekannte Nachbaroperation, API-/Manifestmetadaten |
| FR-009 | README/quickstart/Regelvertrag und Redaktions-/Secretprüfungen |
| FR-010 | Mixed-Scopes, Tests-only und unverändertes Risk; 003-Kompatibilität |

002-Abschluss ist keine vollständige Typ-/Semantik-/Coverageprüfung. Dokumentierte
endliche Muster begrenzen die Aussage; nicht bewertbare Eingaben sind erfolgreiche
Fehlerbehandlung. Offene Feature-003-Matrix und Vierziel-Auslieferung 004 bleiben separat.

## T009 abgeschlossen: Basisnachweise abgeglichen

Die Nachweise aus T004–T007 zusammengeführt, ohne bereits erfolgreiche Prüfungen
erneut vollständig auszuführen. Drei bisher fehlende Fälle ergänzt und erfolgreich geprüft:
- Echtes SHA-256-Repository: 64-stellige Commit-/Blob-OIDs, identische Ergebnisse vom
  Repositoryroot und aus einem verschachtelten Unterverzeichnis. Staged und unstaged
  Inhalte gelangen nicht in den Commitvergleich. HEAD, Indexbytes und Worktreeinhalt
  sind vor/nach beiden Analysen identisch.
- CLI mit leerem PATH, Git <2.49, ungültiger Versionsausgabe oder nicht ausführbarem Git:
  Exit 1, stdout leer, GIT_UNAVAILABLE als verständlicher Fehler.
- CLI außerhalb Repository: Exit 2/stdout leer. help/version erfolgreich ohne Git oder
  Repository, stderr leer.

Basisabdeckung und Herkunft der Nachweise:

| Vertrag | Nachweis |
| --- | --- |
| Root, SHA-1/SHA-256, valide Einzelrevisionen | T004-Modelltests und neuer SHA-256-Unterverzeichnistest |
| Addition/Löschung/Rename/fehlende Dateiseiten | tests/git.rs; nach T004/T005 erfolgreich |
| Raw-NUL-/Batch-Framing, ungültige Pfade/OIDs | T005-Raw-Regression und Batch-Unit-Test |
| Nicht-UTF-8-/Kontrollzeichenpfade | Synthetisches Raw-/Escape- und echtes Rename-Pfadbeispiel |
| Fehlende Objekte, Gitlinks/Symlinks/LFS/Mode/Binärdateien | Vorhandene Git-Integrationstests aus T005/T006 |
| Keine Mutation/Helferausführung | Git-Dirty-Worktree-/Helferproben, zusätzlich Index-/HEAD-Snapshot |
| Timeout/Pipe-/Outputfehler und Cleanup | Kontrollierte Prozessgruppe-/Pipe-Tests aus T005 |
| Blob-/Gesamtbyte-/Datei-/Zeilenlimits, Delta-Deadline | Zähler-/Delta-/Mehrdateitests aus T006 |
| Syntaxfehler/Literale/Kommentare/Layout | Fünf Syntax- und Regelprüfungen aus T007 |
| CLI-Exitklassen, keine Teil-/Geheimnisausgabe | T005-CLI-Prozessfehler und neue Verfügbarkeitsfälle |

Clippy mit -D warnings erfolgreich. Es wurden nur die drei neuen Basisfälle ausgeführt,
keine vollständige neue Suite/Release-/Docker-/003-Matrix. Anwendungscode blieb bei T009
unverändert; Testnachweise ergänzen den Basisabgleich.

Damit ist Phase 2 von 002 abgeschlossen. Nächste offene Aufgabe: T011, Kernformat-/
Fundstellen-/Deduplizierungsnachweise für US1; anschließend T013–T019-Regelrestarbeiten.

## T007 abgeschlossen: Neutralitäts-/Kontextabgrenzung

- UTF-8-Prüfung vor Parsing; Syntaxfehler/fehlende Tokens und Java-Unicode-Escapes
  liefern keine positive Neutralitätsbewertung. Parser-Tiefengrenze bleibt erhalten.
- Kommentare werden ausgeblendet; Strings/Docstrings/Regex-Literale behalten für den
  Neutralitätsvergleich ihren Inhalt, erzeugen aber keine ausführbaren Keyword-Tokens.
- JavaScript-/TypeScript-Template- und Python-f-String-Interpolationen werden zusätzlich
  als Code tokenisiert, auch verschachtelt ohne doppelte Erfassung. Ein Auth-Aufruf
  innerhalb einer Interpolation kann nicht mehr als bloßer Literaltext verschwinden.
- Python-Tokenpositionen erhalten Einrückung/Zeilengrenzen; JS/TS erhalten relevante
  Zeilenpositionen. Der Ansatz ist bewusst konservativ: nicht jede tatsächlich neutrale
  Formatänderung wird als LOW erkannt. Java-Leerraum bei gleichen Tokens bleibt neutral.

Nachweise: 5 Syntax-Unit-Tests mit vierprofiligen Kommentar-/Literal-/Layout-/Fehlerfällen,
14 Regeltests einschließlich neuer interpolierter Auth-Aufrufe erfolgreich. Clippy mit
-D warnings und IDE-Build erfolgreich. Kein Release-/Docker-/anderer Featureabschluss.

Nächste offene Basisaufgabe T009: vollständiger Basisabgleich; danach US1-Regelaufgaben.

## T004 nachgeholt und abgeschlossen

Kernmodelle gegen data-model.md abgeglichen und Validierung ergänzt:
- Comparison wird in der Git-Erzeugungsstelle durch einen geprüften Konstruktor erzeugt:
  absoluter Root, volle hexadezimale SHA-1-/SHA-256-OIDs im selben Format.
- Gemeinsame validate_revision_endpoint-Funktion in src/model.rs wird von CLI und
  Git.compare verwendet. Leere/Options-/Listen-/Zwei-/Dreipunkt-/Pathspec-Ausdrücke und
  Revisionsmengen HEAD^!/HEAD^@ werden ausgeschlossen; Einzelrevisionen bleiben zulässig.
- FileChange verwendet Option-Pfade, rohe Bytes und geordnete ChangedLine-Datensätze;
  Addition/Löschung hat die fehlende Seite ohne Inhalt. Modi/OIDs/Pfade werden im
  T005-Raw-Parser validiert; Blobtypen/Größen vor Lesen geprüft, keine Pfad-/Symlinkverfolgung.
- Findings haben statische Regel-/Begründungsmetadaten, optionales Zeilenfeld und klare
  BASE/TARGET/METADATA-Seite. Verarbeitung Result<T, AnalysisError>, explizite Exitklassen;
  TestAssociation gehört wie vereinbart zu 003, kein zweites Kernmodell eingeführt.

Nachweise: 3 neue Modell-Unit-Tests, 1 echte Git-Addition/Löschung-Prüfung einschließlich
abgelehnter Revisionsmenge, bestehender CLI-Aufrufvertrag erfolgreich. Clippy mit
-D warnings und IDE-Build erfolgreich. T005/T006-Arbeit wird weiterverwendet.
Nächste offene Basisaufgabe ist T007; keine Docker-/Release-/Gesamtprüfung durchgeführt.

## Erste Umsetzung nach der Aufteilung

Konkreter Teil von T005 umgesetzt in src/git.rs:
- Raw-Diff muss vollständig NUL-terminiert sein; leere Zwischenrecords und nachfolgende
  Fremddaten werden nicht als Ende eines erfolgreichen Teildiffs akzeptiert.
- Beide Rename-Pfade repositoryrelativ und ohne leere/./..-Segmente; rohe gültige
  Nicht-UTF-8-Pfadbytes bleiben erhalten.
- Vollständige gleichlange SHA-1/SHA-256-OIDs und bekannte Modi validieren;
  Addition/Löschung muss zu Null-OID/fehlendem Modus passen, R100 zu identischen Blobs.
- Batch-Blob-Antwort benötigt exakten Header, Bytezahl und Abschluss-LF.
- Git-Executable einmal auf absoluten Pfad auflösen. Bei Root-/Revisionssuche werden
  LIMIT_EXCEEDED/GIT_UNAVAILABLE nicht mehr als ungültige Nutzerrevision umklassifiziert.

Nachweise: Neuer Regressionstest reproduzierte zunächst die akzeptierte abgeschnittene
Raw-Antwort. Nach Korrektur 8 Git-Integrationstests und 2 gezielte Git-Unit-Tests erfolgreich;
Clippy mit -D warnings und IDE-Build erfolgreich. Keine Docker-/Releaseprüfungen.
Keine erneute Gesamt-Testmatrix behauptet.

## T005 abgeschlossen: Prozessüberwachung und Cleanup

Nach der ersten Framing-Korrektur auch die verbleibende Prozesssteuerung umgesetzt:
- stdin/stdout/stderr werden nicht blockierend bearbeitet, I/O-Fehler sofort signalisiert.
- Erfolgreicher Prozessabschluss wartet auf vollständig gelesene Ausgabe; ein schneller
  Exit darf weder einen Teiloutput noch übersehene Überschreitung erzeugen.
- Deadline/Outputlimit/Pipefehler beendet die eigene Prozessgruppe, sammelt den direkten
  Git-Prozess ein und beendet/joint die I/O-Threads. Geerbte offene Pipes nach Elternexit
  können die Rückkehr nicht mehr unbegrenzt blockieren.
- Arbeitsverzeichnis explizit auf die Repositorywurzel setzen; nicht das fremde Projekt
  ausführen. Nur Tests verwenden kontrollierte harmlose Shell-Dummies als Git-Ersatz.

Zusätzliche direkte Dependency libc 0.2.190 (bereits transitiv vorhanden) für Unix-fcntl
und Prozessgruppen-Signale. Kein neues Prozessframework; Cargo.lock aktualisiert.

Gezielte erfolgreiche Prüfungen:
- 7 Git-Unit-Tests: exaktes Framing/Fehlerklassifikation, blockiertes stdin + Timeout
  (1s Testbudget, Rückkehr <2s, direkte Prozess-ID verschwunden), vererbte offene Pipes,
  stdout-/stderr-Overflow, Broken Pipe/nonzero Exit und vollständiger schneller Erfolg.
- 1 CLI-Prozessfehler-Test: Ausgabeüberlauf/Fehler nach gültiger Revisionsauflösung
  liefert Exit 1, stdout leer und kein Dummy-Geheimniswert im stderr.
- 8 bestehende Git-Integrationstests erfolgreich, einschließlich ungültiger Revisionen,
  Raw-Pfade, Dirty Worktree, fehlende Objekte und deaktivierter Git-Helfer.
- Clippy mit -D warnings und IDE-Build erfolgreich. Keine Docker-/Releaseprüfung
  oder vollständige neue Regel-/Testzuordnungsmatrix.

T005 ist damit abgeschlossen. Nächste Aufgabe T006: kumulative Eingabe-/Delta-Budgets
und gesamte Analysegrenzen unabhängig von diesem Prozesscleanup vollständig nachweisen.

## T006 abgeschlossen: Kumulative Eingabe-/Delta-Budgets

- InputTotals zählt gelesene Basis-/Zielblobs über alle Dateien, reserviert deren Größe
  vor dem Lesen und verhindert Integerüberlauf. Grenzen inklusive: 2 MiB je Blob,
  32 MiB kumulativ. Fehlende Dateiseiten zählen null Bytes; exakte ungeänderte Renames
  brauchen keine Bloblesung, zählen aber weiterhin als Datei.
- Gesamtes verbleibendes Zeilenkontingent wird in die nächste Delta-Berechnung gegeben;
  jede hinzugefügte/gelöschte Zeile zählt, ohne zunächst einen zu großen Ergebnisvektor
  zu erzeugen. 100.000 Zeilen sind zulässig, die nächste Zeile führt zum Fehler.
- Git, Delta und CLI verwenden denselben absoluten 60s-Endzeitpunkt. similar erhält
  diese Deadline; danach und während Ergebnissammlung wird erneut geprüft. Seine mögliche
  zeitabhängige Approximation wird bei Deadlineüberschreitung ausschließlich verworfen.
  Die CLI prüft das Budget vor erfolgreicher Ausgabe auch nach Regel-/Testzuordnung.
- Dateiobergrenze 1.000 bleibt im strikten Raw-Parser vor der Blobverarbeitung erhalten.

Gezielte Nachweise: 2 Zähler-Unit-Tests (inklusive Grenzen/Überlauf/beide Dateiseiten),
3 Delta-Unit-Tests (Restkontingent, abgelaufenes Budget, teure Berechnung mit 1ms Budget),
10 Git-Integrationstests einschließlich 9 Dateien mit jeweils 2-MiB-Basis/Ziel und zwei
Dateien mit zusammen 100.002 neuen Zeilen. Mehrdatei-Blobüberschreitung liefert auch an
der CLI Exit 1/stdout leer. Bestehender CLI-Prozessfehlertest, Clippy und IDE-Build erfolgreich.
Keine Docker-/Release-/vollständigen anderen Featureprüfungen.

Nächster Schritt: T007 Neutralitäts-/Kontextabgrenzung und anschließend T009 vollständiger
Basisabgleich. T006-Abschluss ist kein Abschluss aller Regeln oder der gesamten Spec 002.

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
