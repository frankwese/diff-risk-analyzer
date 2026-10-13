# Research: Git Diff Risk Analyzer MVP

## 1. CLI-Laufzeit und Distribution

**Decision**: Rust stable >=1.90, Edition 2024, eine Cargo-Crate mit nativem Binary
`risk-analyzer`. std::env::args_os, std::process::Command, structs/enums und Rust-Testharness.
Cargo.lock versionieren und Toolchain beim Implementierungsstart pinnen.

**Rationale**: Kein Anwendungscode vorhanden; das Java-IDE-Grundgerüst begründet keinen
Stackzwang. Der Nutzer hat Rust ausdrücklich gewählt. Ein natives Binary erfüllt die
Standalone-Anforderung ohne separate Python-/JVM-/Rust-Installation bei Nutzern.

**Alternatives considered**: Python+PyInstaller könnte eine Laufzeit bündeln, wurde durch
die Nutzerentscheidung ersetzt. Go mit C-Parseranbindung und Java mit gebündelter JVM
oder Native Image wurden besprochen; Rust bietet direkte Tree-sitter-Crate-Integration.
Git bleibt extern; Parser-C-Code wird beim Build eingebunden. Separate Darwin-Builds
und statische Linux-musl-Builds für jeweils x86_64/arm64, keine zusätzliche Parserdatei.

## 2. Syntax ohne Semantikversprechen

**Decision**: Tree-sitter Rust-Crate und Grammatiken für alle vier Quellsprachen. Pins:
`tree-sitter 0.27.1`, Java 0.23.5, JavaScript 0.25.0, TypeScript 0.23.2, Python 0.25.0.
Nur endliche lokale Bindungsregeln aus [test-association.md](contracts/test-association.md).

**Rationale**: Die Nutzervorgabe verlangt tatsächliche Verwendung im geänderten Testfall,
nicht allein Namen. Syntaxbäume liefern Bereiche und Aufrufe, aber keine Typ-/Callgraphanalyse.
Kommentare, Literale, Receiver und geänderte Testgrenzen sind mit Regex allein unzuverlässig.

**Alternatives considered**: Eigener Teilparser würde vier Lexer/Scope-Modelle
erfordern. Vollständige Compiler/Language-Server wären unnötig groß und projektabhängig.
Kein zweiter Regex-Fallback, der trotz Parserfehlern positive Testnutzung behauptet.

**Evidence**: crates.io-Metadaten am 2026-10-13 bestätigen die Pins; tree-sitter 0.27.1
benötigt Rust >=1.90. Rust-Binding dokumentiert Einbindung über LANGUAGE-Konstanten.
Frühere Python-Parserproben gelten nicht als Nachweis des Rust-Builds. Die lokale
rustc-Installation meldet `bad CPU type in executable`; native Toolchain-Reparatur und
Parser-ABI-/Release-Proben sind vor Implementierung erforderlich. Keine Installation
oder Änderung der Toolchain im Rahmen dieser Planaktualisierung.

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

**Decision**: Zeilenvergleich via `similar 3.2.0`, festes `Algorithm::Myers`;
feste Limits aus cli.md. Kein Score, max(LOW, MEDIUM, HIGH). Regelstand 1.0.0 in Ausgabe.
Unbekannte substanzielle Änderungsbereiche führen zu Fehler, auch neben einem HIGH-Befund.
Kein zeitabhängiger Diff-Fallback; Timeout bedeutet Analysefehler statt Teilergebnis.

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

**Decision**: JSON über `serde_json 1.0.154`; YAML über `serde_yaml_ng 0.10.0` mit
Ablehnung doppelter Schlüssel vor Map-Konvertierung. Benutzerdefinierte Tags nicht ausführen;
Alias-/Rekursionstiefe und geladene Elementzahl begrenzen. OpenAPI ausschließlich
paths/components.schemas/properties/required und primitive type-Verträge vergleichen;
externe $ref nicht auflösen, komplexe allOf/oneOf als nicht bewertbar markieren.

**Rationale**: OpenAPI und Python/Node-Lockfiles sind verbreitete strukturierte Eingaben.
Ein selbstgebauter YAML-Parser spart hier keine Wartung und erzeugt falsche Feldzuordnungen.

**Alternatives considered**: Nur JSON würde die vorgesehenen YAML-Vertragsbeispiele nicht
abdecken. Allgemeine Schema-/Consumeranalyse ist für den heuristischen MVP zu umfangreich.
crates.io am 2026-10-13 bestätigt beide Pins; serde_yaml_ng MSRV 1.64. Rust-Syntax-/
Alias-/Schlüsselprüfungen sind noch zu implementieren und zu testen. Kein eigenes YAML-
Parsing; der archivierte serde_yaml-Vorgänger wird nicht als neue Abhängigkeit gewählt.

## Sources

- [Git globale Optionen](https://git-scm.com/docs/git)
- [Git diff-tree](https://git-scm.com/docs/git-diff-tree)
- [Git cat-file](https://git-scm.com/docs/git-cat-file)
- [Git rev-parse](https://git-scm.com/docs/git-rev-parse)
- [Rust Command](https://doc.rust-lang.org/std/process/struct.Command.html)
- [Tree-sitter Rust bindings](https://github.com/tree-sitter/tree-sitter/tree/master/lib/binding_rust)
- [similar algorithms](https://docs.rs/similar/3.2.0/similar/enum.Algorithm.html)
- [Tree-sitter Syntax/Fehler](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html)
- [JUnit Test](https://docs.junit.org/current/api/org.junit.jupiter.api/org/junit/jupiter/api/Test.html)
- [Jest API](https://jestjs.io/docs/api)
- [pytest conventions](https://docs.pytest.org/en/stable/explanation/goodpractices.html)
- [serde_yaml_ng](https://docs.rs/serde_yaml_ng/0.10.0/serde_yaml_ng/)

Technische Entscheidungen getroffen. Rust-Build-/Parserkompatibilität und vier Releaseziele
sind Implementierungsprüfungen; aktuell durch ungeeignete lokale rustc-Binärdatei blockiert.
