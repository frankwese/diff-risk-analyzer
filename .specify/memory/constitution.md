# diff-risk-analyzer Constitution

## Core Principles

### I. Nachvollziehbare Anforderungen

Für jedes neue Feature MÜSSEN vor der Implementierung Ziel, Umfang und überprüfbare
Akzeptanzkriterien in einer Spezifikation festgehalten werden. Annahmen MÜSSEN als solche
gekennzeichnet werden. Entscheidungen, die Umfang oder Akzeptanzkriterien verändern,
MÜSSEN in den zugehörigen Spec-Kit-Artefakten nachgeführt werden. Fehlerkorrekturen
DÜRFEN mit einer dokumentierten Fehlerbeschreibung und einem überprüfbaren Sollverhalten
statt einer vollständigen Feature-Spezifikation erfolgen.

### II. Kleinste vollständige Lösung

Änderungen MÜSSEN den vereinbarten Umfang vollständig erfüllen und vorhandene Strukturen
und Konventionen nutzen. Neue Abhängigkeiten oder Abstraktionen MÜSSEN mit einem konkreten
Bedarf begründet werden, der nicht durch bestehende Mittel erfüllt werden kann.
Vorsorgliche Erweiterungen ohne aktuelle Anforderung DÜRFEN NICHT eingeführt werden.
Damit bleiben Änderungen verständlich, wartbar und gezielt überprüfbar.

### III. Überprüfbare Korrektheit

Neue oder geänderte nichttriviale Logik MUSS durch automatisierte Prüfungen ihres
beobachtbaren Verhaltens abgesichert werden. Fehlerkorrekturen MÜSSEN einen
Regressionstest enthalten, sofern der Fehler automatisiert reproduzierbar ist;
andernfalls MÜSSEN reproduzierbare manuelle Prüfschritte dokumentiert werden.
Betroffene Schnittstellen und Integrationen MÜSSEN bei Vertragsänderungen geprüft werden.
Reine Dokumentations- oder Formatierungsänderungen benötigen keine neuen Verhaltenstests.
Ausgeführte Prüfungen, Ergebnisse und verbleibende Prüflücken MÜSSEN im Abschlussbericht
genannt werden; fehlende Prüfungen DÜRFEN NICHT als erfolgreich dargestellt werden.

### IV. Explizite Eingaben und Fehler

Externe Eingaben MÜSSEN an Systemgrenzen validiert werden. Ungültige oder unvollständige
Eingaben und fehlgeschlagene Verarbeitung MÜSSEN als Fehler oder ausdrücklich
gekennzeichnetes Teilergebnis erkennbar sein; sie DÜRFEN NICHT stillschweigend ein
erfolgreiches Ergebnis erzeugen. Fehlermeldungen MÜSSEN die betroffene Operation und
eine handlungsfähige Ursache nennen. Zugangsdaten DÜRFEN NICHT versioniert werden;
Diagnoseausgaben DÜRFEN keine Zugangsdaten enthalten.

### V. Nachvollziehbare Ergebnisse und Änderungen

Ergebnisse MÜSSEN bestätigte Fakten, Annahmen und nicht verfügbare Informationen
unterscheiden. Für abgeleitete Bewertungen MÜSSEN die entscheidungsrelevanten Eingaben
und Regeln nachvollziehbar sein. Bei unveränderten Eingaben und Regeln MÜSSEN
deterministische Verarbeitungsschritte dieselben Ergebnisse liefern. Abweichungen durch
nichtdeterministische Verfahren MÜSSEN dokumentiert werden. Änderungen an öffentlichen
Schnittstellen oder Bewertungsregeln MÜSSEN einschließlich ihrer Auswirkungen dokumentiert
werden; inkompatible Änderungen benötigen Hinweise zur Anpassung bestehender Nutzer.

## Projektgrenzen

Diese Constitution legt Entwicklungsregeln für `diff-risk-analyzer` fest. Produktumfang,
Programmiersprache, Laufzeit, Bedienoberfläche und Betriebsmodell werden in den jeweiligen
Spezifikationen und Plänen entschieden. Eine lokale IDE-Konfiguration gilt nicht als
Architekturentscheidung. Technische Entscheidungen MÜSSEN mit den Feature-Anforderungen
begründet und im Plan festgehalten werden.

## Entwicklungsablauf und Qualitätsprüfungen

Neue Features folgen dem Spec-Kit-Ablauf Spezifikation → Plan → Aufgaben → Implementierung
→ Konvergenzprüfung. Ungeklärte Fragen, die das überprüfbare Sollverhalten bestimmen,
MÜSSEN vor der betroffenen Implementierung geklärt werden.

Vor dem Abschluss einer Änderung MÜSSEN:
- die betroffenen Akzeptanzkriterien und Aufgaben mit der Umsetzung abgeglichen werden;
- die relevanten verfügbaren Tests, Build- und statischen Prüfungen ausgeführt werden;
- der Diff auf unbeabsichtigte Änderungen und Verstöße gegen diese Constitution geprüft werden;
- verbleibende Aufgaben, bekannte Einschränkungen und blockierte Prüfungen dokumentiert werden.

Ein Feature DARF erst als vollständig abgeschlossen bezeichnet werden, wenn alle vereinbarten
Akzeptanzkriterien erfüllt und ihre erforderlichen Prüfungen erfolgreich sind.

## Governance

Diese Constitution hat Vorrang vor widersprechenden Projektplänen und Arbeitskonventionen.
Jeder Feature-Plan und jede abschließende Änderungsprüfung MUSS ihre Einhaltung prüfen.
Abweichungen MÜSSEN mit Grund, Geltungsbereich und Auswirkungen dokumentiert und vor der
betroffenen Umsetzung durch den Projektverantwortlichen ausdrücklich genehmigt werden.

Änderungen an dieser Constitution MÜSSEN die geänderten Regeln, den Änderungsgrund und
Auswirkungen auf bestehende Spezifikationen und Pläne benennen. Der Projektverantwortliche
MUSS Änderungen genehmigen; ein ausdrücklicher Auftrag zur Erstfassung oder Änderung gilt
als Genehmigung innerhalb des beauftragten Umfangs. Betroffene laufende Arbeiten MÜSSEN
vor ihrer Fortsetzung auf Anpassungsbedarf geprüft werden.

Die Version folgt `MAJOR.MINOR.PATCH`: MAJOR für inkompatible Aufhebungen oder
Neudefinitionen von Regeln, MINOR für neue Prinzipien oder wesentlich erweiterte Vorgaben,
PATCH für Klarstellungen ohne Bedeutungsänderung. Jede Änderung MUSS Version und
Änderungsdatum aktualisieren; das ursprüngliche Ratifikationsdatum bleibt bestehen.
Der Sync Impact Report ist vorläufiges Prüfmaterial und MUSS nach der Änderungsprüfung
vor dem Commit entfernt werden.

**Version**: 1.0.0 | **Ratified**: 2026-10-13 | **Last Amended**: 2026-10-13
