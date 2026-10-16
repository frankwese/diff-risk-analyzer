# Test Association Contract v1.0.0

**Owner**: Feature 003. Aus Gesamt-Spec 001 übernommen; Risiko-/Testklassifikation und
Basis-CLI bleiben bei [Feature 002](../../002-local-risk-analysis/spec.md).

## Positive syntax profiles

Risikoerkennung bleibt Pfad-/Textanalyse auch außerhalb dieser Syntaxprofile.
Parserfehler (Tree-sitter ERROR/MISSING in jedem der vier Profile) sperren positive
Testzuordnung der betroffenen Datei. Kein toleranter Text-Fallback für USED.

| Sprache | Deklarationen | Testfälle | Herkunft/Bindung |
| --- | --- | --- | --- |
| Java | Top-Level class, direkte Methoden/Konstruktoren mit Körper; static explizit | JUnit Jupiter `@Test`, expliziter Import oder voller Annotationname | Klassen-FQN via package plus Einzelimport; direkte lokale `C x = new C()`/`var x = new C()`; `x.m()`/statische `C.m()` |
| JS | Benannte Top-Level class, direkte normale/async Methoden; Top-Level functions | `test`/`it` mit inline arrow/function callback, auch in `describe`; Jest/Vitest globals oder expliziter Import | Relative direkte ESM imports/aliases zu geänderten Dateien; `const x = new C()`; `x.m()`/statische `C.m()` |
| TS | JS-Profil plus Typannotationen und Sichtbarkeitsmodifier | wie JS | wie JS; type-only import/Typannotation allein erzeugt keine Runtime-Bindung |
| Python | Top-Level class/def/async def, direkte Methoden, explizite staticmethod | Top-Level test_*, Test*-Klassen, unmittelbar erkennbare unittest.TestCase | expliziter from-import/alias oder Modulimport; `x = C()`; `x.m()`/staticmethod |

Innere/anonyme Klassen, Overloads, JSX/TSX, berechnete Methoden, Properties, classmethod,
eigene Dekoratoren, parametrische/generierte Testwrapper: nicht positiv auflösbar.
JS/TS-Testnamen dürfen nicht lokal überschattet sein.

## File and test matching

Testkandidaten: Segmente `test`, `tests`, `__tests__`, `src/test`; Basenames
`test_*.py`, `*_test.py`, `Test*.java`, `*Test.java`, `*Tests.java`,
`*.test.js`, `*.spec.js`, `*.test.ts`, `*.spec.ts`. Erkennbare Testfallsyntax kann
auch ohne passenden Basename TEST begründen. Dokumentation wird zuvor ausgeschlossen.

Nur geänderte Testdateien im Zielstand scannen; alle ihre unterstützten Testfälle parsen.
Einzelnen Testfall nach (Sprache, Container, Testname) zwischen Basis und Ziel abgleichen.
Doppelte Namen/unklare Grenzen: UNKNOWN. Seine eigenen Quellbytes einschließlich Kopf/
Annotation/Decorator vergleichen; geänderte Imports, Setup, Suite oder Zeilenverschiebung
aktivieren keinen unveränderten Fall. Löschung innerhalb weiterbestehendem Test zählt;
gelöschter Test nicht. Unveränderte Aufrufzeile im geänderten Test zählt.

## Import and receiver resolution

- Java: expliziter Einzelimport/FQN; gleicher package-Name nur wenn eindeutig innerhalb
  desselben erkennbaren `src/main/java`/`src/test/java`-Modulroots. Wildcards/mehrere
  passende Klassen: UNKNOWN.
- JS/TS: direkte relative Imports; explizite Erweiterung oder eindeutiger Kandidat unter
  `.js`, `.ts`, `.mjs`, `.cjs` sowie `/index` derselben Erweiterungen. `.cjs` bleibt
  nur für ESM-Syntax positiv; kein CommonJS require. Mehrere Kandidaten: UNKNOWN.
  Kein Barrel, tsconfig-Alias, Package-Export oder dynamischer Import.
- Python: Repositoryroot oder `src/` als einziger eindeutiger Modulroot; `.py` oder
  Package `__init__.py`. Keine Laufzeitimports/sys.path-Manipulation oder Exportketten.
- Auflösungen nur gegen geänderte Produktionsdateien; mehrere mögliche Herkunftsziele
  sperren USED. Ein Dateiname allein erzeugt niemals Bindung.
- Konstruktorbindung innerhalb geradliniger Statements desselben Testfalls; kein
  Shadowing/Reassignment. Importalias erlaubt, Receiveraliasfortpflanzung nicht.
- Aufruf auf direkt konstruierter lokaler Instanz oder direkter Konstruktion zählt,
  ebenso eindeutige direkte statische/Funktionsaufrufe. Methodennamen müssen unmittelbar
  auf der gebundenen Klasse deklariert sein. Mehrere gleichnamige Deklarationen: UNKNOWN.
- Keine Fixture-/Factory-/Mock-/DI-/self-/this-Feldbindung, Vererbung, Reflection,
  Helper-/Callgraph-Verfolgung oder Kontrollflusszusammenführung.
- Aufrufe innerhalb gewöhnlicher Assertionargumente zählen; Aufrufe in weiter
  verschachtelter Lambda/Funktion bleiben UNKNOWN.
- Kommentare, Strings, Imports, Bezeichner ohne Aufruf sind keine UsageEvidence.

## Changed symbols and status

Geänderte Methodenkörper/-signaturen auf Methodenebene; Klasse nicht zusätzlich aufgrund
derselben Methode als erledigbarer Class-Hinweis behandeln. Klassenkopf/-felder → CLASS.
Renames von Methoden: altes Symbol gelöscht, neues hinzugefügt, keine Semantikbehauptung.

USED wenn mindestens ein valider Befund vorliegt; andernfalls UNKNOWN wenn relevanter
Kandidat/Scope/Binding unklar; MISSING wenn unterstützter Umfang keinen Befund liefert.
Gelöschtes Symbol/reine nicht-symbolbezogene Metadaten → NOT_APPLICABLE.
Unbekannte Quellsyntax → dateibezogen UNKNOWN. Zustände verändern Risk nicht.

## Required reference matrix

In Java, JS, TS und Python jeweils direkte Konstruktion+Methodenaufruf, statischer Aufruf,
Importalias, andere gleichnamige Klasse, passende Testdatei ohne Aufruf, Teilabdeckung
zweier Methoden und Import/Kommentar/String als Scheintreffer prüfen.
Zusätzlich: Assertionänderung mit unveränderter Nutzung im selben Test; nur anderer Test
geändert; Test-/Aufruflöschung; nur Setup/Import/Suite geändert; Zeilenverschiebung;
Fixture/Factory/Receiveralias; Shadowing; overload; indirekter Helper; Parserfehler.
Erwartete Zustände strikt aus obigen Regeln ableiten, nicht Laufzeitcoverage behaupten.
