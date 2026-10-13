# Implementation Plan: Git Diff Risk Analyzer MVP

**Branch**: `main` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-diff-risk-analysis/spec.md`

## Summary

Lokale CLI `risk-analyzer BASE..TARGET` mit 14 deterministischen Risikosignalen,
höchster Signalstufe als Gesamtrisiko und getrennten symbolbezogenen Testhinweisen.
Python steuert Git ausschließlich lesend an. Rohdaten und vollständige Revision-Blobs
werden ohne externe Diff-/Textfilter gelesen. Ein fester Pfad-/Textkatalog bewertet
Risiken; Python `ast` und drei Tree-sitter-Grammatiken liefern Syntax für begrenzte
Testnutzungszuordnung. Kein LLM, Netzwerk, Repository-Codeimport oder Testlauf der
analysierten Anwendung. Unbekannte Risikoänderung bedeutet Fehler; unklare Testnutzung
bleibt ein Hinweis. Ausführbare Regelspezifikation: [contracts/rules.md](contracts/rules.md).

## Technical Context

**Language/Version**: Python >=3.12,<3.15; Entwicklungsbasis CPython 3.12.

**Primary Dependencies**: Git >=2.49 mit `--no-lazy-fetch`; `tree-sitter==0.26.0`,
`tree-sitter-java==0.23.5`, `tree-sitter-javascript==0.25.0`,
`tree-sitter-typescript==0.23.2`, `PyYAML==6.0.3` für YAML-Verträge/Manifeste.
Python-Syntax über `ast`/`tokenize`.
Packaging mit setuptools als reiner Build-Abhängigkeit; console-script entry point.

**Storage**: Keine persistente Anwendungsablage; Revision-Blobs, Befunde und Syntaxbäume
im Speicher. Git-Integrationstests verwenden temporäre Repositorys.

**Testing**: Standardbibliothek `unittest`; Regel-/Bindungsfixtures, CLI-Vertragstests
und echte Git-Repositorytests. Keine zusätzliche Testframework-Abhängigkeit.

**Target Platform**: macOS und Linux, x86_64/arm64 mit Python und Git; erstes praktisch
geprüftes Entwicklungsziel macOS arm64. Andere Ziele vor Release gesondert validieren.

**Project Type**: Installierbare CLI, ein Paket, keine öffentliche Bibliotheks-API.

**Performance Goals**: Entwicklungsziel <=5 Sekunden für 100 Dateien/10.000 geänderte
Zeilen und <=256 MiB Spitzen-RSS auf dokumentierter Referenzmaschine; kein neues
Produkt-Akzeptanzkriterium. Ein repräsentativer Smoke-Benchmark im Abschluss reicht.

**Constraints**: Offline zur Laufzeit; keine Git-Filter/Hooks/Builds ausführen.
Feste Schutzgrenzen: 1.000 Dateien, 2 MiB pro Blob, 32 MiB kumulative gelesene Blobdaten,
100.000 geänderte Zeilen; 60 Sekunden Gesamtlaufzeit. Überschreitung: Fehler ohne Risk.
Keine Mutation des analysierten Repositorys. Schutzgrenzen sind versionierter CLI-Vertrag.

**Scale/Scope**: 14 Regeln; allgemeinere Dateiklassifikation und vier Syntaxprofile für
Testzuordnung. Kein vollständiger Callgraph, Typchecker, Regelplugin oder Konfigurationssystem.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Prinzip | Vor Recherche | Nach Design |
| --- | --- | --- |
| Nachvollziehbare Anforderungen | PASS: spec.md und 16 geprüfte Checklistenpunkte | PASS: FR-001–015 auf CLI-, Regel- und Bindungsverträge abgebildet |
| Kleinste vollständige Lösung | PASS: leeres src, keine bestehende Implementierung | PASS: ein Python-Paket; Parser ersetzen komplexe eigene Teilparser; keine Service-/Plugin-Schichten |
| Überprüfbare Korrektheit | PASS: Akzeptanzfälle vorhanden | PASS: 14 positive/negative Regelpaare, Bindungsmatrix und Git-Vertragstests geplant |
| Explizite Eingaben und Fehler | PASS: Fehler statt falscher LOW-Bewertung | PASS: OID-Validierung, Größenlimits, Secrets ohne Quelltextausgabe, Status getrennt |
| Nachvollziehbare Ergebnisse | PASS: feste Regeln, stabile Reihenfolge | PASS: Regelstand, Datei/Seite/Zeile, deterministische Diff-/Sortierpolitik |

Keine ungerechtfertigten Abweichungen. Plan endet nach Design; Umsetzung erst über Aufgaben.

## Project Structure

### Documentation (this feature)

```text
specs/001-diff-risk-analysis/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/requirements.md
└── contracts/
    ├── cli.md
    ├── rules.md
    └── test-association.md
