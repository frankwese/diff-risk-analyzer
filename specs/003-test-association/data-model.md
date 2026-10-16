# Data Model: Testzuordnung

Basisdaten Comparison/FileChange/Fehler aus [002](../002-local-risk-analysis/data-model.md).

- **SymbolChange**: Sprache, Pfad, Container, Name, Typ CLASS/METHOD/FUNCTION, normalisierte
  Signatur, Basis-/Zielbereich, Änderungsart, static/ambiguity. Identität niemals allein
  Name bei Overloads; Methodenänderung kleinste Methode, Klassenkopf/-feld eigene Klasse.
- **TestCaseChange**: Sprache, Testpfad, Container/Name, eigene Basis-/Zielbytes und Bereiche,
  is_added_or_changed, Importbindungen. Mehrfach-/unklarer Match UNKNOWN.
- **UsageEvidence**: Testfall-/Symbolidentität, Zeile, CONSTRUCTOR/INSTANCE_CALL/STATIC_CALL/
  FUNCTION_CALL. Methode benötigt passenden Aufruf, nicht bloß importierten Namen.
- **TestAssociation**: Symbol oder unbekannter Dateibereich, USED/MISSING/UNKNOWN/
  NOT_APPLICABLE, Belege und statischer Ursachencode. Kein erfundener Symbolname.

Priorität: eindeutiger gültiger Beleg USED; andernfalls unaufgelöster relevanter Kandidat
UNKNOWN; vollständig unterstützter Umfang ohne Beleg MISSING; gelöschtes/nicht anwendbares
Ziel NOT_APPLICABLE. Geordnete Ausgabe verändert weder Risk noch Kernfehlerstatus.
