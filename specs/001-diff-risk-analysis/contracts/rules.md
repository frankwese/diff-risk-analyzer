# Risk Rules Contract v1.0.0

## Evaluation and coverage

Regeln sind im Code fest eingebettet; kein Laden von Repositorykonfiguration oder Plugins.
Pfadsegmente case-insensitive, Symbolbestandteile nach CamelCase/underscore segmentieren;
keine Substringtreffer `auth` in `author`. Inhaltskeywords nach Token-/Wortgrenzen,
case-insensitive; Fundstellen beziehen sich ausschließlich auf geänderte Zeilen.

Reihenfolge: DOC/TEST zuerst ausschließen, dann übrige Kategorien und alle zutreffenden
Signale sammeln. TEST kann zusätzlich durch Testsyntax erkannt werden. Gemischte
Test-/Produktionsdateien nicht als vollständig TEST ausschließen, sondern Scopes trennen.
Ein Inhaltsmuster deckt nur seine Anweisung/Deklaration oder bekannten strukturgebundenen
Bereich ab; keine pauschale Vollbewertung aller restlichen Dateiänderungen.

Für Quellen/SQL/CSS Kommentare nach unterstütztem Lexer/Parser abgrenzen. Source-Strings
werden nicht als ausführbare Symbolaufrufe gewertet, aber SQL-Stringmuster dürfen QUERY
auslösen. Reiner String mit Risikobegriff ohne passenden Operation-/Pfadbefund nicht HIGH.
Nichtdecodierbarer Inhalt oder unbekannte substanzielle Region → UNSUPPORTED_CHANGE.
Ein bekanntes HIGH mit einer unabhängigen unbekannten Region bleibt unvollständig.

## Classification vocabulary

- DOC: `docs/`, `doc/`; `.md`, `.rst`, `.adoc`; `README*`, `CHANGELOG*`, `LICENSE*`.
- TEST: Konventionen/Syntax aus test-association.md; immer von DOC abgegrenzt.
- SOURCE: `.java`, `.js`, `.mjs`, `.cjs`, `.ts`, `.py` mit unterstützten Textoperationen;
  andere Formate können über spezielle Pfad-/Formatregeln erfasst werden.
