# CLI Contract v1.0.0

## Invocation

```text
risk-analyzer BASE..TARGET
risk-analyzer --help
risk-analyzer --version
```

Genau ein positional argument, genau ein `..` und kein `...`; Endpunkte nicht leer,
keine führenden Optionszeichen. Git-Revisionausdrücke wie HEAD~1 und Branch/Tag erlaubt,
aber ausschließlich einzeln zu Commit-OIDs auflösen. Kein Pathspec, Staging- oder
Worktree-Modus. Im Repository-Unterverzeichnis dieselben Endpunkte und Root-Pfade.
`--help` und `--version`: Exit 0, keine Repositoryanalyse.

## Successful output

UTF-8, Englisch, LF, kein Farbcode oder Zeitstempel. Vollständig gepufferte Ausgabe:

```text
Risk: HIGH
Reasons:
- [AUTH_CHANGE] AuthN/AuthZ change detected; security impact possible — src/auth/Guard.java (target:12)
- [DESTRUCTIVE_MIGRATION] Destructive migration operation detected; data loss possible — migrations/V2.sql (target:1)
Tests:
- MISSING src/auth/Guard.java::Guard.authorize — no changed test using this method detected
Rules: 1.0.0
Note: Heuristic review aid; test usage does not prove coverage or correctness.
```

Reasons nach rules.md/data-model.md sortieren. BASE-Fundstellen für Löschungen;
METADATA-Gründe enthalten keine erfundene Zeile. Jede Datei eindeutig escaped anzeigen:
Kontrollzeichen als `\n`, `\t` etc., nicht-UTF-8-Bytes als `\xNN`; Backslash escapen.
Produktions-/Testnamen aus Symbolmetadaten, nie vollständige Quellzeilen ausgeben.
Secrets-Signale nennen ausschließlich Regel und Fundstelle, keine Schlüsselwerte.

Tests separat nach Pfad/Symbol sortieren: USED, MISSING, UNKNOWN oder NOT_APPLICABLE.
USED nennt Testpfad, Testname und Aufrufzeile; UNKNOWN nennt statischen Ursachencode.
Ohne anwendbare Produktionsänderung `Tests: Not applicable.`

Leerer Diff:

```text
Risk: LOW
Reasons:
- No changes between the requested revisions.
Tests: Not applicable.
Rules: 1.0.0
Note: Heuristic review aid; test usage does not prove coverage or correctness.
```

Erfolg aller Klassen: Exit 0, stderr leer. Risk HIGH ist kein technischer Fehler.

## Errors

- Exit 2: ungültiger Aufruf/Revisionvergleich, kein Repository oder ungültige Revision.
- Exit 1: fehlendes/ungeeignetes Git, fehlende Objekte, Prozess-/Lesefehler,
  Ressourcenlimit oder nicht bewertbare substanzielle Änderungen.
- stdout leer; stderr `Error: CODE — statische Ursache` und ggf. sortierte escaped Pfade.
  Keine abschließende Risk-Klasse; auch bekannter HIGH-Treffer neben unbekanntem Inhalt
  wird nicht als vollständiges Ergebnis ausgegeben.
- UNKNOWN der Testzuordnung allein ist kein Fehler und erlaubt erfolgreiche Risikoausgabe.

## Input limits and unsupported data

Maximal 1.000 Dateien, 2 MiB je Blob, 32 MiB insgesamt gelesene Blobdaten,
100.000 hinzugefügte/gelöschte Zeilen, 60 Sekunden insgesamt. Bounded Prozess-/Bloblesen;
Limits vor Syntax-/Zeilenvergleich prüfen. Unterschiedliche Parser-/Diffkosten zählen mit.
Binärdaten, nicht decodierbare Inhalte, Gitlinks, unbekannte substantielle Änderungsbereiche:
UNSUPPORTED_CHANGE. Exakt umbenannte unveränderte Dateien benötigen keine Inhaltsbewertung.
LFS-Pointeränderungen sind nicht die referenzierte Binärdatei und bleiben unsupported.

## Read-only guarantees

Nur rev-parse, diff-tree, cat-file; kein Checkout, Fetch, Hook, Textconv, External Diff,
Build, Testlauf oder Import aus dem analysierten Repository. Vollständige Revisioninhalte
dürfen für Kontext/Testfälle gelesen werden; keine persistenten Reports oder Caches.
Installations-/Entwicklungsbefehle können Abhängigkeiten laden; die installierte CLI nicht.
