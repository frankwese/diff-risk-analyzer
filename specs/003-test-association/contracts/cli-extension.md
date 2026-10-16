# Testhinweis-Erweiterung des Kern-CLI-Vertrags

**Owner**: 003; Basis: [002 CLI](../../002-local-risk-analysis/contracts/cli.md).

Nach Reasons und vor Rules separaten `Tests:`-Abschnitt einfügen. Zeilen nach Pfad/Symbol
sortieren; USED nennt eindeutiges Symbol, Testdatei/-fall und Aufrufzeile; UNKNOWN nennt
statischen Ursachencode. MISSING/NOT_APPLICABLE dürfen keinen Coveragebeweis suggerieren.

```text
Tests:
- USED src/Guard.java::Guard.authorize — usage detected in changed test; src/GuardTest.java::allows (target:8)
```

Ohne anwendbare Symboländerung `Tests: Not applicable.`; keine Quelltext-/Geheimniswerte.
Note darf ergänzen: `test usage does not prove coverage or correctness`.
UNKNOWN allein verändert weder Kern-Risk noch Exit 0. Reihenfolge byteidentisch bei
gleichen Eingaben/Regeln; bestehende LOW/MEDIUM/HIGH-Befunde unverändert lassen.