- STYLE: `.css`; nur Regeln mit unverändertem Selektor strukturell bewerten.
- DEPENDENCY: `pom.xml`, `build.gradle`, `build.gradle.kts`, `package.json`,
  `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `pyproject.toml`, `uv.lock`,
  `poetry.lock`, `requirements*.txt`. Manifeständerungen außerhalb des Abhängigkeitsbereichs
  lösen nicht allein DEPENDENCY aus. Lockfiles sind vollständig dependencybezogen.
- MIGRATION: Segmente `migration`, `migrations`, `db/migrate`, `db/migration` und `.sql`;
  unterstützte imperative `drop_table`, `drop_column` innerhalb derselben Pfadkonvention.
- CONTRACT: Segmente `api`, `contracts`, `domain`; `.java/.js/.ts/.py` öffentliche
  Deklarationen; `openapi.json`, `openapi.yaml`, `openapi.yml` strukturierte Verträge.
- INFRA: `.tf`, `.yaml`, `.yml`, `.json` in Segmenten `prod`, `production`;
  `iam`/`secrets`-Bereiche können auch ohne prod-Pfad ein eigenständiges HIGH liefern.

## Finite signal catalog

Jede Zeile definiert ID, Stufe, Auslöser und das Mindest-Referenzpaar. Pro Signalbefund
statische englische Rationale aus spec.md sinngemäß ausgeben, niemals Quelltextwerte.

| ID / Level | Auslöser und begrenzter Scope | Positives Referenzbeispiel | Negatives Referenzbeispiel |
| --- | --- | --- | --- |
| DOC_CHANGE / LOW | Inhalt einer DOC-Datei geändert; Source rein neutral nach untenstehendem Normalisierungsvergleich | README-Absatz ergänzt | README plus neue ausführbare if-Anweisung: DOC allein deckt Source nicht ab |
| NEUTRAL_RENAME / LOW | Raw R100, identischer Blob und Modus; keine neue Rolle in Auth/Payment/IAM/Prod-Pfad | `src/A.java` → `src/B.java`, identische Bytes | Umzug nach `src/auth/A.java` löst AUTH aus; Signaturänderung nicht neutral |
| TEST_CHANGE / LOW | Änderungen vollständig in TEST-Dateien/-Scopes | Assertion in `OrderTest.java` geändert | Produktionsmethode zusätzlich geändert: TEST deckt diese nicht ab |
| VISUAL_STYLE / LOW | Bei unveränderten CSS-Selektoren ausschließlich Werte von `color`, `background-color`, `margin`, `margin-top/right/bottom/left`, `padding`, `padding-top/right/bottom/left`, `font-size`, `font-weight` geändert | `.button { color: red; }` → blue | `display`, `visibility`, Selektoränderung oder `url(...)` nicht visuell abgedeckt |
| BUSINESS_LOGIC / MEDIUM | Source-Anweisungen mit `if`, `else`, `switch`, `case`, `match`, `for`, `while`, `return`, `throw`, `raise`, arithmetischen/vergleichenden Operatoren `+ - * / % == != < > <= >=`; zugehörige Block-/Deklarationshüllen nur soweit vollständig strukturell erkennbar | `return amount * 2` hinzugefügt | Keyword ausschließlich im Kommentar; freier unbekannter Aufruf `customSideEffect()` wird nicht durch Fallback MEDIUM abgedeckt |
| API_EXTENSION / MEDIUM | CONTRACT: neue öffentliche Java-Methode, JS/TS exportierte Funktion/Klasse, Python-Methode/Funktion ohne führendes `_`; OpenAPI neues optionales Feld/Endpoint; bestehende Vertragselemente erhalten | optionales Feld neben bestehenden Feldern hinzugefügt | entfernte erforderliche Eigenschaft: CONTRACT_BREAK |
| DEPENDENCY_UPDATE / MEDIUM | Geänderte Abhängigkeits-/Versionsdeklarationen in Manifest oder beliebige Änderung im benannten Lockfile; package.json dependencies/devDependencies/peerDependencies/optionalDependencies, pyproject project.dependencies/optional-dependencies oder tool.poetry.dependencies, requirements Einträge, Maven dependency-Blöcke, Gradle dependencies-Blöcke | Version einer dependency erhöht | package.json description geändert: kein DEPENDENCY-Befund |
| QUERY_PERFORMANCE / MEDIUM | Nicht-DOC/TEST: SQL `SELECT/INSERT/UPDATE/DELETE`, `CREATE/ALTER INDEX`, Source-Operationen `execute`, `executeQuery`, `query`, `prepareStatement` mit geändertem SQL; in MIGRATION zusätzlich additive `CREATE TABLE/ALTER TABLE ... ADD` | `SELECT ... WHERE` geändert | SQL nur in README; additive Migration nicht DESTRUCTIVE |
| CONCURRENCY_CACHE / MEDIUM | Source-Pfadsegmente `cache`, `caching`, `concurrency` bei ausführbarer Inhaltsänderung, oder geänderte Operationsidentifier `lock`, `unlock`, `synchronized`, `Thread`, `Executor`, `async`, `await`, `cache`, `invalidate`, `evict` | `cache.invalidate(key)` geändert | cache nur im unveränderten Kontext oder String: kein Inhaltsbefund |
| AUTH_CHANGE / HIGH | Source in Segmenten `auth`, `authn`, `authz`, `authentication`, `authorization` bei nicht neutraler Änderung; oder geänderte ausführbare Identifier `authenticate`, `authorize`, `checkPermission`, `hasRole`, `verifyToken` | `authorize(user)` entfernt | `authorName` oder Auth im Test/README: kein produktiver Befund |
| DESTRUCTIVE_MIGRATION / HIGH | MIGRATION: geänderte SQL-Operation `DROP TABLE`, `DROP COLUMN`, `TRUNCATE [TABLE]` oder ausführbarer `drop_table`/`drop_column`-Aufruf | migrations/V2.sql enthält neues DROP TABLE | migrations/V3.sql enthält nur CREATE TABLE |
| PAYMENT_CHANGE / HIGH | Source-Pfadsegment `payment`, `payments`, `billing`, `invoice`, `pricing` bei nicht neutralem Inhalt; oder ausführbare Identifier `charge`, `refund`, `capturePayment`, `calculatePrice`, `calculateInvoice` | billing/Invoice.java Preisrechnung geändert | Preistext in README; Wort charge ohne Operation/Pfad nicht ausreichend |
| SECRET_IAM_PROD / HIGH | Nicht-DOC/TEST: Secret-/IAM-Pfadsegment bei substanziellem Inhalt; Operationsidentifier `getSecret`, `putSecret`, `rotateSecret`; strukturelle IAM-Felder `Action`, `Resource`, `Effect`, `Principal` in Policy/iam-Datei; INFRA mit prod-Pfad und struktureller Änderung | production/service.tf Ressource geändert; iam/policy.json Action geändert | infra/local/service.tf allein nicht prod; IAM nur in Kommentar nicht ausreichend |
| CONTRACT_BREAK / HIGH | CONTRACT: vorhandene öffentliche Deklaration entfernt oder Name/Parameteranzahl/-typ/Rückgabetyp geändert; OpenAPI erforderliches Feld entfernt/Typ geändert oder Endpoint entfernt | required id entfernt; öffentliche Methode in domain/ geändert | neues optionales Feld, private Methode oder nur Methodenrumpf ohne Signaturänderung |

## Neutrality and exclusions

- Keine allgemeine Refactoring-Erkennung; NEUTRAL_RENAME deckt ausschließlich exakte
  Dateiumbenennungen ab. R100 mit Modusänderung ist nicht neutral.
- Source-Kommentare/Formatierung nur LOW, wenn gültiger Parser-/Tokenvergleich vor/nach
  identische Tokenwerte und relevante Zeilengrenzen liefert. Python INDENT/DEDENT,
  NEWLINE und String/Docstring erhalten; JS/TS Line-Terminatoren nicht pauschal entfernen.
  Java-Quellen mit Unicode-Escapes `\u....` für Neutralität/Bindung konservativ UNKNOWN
  behandeln. CSS nur explizite erlaubte Propertyänderungen als neutral/visuell akzeptieren.
- Auth-/Payment-/Secrets-Source-Kommentaränderung allein ist DOC_CHANGE, nicht HIGH.
  Neue Pfadrolle bei Rename kann dagegen einen Pfadbefund liefern.
- SOURCE-Parserfehler verhindern positive Neutralität und Testzuordnung, nicht zwingend
  unabhängige klar erkannte Pfad-/Text-Risikobefunde. Unbewertete Restbereiche bleiben Fehler.
- Entfernte ausführbare Entscheidungen gelten als BUSINESS_LOGIC-Verhaltensänderung,
  obwohl die fachliche Kategorie „Neue Business-Logik“ heißt; Ausgabe formuliert
  `Business behavior change detected` statt fälschlich „neue Logik“.
- Öffentliche Vertragsvergleiche nur auf erkennbare Signaturen und strukturierte OpenAPI-
  Elemente begrenzen; keine Vererbungs-/Typ-/Consumeranalyse. YAML via PyYAML SafeLoader
  mit eindeutigen Schlüsseln und begrenzter Alias-/Rekursionstiefe laden; keine eigenen
  Objekt-Tags, keine externen $ref. OpenAPI nur paths und components.schemas mit
  properties/required/primitive type vergleichen; komplexe allOf/oneOf sind unsupported.
  JSON via Standardbibliothek, doppelte Schlüssel ebenfalls ablehnen.

## Region accounting

Für jede Dateiseite/Änderungsregion Erfassen: neutral, durch spezielle Regel abgedeckt,
oder unsupported. Pfadregeln AUTH/PAYMENT decken ausführbare Änderungen im jeweiligen
Bereich, nicht beliebige unlesbare Inhalte. Infrastruktur-, Lockfile- und Dokumentations-
kategorien decken ihren ausdrücklich beschriebenen Format-/Pfadumfang.
UNKNOWN bei der Symbolzuordnung beeinflusst Region accounting nicht.
Aggregation nur bei vollständiger Bewertung; leeres Delta LOW; max level ohne Testbonus.
Sortierung und Deduplizierung siehe data-model.md.

## Reference acceptance inventory

Die 14 Tabellenpaare werden als Datenfixtures mit Basis/Zieltext, Pfad, erwarteten IDs,
Stufe oder UNSUPPORTED_CHANGE umgesetzt. Zusätzliche Pflichtfälle: gemischte Stufen,
unbekannter Bereich neben HIGH, reine Löschung, Mode-only, Pfadwechsel in kritische Rolle,
Kommentar/String-Scheintreffer, additive Migration, Python Einrückung, JS Zeilenumbruch,
OpenAPI optional versus required, Unicode-/Kontrollzeichenpfad. Identische Eingaben zehnmal
mit byteidentischer Ausgabe; keine behauptete Prüfung tatsächlicher Laufzeitsemantik.
