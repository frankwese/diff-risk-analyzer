# Research: Git Diff Risk Analyzer MVP

## 1. CLI-Laufzeit und Distribution

**Decision**: Python >=3.12,<3.15, ein installierbares src-layout-Paket; `argparse`,
`subprocess`, `difflib`, `dataclasses`, `unittest`. setuptools nur für den Paketbau.

**Rationale**: Kein Anwendungscode vorhanden; das Java-IDE-Grundgerüst begründet keinen
Stackzwang. Byte-Prozessausgabe, CLI und Tests sind in Python bereits verfügbar.

**Alternatives considered**: Java 25 mit ProcessBuilder ist möglich, benötigt hier mehr
CLI-/Textverarbeitungscode. JGit würde eine zusätzliche Git-Implementierung einführen.

## 2. Syntax ohne Semantikversprechen

**Decision**: Python `ast`/`tokenize`; Java/JS/TS über Tree-sitter. Pins:
`tree-sitter 0.26.0`, Java 0.23.5, JavaScript 0.25.0, TypeScript 0.23.2.
Nur endliche lokale Bindungsregeln aus [test-association.md](contracts/test-association.md).

**Rationale**: Die Nutzervorgabe verlangt tatsächliche Verwendung im geänderten Testfall,
nicht allein Namen. Syntaxbäume liefern Bereiche und Aufrufe, aber keine Typ-/Callgraphanalyse.
Kommentare, Literale, Receiver und geänderte Testgrenzen sind mit Regex allein unzuverlässig.

**Alternatives considered**: Eigener stdlib-Teilparser würde vier Lexer/Scope-Modelle
erfordern. Vollständige Compiler/Language-Server wären unnötig groß und projektabhängig.
Kein zweiter Regex-Fallback, der trotz Parserfehlern positive Testnutzung behauptet.

**Evidence**: PyPI-Metadaten am 2026-10-13 prüfen die Pins und macOS-arm64-/Linux-arm64-Wheels.
Ein isolierter `uv run --no-project --python 3.12`-Probeaufruf mit allen vier Pins konnte
Java-, JS- und TS-Klassen ohne Parsefehler parsen. Keine Projektabhängigkeiten installiert.
Wheel-/Release-Test anderer Plattformen und Python-Versionen bleibt Implementierungsprüfung.

## 3. Read-only Git-Zugriff

**Decision**: Git >=2.49; keine Shell. Vertrauenswürdiges Git aus PATH einmal auflösen.
Jeder Prozess erhält `--no-pager --no-optional-locks --no-lazy-fetch --no-replace-objects`,
`-C ROOT`, `-c core.hooksPath=/dev/null -c core.fsmonitor=false`, deaktivierte globale/
systemweite Konfiguration, `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_ALLOW_PROTOCOL=`.
Geerbte `GIT_*`-Steuervariablen entfernen; keine Trace-Dateien, Git-Diff-Helfer oder Fetches.

Endpunkte separat mit `rev-parse --verify --end-of-options REV^{commit}` auflösen.
Danach nur volle hexadezimale OIDs verwenden. Repositorywurzel mit `rev-parse` bestimmen.
Raw-Delta:

```text
git diff-tree -r --no-commit-id --raw -z --no-abbrev --no-ext-diff --no-textconv
  --no-color --no-relative --ignore-submodules=dirty --submodule=short
  --find-renames=100% --no-rename-empty -l1000 -O/dev/null BASE_OID TARGET_OID --
```

Blobs über `cat-file --batch-check` und `cat-file --batch` lesen, Größe/Typ vor Lesen prüfen.
Batch-Header und exakt angegebene Bytes getrennt parsen. Keine Filter, Symlinkverfolgung
oder Repositoryprogramme. Gitlinks sind nicht bewertbar; Symlinkänderungen sind nicht
bewertbar außer exakter unveränderter Umbenennung. Fehlende Partial-Clone-Objekte: Fehler.

**Rationale**: Raw NUL-Pfade vermeiden problematische quoted Patchheader. Blobvergleich
ignoriert benutzerdefinierte Diff-/Textfilter. Nur exakte Renames sind kostenarm und stabil.

**Alternatives considered**: `git diff A...B` beantwortet eine andere Frage; nicht unterstützt.
Textpatch-/numstat-Parsing ist attributabhängig. Checkout/Fetch würde Laufzeitvertrag verletzen.

