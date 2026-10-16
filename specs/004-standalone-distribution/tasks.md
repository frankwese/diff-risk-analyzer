---
description: "Migrierte Aufgaben für native Vier-Ziel-Auslieferung"
---

# Tasks: Standalone-Auslieferung

**Input**: spec/plan/research/data-model.md, contracts/release.md; freizugebender Umfang 002/003.
**Tests**: Aktuelle Artefakte/Zielruntime; alte Buildnachweise nicht zu Erfolg hochstufen.

## Format and Path Conventions

Lokale T-IDs; Cargo-Releaseausgaben unter ignoriertem target/, Nachweise in status.md.

## Phase 1: Setup

- [ ] T001 Lege Quellstand/Funktionsumfang/Version und vier Targetvoraussetzungen in `status.md`, `README.md` fest; vorhandenes `Cargo.toml`/Lockfile weiterverwenden (Releaseanteil alt T003).

## Phase 2: Foundational

002-Abschluss vor Kernfreigabe, zusätzlich 003 vor Vollprodukt. Geeignete native/musl-
Runner bereitstellen; fehlenden Intel-macOS-Runner als Blocker in status.md dokumentieren.

## Phase 3: US1 — Laufzeitfreies Starten

**Goal**: Passendes aktuelles Executable ohne Rust/Python/JVM/externen Parser.
**Independent Test**: Saubere Runtime mit Git, offline, eigene Kern-Referenzfälle.

- [ ] T002 [P] [US1] Definiere wiederverwendbare Release-Smokes aus bestehenden `tests/cli.rs` und 002-Fällen in `quickstart.md`; Artefakt außerhalb Quellbaum und ohne Sprachlaufzeit ausführen (Releaseanteil alt T031).
- [ ] T003 [P] [US1] Definiere Vollprodukt-Smokes aller vier 003-Sprachprofile in `contracts/release.md`, `status.md`; Testumfang nicht schon durch Parser-Build als erfüllt ansehen (Releaseanteil alt T031).
- [ ] T004 [US1] Baue aktuelles macOS-arm64-Executable aus `Cargo.toml`, prüfe Verknüpfung/Offline-Smokes und dokumentiere Commit/Artefakt in `status.md` (alt T033).

## Phase 4: US2 — Vier geprüfte Zielartefakte

**Goal**: Nachvollziehbare aktuelle Zielmatrix statt bloßer Cross-Builds.
**Independent Test**: Je Artefakt Zielruntime und Prüfumfang belegt, blockierte Ziele offen.

- [ ] T005 [US2] Baue x86_64-apple-darwin und führe aktuelle Tests auf geeignetem Runner aus; Quellstand/Verknüpfung und fehlende Umgebung in `status.md` belegen (Teil alt T034).
- [ ] T006 [US2] Baue aarch64-unknown-linux-musl und prüfe statisches Artefakt in sauberer Zielruntime gemäß `quickstart.md`; Ergebnisse in `status.md` (Teil alt T034).
- [ ] T007 [US2] Baue x86_64-unknown-linux-musl und prüfe statisches Artefakt gemäß `quickstart.md`; Emulation/native Ausführung und Quellstand in `status.md` getrennt belegen (Teil alt T034).

## Phase 5: Polish

- [ ] T008 Finalisiere Standaloneinstallation/Target-/Gitvoraussetzungen und Release-Matrix in `README.md`, `quickstart.md`, `status.md`; Docker nur Buildtechnik, keine Nutzerabhängigkeit (Anteil alt T035).
- [ ] T009 Prüfe 004-FR-001–006/SC-001–004 und Freigabe aktueller Artefakte in `status.md`; vier Zielruntime-Nachweise, keine falsche Freigabe bei alten/fehlenden Nachweisen (Anteil alt T036).

## Dependencies & Execution Order

002 → Kern-Smokes/Builds; 002+003 → Vollprodukt-Smokes. T002/T003 getrennte Dokumente
parallel vorbereitbar; T005–T007 in getrennten Zielumgebungen nach T001–T003 parallel
prüfbar, Ergebnisse danach sequenziell in status.md zusammenführen. Keine wiederholten
Zielbuilds während noch laufender Funktionsänderungen.

## Implementation Strategy

Gezielter aktueller Release-Meilenstein nach Funktionsabschluss. 9 offene Aufgaben;
historische Buildproben reichen für keine vollständige erledigte Aufgabe. Fehlender
Runner blockiert nur dessen Freigabe, nicht 002/003.
