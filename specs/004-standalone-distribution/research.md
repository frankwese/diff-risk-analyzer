# Research: Standalone-Auslieferung

**Decision**: Native Darwin-Dateien und statische Linux-musl-Executables aus derselben Crate.
**Rationale**: Explizite Rust-/Standalone-Entscheidung und vier Zielarchitekturen erhalten.
**Alternatives considered**: Python-Bündelung/Go/JVM bereits verworfen; kein neues Packagingframework.

Originalquellen: [001 research.md](../001-diff-risk-analysis/research.md), Abschnitt 1.
Vorliegende frühere Linux-Builds und Intel-macOS-Cross-Build sind Historie, keine aktuelle
Vier-Ziel-Freigabe. Geeignete Zielrunner sind Umgebungsanforderung und expliziter Blocker.
Keine neuen Architekturfragen; konkrete Releasearbeit erst am Funktionsmeilenstein.