## 4. Deterministischer Vergleich und vollständige Bewertung

**Decision**: Zeilenvergleich per SequenceMatcher mit deaktivierter autojunk-Optimierung;
feste Limits aus cli.md. Kein Score, max(LOW, MEDIUM, HIGH). Regelstand 1.0.0 in Ausgabe.
Unbekannte substanzielle Änderungsbereiche führen zu Fehler, auch neben einem HIGH-Befund.

**Rationale**: Ein Signal erkennt ein Indiz, nicht sämtliche Änderungen in derselben Datei.
Vollständigkeit bedeutet Bewertung im festen Regelumfang, keine garantierte Risikoerkennung.

**Alternatives considered**: Pauschales MEDIUM für jede unbekannte Quelltextänderung würde
FR-012 aufheben. Vollständige Semantikgleichheit ist ohne Compiler-/Laufzeitanalyse unmöglich.

## 5. Geänderte Tests und lokale Nutzungsbindung

**Decision**: Vollständige Basis-/Zielinhalte nur geänderter Dateien lesen. Testfälle über
eindeutige Namen/Container abgleichen und eigene Quellbereiche vergleichen; Zeilenverschiebung
allein ist keine Änderung. Direktimport, direkter Konstruktor und unveränderter Receiver
im selben Testfall sind der positive Umfang. Aufruf kann in unveränderter Zeile des
geänderten Testfalls stehen. Keine Klasse-/Methodennutzung aus Kommentaren oder Import allein.

**Rationale**: Änderungen in Test B dürfen unveränderte Nutzung in Test A nicht aktivieren.
Bindings aus Fixture/Factory oder Overloads liefern unklar statt falsche Bestätigung.

**Alternatives considered**: Reine Pfadzuordnung widerspricht Nutzerklärung. Vollständiger
Callgraph über alle Repositorydateien ist für MVP nicht erforderlich.

## 6. Strukturierte Vertragsformate

**Decision**: JSON über die Standardbibliothek; YAML über `PyYAML==6.0.3` mit
SafeLoader und Ablehnung doppelter Schlüssel. Benutzerdefinierte Tags nicht ausführen;
Alias-/Rekursionstiefe und geladene Elementzahl begrenzen. OpenAPI ausschließlich
paths/components.schemas/properties/required und primitive type-Verträge vergleichen;
externe $ref nicht auflösen, komplexe allOf/oneOf als nicht bewertbar markieren.

**Rationale**: OpenAPI und Python/Node-Lockfiles sind verbreitete strukturierte Eingaben.
Ein selbstgebauter YAML-Parser spart hier keine Wartung und erzeugt falsche Feldzuordnungen.

**Alternatives considered**: Nur JSON würde die vorgesehenen YAML-Vertragsbeispiele nicht
abdecken. Allgemeine Schema-/Consumeranalyse ist für den heuristischen MVP zu umfangreich.
PyPI am 2026-10-13: Version 6.0.3, Python >=3.8. Isolierte Probe mit CPython 3.12:
Safe-load eines einfachen required/properties-Mappings erfolgreich. Laufzeit nutzt keine
Netzwerkreferenzen; Alias-/Schlüsselprüfungen sind noch zu implementieren und zu testen.

## Sources

- [Git globale Optionen](https://git-scm.com/docs/git)
- [Git diff-tree](https://git-scm.com/docs/git-diff-tree)
- [Git cat-file](https://git-scm.com/docs/git-cat-file)
- [Git rev-parse](https://git-scm.com/docs/git-rev-parse)
- [Python subprocess](https://docs.python.org/3/library/subprocess.html)
- [Python ast](https://docs.python.org/3/library/ast.html)
- [Tree-sitter Python bindings](https://github.com/tree-sitter/py-tree-sitter)
- [Tree-sitter Syntax/Fehler](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html)
- [JUnit Test](https://docs.junit.org/current/api/org.junit.jupiter.api/org/junit/jupiter/api/Test.html)
- [Jest API](https://jestjs.io/docs/api)
- [pytest conventions](https://docs.pytest.org/en/stable/explanation/goodpractices.html)
- [PyYAML documentation](https://pyyaml.org/wiki/PyYAMLDocumentation)

Alle technischen Unbekannten für Phase 1 entschieden. Parserprobe ist kein Anwendungstest.
