# Data Model: Kernanalyse

Interne Rust-structs/enums aus src/model.rs, keine Datenbank/öffentliche Bibliotheks-API.
Optionen `Option<T>`, Rohdaten `Vec<u8>`, Ergebnisse `Result<T, AnalysisError>`.
Modelle bleiben interne Datenträger; externe Werte werden an Erzeugungsstellen validiert.
Comparison::new prüft Root/OID-Format; validate_revision_endpoint wird von CLI und Git
gemeinsam genutzt, FileChange-Metadaten vom strikten Raw-Parser geprüft.

- **Comparison**: absoluter root, volle Basis-/Ziel-Commit-OIDs (SHA-1/SHA-256), Dateien.
  Endpunkte nicht leer, keine Option/Revisionliste/Dreipunktnotation.
- **FileChange**: alte/neue rohe Pfade, Modi/OIDs, ADD/MODIFY/DELETE/RENAME/TYPE,
  Blobs (fehlende Seite leer), geordnete ChangeRegions und Dateikategorie.
  Keine Symlinkverfolgung/Checkout; Größen-/Typprüfung vor Lesen/Analyse.
  ChangeRegions werden aktuell als geordnete ChangedLine-Datensätze (Seite, 1-basierte
  Zeile, Inhalt) gehalten; Dateikategorie wird aus Pfad/Inhalt in rules.rs abgeleitet,
  statt redundant auf FileChange gespeichert. Keine zusätzliche Persistenz.
- **Finding**: Regel-ID, Stufe, statische Rationale, Pfad, BASE/TARGET/METADATA und
  optionale Zeile. Kein Quelltext-/Geheimniswert; Metadaten ohne erfundene Zeile.
- **Result/Error**: geordnete Findings/risk/rules_version oder Fehlercode/Pfade ohne risk.
  Sortierung Stufe absteigend, rule_id, rohe Pfadbytes, Seite, Zeile; Deduplizierung
  vollständiger Fundstellenidentität. Risk max-Level, leerer Diff LOW.

Ablauf INPUT → RESOLVED → READ → RISK_EVALUATED → RENDERED; jeder unbekannte Bereich
endet in ERROR ohne Teil-Risk. 003 ergänzt optional ASSOCIATED vor Rendern.

**InputTotals**: interne kumulative Blobbytes/geänderte Zeilen. Reservierung vor Lesen,
checked_add gegen Überlauf; Delta erhält verbleibendes Zeilenkontingent.
**Budget**: gemeinsamer absoluter Endzeitpunkt für Git, Delta und CLI-Ausgabefreigabe;
Deadlineüberschreitung liefert LIMIT_EXCEEDED, niemals einen approximativen Erfolgsdiff.
