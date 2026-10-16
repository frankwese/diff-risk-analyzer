# Status: Lokale Risikobewertung

**Stand**: Dokumentenmigration am 2026-10-13, Basiscommit eb5de1e.
**Status**: Teilweise implementiert; noch nicht konvergiert.

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
