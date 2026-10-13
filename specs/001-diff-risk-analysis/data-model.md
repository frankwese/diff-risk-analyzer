# Data Model: Git Diff Risk Analyzer

Alle Modelle sind interne Rust-structs/enums mit unveränderlichen Referenzen während
der Auswertung; keine persistente Datenbank und keine zugesicherte öffentliche Rust-API.
Optionale Werte als `Option<T>`, Rohdaten als `Vec<u8>`, Ergebnisse als `Result<T, AnalysisError>`.

## RevisionComparison

- `root`: absoluter lokaler Repositorypfad.
- `base_oid`, `target_oid`: volle auf Commits aufgelöste OIDs (SHA-1 oder SHA-256).
- `requested_range`: validierter ursprünglicher Zwei-Endpunktvergleich.
- Validierung: beide Endpunkte nicht leer, keine Option/Revisionliste/Dreipunktnotation.

## FileChange

- `old_path`, `new_path`: rohe Git-Pfadbytes oder null bei Addition/Löschung.
- `old_mode`, `new_mode`, `old_oid`, `new_oid`; `kind`: ADD/MODIFY/DELETE/RENAME/TYPE.
- `old_blob`, `new_blob`: rohe Bytes, fehlende Seite als leerer Inhalt.
- `regions`: geordnete ChangeRegions mit alten/neuen Zeilenbereichen und Inhaltsreferenz.
- `category`: DOC/TEST/SOURCE/STYLE/DEPENDENCY/MIGRATION/CONTRACT/INFRA/UNKNOWN.
- Bounds: Pfade repositoryrelativ; niemals zur Symlinkverfolgung oder Checkout verwenden.
  Blobgröße/Typ vor Analyse prüfen. Bytepfade escaped anzeigen, nicht verlustbehaftet ersetzen.

## RiskRule und Finding

- RiskRule: `id`, `level`, `title`, `rationale`, Bedingungen aus rules.md; versionierter
  fester Katalog, keine benutzerdefinierten Plugins.
- Finding: `rule_id`, `level`, `path`, `side` (BASE/TARGET/METADATA), optional `line`,
  `description` aus statischen Vorlagen. Kein freier Quelltext-/Geheimniswert.
- Beziehung: Rule 1:n Findings; FileChange 1:n Findings.
- Sortierung: absteigende Risikostufe, rule_id, rohe Pfadbytes, Seite, Zeile;
  Deduplizierung nach kompletter Fundstellenidentität, nicht bloß Regel-ID.

## SymbolChange

- `language`, `path`, `container`, `name`, `kind` (CLASS/METHOD/FUNCTION),
  `declaration_key`, Basis-/Zielbereich, `change_kind`, `static`, Mehrdeutigkeitsmarker.
- Deklarationsschlüssel: Sprache, Pfad, Container, Name, normalisierte Signatur;
  keine alleinige Namensidentität bei Overloads.
- Methodenrumpfänderung: kleinste umgebende Methode, nicht zusätzlich gesamte Klasse.
  Klassenkopf/-feldänderung: Klasse. Gelöschtes Symbol: NOT_APPLICABLE.

## TestCaseChange und UsageEvidence

- TestCaseChange: Sprache, Pfad, Testcontainer/-name, eigene Basis-/Zielbereiche,
  `is_added_or_changed`, Importbindungen. Eindeutiges Matching erforderlich.
- UsageEvidence: Testfall-ID, Produktionssymbol-ID, Aufrufzeile und Bindungsart
  (CONSTRUCTOR/INSTANCE_CALL/STATIC_CALL/FUNCTION_CALL).
- Zählt nur im geänderten Testfall, mit eindeutiger Herkunft, zulässigem Receiver;
  Methodenänderung braucht passenden Methodenaufruf. Literal/Import ist kein Aufruf.

## TestAssociation

- `symbol` oder unbekannter Dateibereich; `status`: USED/MISSING/UNKNOWN/NOT_APPLICABLE.
- `evidence[]` bei USED; `reason_code` und Kandidatenfundstellen bei UNKNOWN.
- Priorität: valider positiver Befund → USED; andernfalls relevanter offener Kandidat →
  UNKNOWN; vollständig untersuchter unterstützter Umfang ohne Nutzung → MISSING;
  gelöschtes/inhaltlich nicht symbolbezogenes Ziel → NOT_APPLICABLE.
- Nicht erkennbare Produktionssyntax → UNKNOWN, nie erdachte Symbolnamen.

## AnalysisResult / AnalysisError

- Result: `rules_version`, `risk`, geordnete `findings`, `associations`, `empty_diff`.
- Error: `code`, betroffene escaped Pfade, statische Ursache; kein `risk`.
- Ablauf: INPUT → RESOLVED → READ → RISK_EVALUATED → ASSOCIATED → RENDERED.
  Jede ungültige/unsupported/limitbedingte Risikoanalyse endet in ERROR ohne Teil-Risk.
  UNKNOWN in TestAssociation stoppt diesen Ablauf nicht.
- `risk` ist Maximum aller Findings; leerer Diff Sonderfall LOW mit explizitem Hinweis.
  Keine TestAssociation verändert risk.