```

`tasks.md` wird erst durch `/speckit.tasks` erzeugt.

### Source Code (repository root)

Geplante Struktur, noch nicht implementiert:

```text
pyproject.toml
README.md
src/risk_analyzer/
├── __init__.py
├── __main__.py
├── cli.py
├── git.py
├── model.py
├── rules.py
├── association.py
└── syntax.py
tests/
├── test_rules.py
├── test_association.py
├── test_git.py
├── test_cli.py
└── fixtures/
    ├── rules/
    └── association/
```

**Structure Decision**: Das vorhandene leere `src/` wird genutzt. Module entsprechen
konkreten Verantwortungen; keine austauschbaren Parser-/Repository-Frameworks.
IDE-Java-Metadaten sind keine Produktvorgabe. README beschreibt Installation und Grenzen.

## Design Execution

1. CLI validiert genau einen Zwei-Revisionsvergleich und löst beide Endpunkte in OIDs auf.
2. Git liefert NUL-getrennte Raw-Änderungen und rohe Blobs. Inhaltsvergleich über
   `difflib.SequenceMatcher(autojunk=False)` auf vollständigen Zeilen; Limits vor Vergleich.
3. Risikoanalyse klassifiziert Pfade und geänderte Inhalte anhand Regelkatalog 1.0.0.
   Jeder substanzielle Änderungsbereich muss ein unterstütztes Bewertungsmuster besitzen;
   ein HIGH-Treffer macht andere unbekannte Bereiche nicht automatisch bewertbar.
4. Syntaxanalyse der geänderten Quell-/Testdateien bestimmt Symbol- und Testfallbereiche.
   Die Risikoregeln sind keine AST-/Compiler-Vollanalyse und verlangen kein Sprachprofil.
5. Testfälle werden anhand eigener Alt-/Neu-Inhalte abgeglichen. Direkte Nutzung plus
   eindeutige lokale Bindung bestätigt ein Symbol; indirekte Nutzung bleibt unklar.
6. Gesamtergebnis erst nach vollständiger Risikoanalyse ausgeben; Gründe und Testhinweise
   stabil sortieren, Quelltext und Geheimniswerte nie ausgeben.

## Verification Strategy

- FR-001/002/011/012: Integrationstests in temporären Repositorys; invalid refs,
  Dirty Worktree, fehlende Blobs, ungewöhnliche Pfade, Gitlink/Binärdatei und Limits.
- FR-003–007/010: alle Referenzpaare aus rules.md, gemischte Stufen, unbekannter Bereich
  neben HIGH, zehn Wiederholungen mit byteidentischer Ausgabe.
- FR-008/009/014/015: Testfall-/Receiver-Matrix aus test-association.md in allen vier Profilen.
- FR-004/013: exakte stdout/stderr-/Exit-Verträge, escaped Pfade und Secret-Redaktion.
- Packaging: Installation aus Wheel mit vorgebauten Parser-Wheels; CLI läuft außerhalb
  Quellbaum sowie im Unterverzeichnis eines analysierten Repositorys. Laufzeittest offline.

## Complexity Tracking

Keine Constitution-Verletzungen. Die vier Syntaxparserpakete sind durch präzise
Testfallgrenzen, Kommentar-/String-Abgrenzung und Klassen-/Methodennutzung begründet.
PyYAML ersetzt einen eigenen fehleranfälligen YAML-Teilparser. Ein eigener Parser
für vier Sprachen wäre mehr Code und schlechter überprüfbar.
