# Quickstart and Validation Guide

Dieser Guide beschreibt die geplante Anwendung nach Implementierung. Die folgenden
Anwendungsbefehle sind heute noch nicht verfügbar und wurden nicht als E2E-Test ausgeführt.

## Prerequisites and setup

macOS/Linux, Python 3.12–3.14, Git >=2.49. Parser-Wheels für Zielplattform erforderlich.
Installation kann Internet benötigen; Analyse läuft danach ausschließlich lokal.

Aus Projektroot nach Implementierung:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install .
.venv/bin/risk-analyzer --help
.venv/bin/risk-analyzer --version
```

`--version` nennt Anwendungsversion und Regelstand; `--help` beschreibt BASE..TARGET.
Keine Installation von Code aus dem zu analysierenden Fremdrepository.

## Run checks

```sh
.venv/bin/python -m unittest discover -s tests -v
.venv/bin/python -m compileall -q src/risk_analyzer
```

Regeltests: 14 positive/negative Paare plus Grenzfälle aus [rules.md](contracts/rules.md).
Testzuordnung: Matrix aus [test-association.md](contracts/test-association.md).
CLI/Git-Tests erstellen und entfernen selbst temporäre Repositorys; lokale Projektdateien
werden nicht zum Zweck der Validierung geändert. Git-Konfiguration nur innerhalb Testrepos.

## End-to-end scenarios

In einem separaten lokalen Repository mit zwei bekannten Commits aufrufen; CLI vorher
installieren und über absoluten Installationspfad starten:

```sh
/ABSOLUTER/INSTALLATIONSPFAD/risk-analyzer HEAD~1..HEAD
```

| Vorbereitung in Testrepository | Erwartetes Ergebnis |
| --- | --- |
| Nur README geändert | Risk LOW, DOC_CHANGE, Exit 0 |
| package.json dependency version geändert | Risk MEDIUM, DEPENDENCY_UPDATE mit Pfad, Exit 0 |
| src/auth/Guard.java ausführbare authorize-Methode geändert | Risk HIGH, AUTH_CHANGE; fehlende geänderte Tests separat |
| migrations/V2.sql DROP TABLE ergänzt | Risk HIGH, DESTRUCTIVE_MIGRATION, keine Testabdeckungsbehauptung |
| Passender GuardTest-Test geändert, Guard direkt konstruiert und authorize aufgerufen | Gleiches HIGH; USED mit Testfall/Fundstelle |
| Passender Testname, aber nur andere Methode verwendet | MISSING für authorize |
| Test B geändert, nur unveränderter Test A nutzt authorize | MISSING; Test A zählt nicht |
| Fixture/Helper liefert Receiver | UNKNOWN mit Ursache, Risikoanalyse weiterhin erfolgreich |
| Identische Revisionen | LOW und No changes; Exit 0 |
| Unbekannte Revision/außerhalb Repository | stdout leer, Fehler auf stderr, Exit 2 |
| Binäränderung oder unbekannte Region neben HIGH | stdout leer, UNSUPPORTED_CHANGE, Exit 1 |

Exakte Format-/Sortierungsregeln stehen in [cli.md](contracts/cli.md).

## Offline and read-only validation

Erfolgsfälle nach Installation ohne Netzwerk erneut ausführen. Vorher/nachher HEAD,
Branch, Index, Worktree-Status und Repositorydateien vergleichen: unverändert.
External-Diff-/Textconv-/fsmonitor-Helfer im temporären Testrepository als Markerprobes
konfigurieren; Analyse darf keine Probe ausführen. Partial-Clone-Objekt fehlt: Fehler,
kein Lazy-Fetch. Jeden Referenzdiff zehnmal ausführen und stdout byteweise vergleichen.

## Packaging and limits

Wheel mit `pip wheel --no-deps .` bauen und in frischer Umgebung installieren; Parser-
Wheels vorab beschaffen, anschließend offline installieren/ausführen. Plattformen und
Python-Versionen nur nach erfolgreichem Packaging-/Probecheck als geprüft dokumentieren.
Grenzfallfixtures für Limits aus cli.md müssen ohne Teil-Risk abbrechen. Repräsentativen
100-Dateien-/10.000-Zeilen-Fall einmal hinsichtlich Zeit/RSS messen und Maschine nennen.
