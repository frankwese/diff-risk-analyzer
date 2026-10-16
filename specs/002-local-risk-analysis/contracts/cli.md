# Kern-CLI Contract v1.0.0

**Owner**: 002; erweitert durch 003, ausgeliefert durch 004.

## Invocation and output

`risk-analyzer BASE..TARGET`, `--help`, `--version`; genau ein Zwei-Revisionsvergleich.
Nichtleere Endpunkte, kein Dreipunkt/Pathspec/Optionsargument. Einzelne Endpunkte auf volle
Commit-OIDs auflösen; Root/Unterverzeichnis identisch behandeln.

Erfolgreiche Ausgabe UTF-8, Englisch, LF, keine Farben/Zeitstempel:

```text
Risk: HIGH
Reasons:
- [AUTH_CHANGE] AuthN/AuthZ change detected; security impact possible — src/auth/Guard.java (target:12)
Rules: 1.0.0
Note: Heuristic review aid; no correctness or security guarantee.
```

003 darf vor Rules einen separaten Tests-Abschnitt und einen ergänzten Heuristikhinweis
einfügen. 002 bewertet diesen Abschnitt nicht als Voraussetzung für seinen Abschluss;
bestehende Tests-Ausgabe muss bei Kernänderungen weiterhin kompatibel bleiben.

Gründe nach Stufe absteigend, Regel-ID, rohen Pfadbytes, Seite, Zeile sortieren und nach
vollständiger Fundstelle deduplizieren. BASE für Löschungen, TARGET für neue Inhalte,
METADATA ohne erfundene Zeile. Kontroll-/Nicht-UTF-8-Bytes escapen, keine Rohquelltexte
oder Geheimniswerte. Leerer Diff: LOW mit `No changes between the requested revisions.`

## Exit and completeness

Exit 0 für vollständige Analyse jeder Stufe; stderr leer. Exit 2 für ungültigen Aufruf,
Revision oder fehlendes Repository; Exit 1 für fehlendes/ungeeignetes Git, Verarbeitung,
fehlende Objekte, unsupported Regionen oder Limits. Fehler stdout leer, stderr
`Error: CODE — statische Ursache` und betroffene sortierte Pfade; kein Teil-Risk.

Maximal 1.000 Dateien, 2 MiB je Blob, 32 MiB kumulative Blobdaten, 100.000 geänderte
Zeilen, 60 Sekunden gesamte Analyse. Binär-/LFS-/Gitlink/unbekannte Regionen unsupported.
Exakte unveränderte Renames nur im vereinbarten Regelumfang neutral behandeln.

## Read-only Git

Git >=2.49; nur rev-parse/diff-tree/cat-file, keine Shell oder Repositoryprogramme.
Raw-NUL-Pfade und rohe Blobs, no-ext-diff/no-textconv/no-lazy-fetch/no-optional-locks/
no-replace-objects, deaktivierte fsmonitor/hooks, bereinigte GIT-Umgebung.
Keine Laufzeitdownloads, Checkout/Fetch, Anwendungstests oder persistente Reports.
