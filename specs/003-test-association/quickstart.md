# Testzuordnung: Validierung

Bestehende Rust-Crate und Git-/Kernintegration von 002 nutzen.

```sh
cargo test --locked --test association
cargo test --locked --test cli
```

In allen vier Profilen direkte Konstruktion/Methodennutzung, statischen Aufruf, importierte
Funktion und Alias prüfen; falsche Klasse/Methode, Scheinnennung, unveränderten Test,
Teilzuordnung, Fixture/Factory/Helper und Parserfehler gemäß contracts/test-association.md.
Unveränderte Aufrufzeile im geänderten Einzeltest zählt; Setup/Import/Test-B-Änderung allein
nicht. Gleiche Suite-Namen unterscheiden, Überschattung nie USED. USED mit Fundstelle;
UNKNOWN begründen, Risk/Exit aus 002 unverändert. Zehn gleiche Ausgaben dokumentieren.

Kein erneuter Build anderer Zielplattformen zum Abschluss dieser Funktionsmatrix.
Fortschritt [status.md](status.md), Aufgaben [tasks.md](tasks.md).
