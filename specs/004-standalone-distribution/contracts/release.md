# Standalone Release Contract

**Owner**: 004. Kernverhalten gehört 002, Testhinweise 003.

| Target | Format und Verknüpfung | Benötigter Nachweis |
| --- | --- | --- |
| aarch64-apple-darwin | Native macOS-Datei, nur OS-Systembibliotheken | Aktueller Build und arm64-Laufzeitfälle |
| x86_64-apple-darwin | Native macOS-Datei, nur OS-Systembibliotheken | Aktueller Build und x86_64-Laufzeitfälle |
| aarch64-unknown-linux-musl | Statisches Executable einschließlich Parser | Aktueller Build, Verknüpfung und arm64-Laufzeitfälle |
| x86_64-unknown-linux-musl | Statisches Executable einschließlich Parser | Aktueller Build, Verknüpfung und x86_64-Laufzeitfälle |

Artefaktname identifiziert Version/Target; mit dokumentiertem Commit/Quellstand und
Prüfumfang. Nutzer brauchen Git >=2.49 und OS, keine Sprachlaufzeit/Compiler/externen
Grammatikdateien. Buildsystem benötigt Rust 1.99.0, Cargo und passenden C-/musl-Linker.

Kernrelease: vereinbarte 002-Fälle auf allen vier Zielen. Vollprodukt: zusätzlich 003-
Matrix mit direkter Nutzung in Java/JS/TS/Python. Ein älterer Build, Emulationsnachweis
oder Cross-Build ist als solcher zu kennzeichnen und nicht als native Zielprüfung auszugeben.

Docker kann eine Build-/Prüfumgebung sein, niemals Nutzerlaufzeitvoraussetzung.
Finale Prüfung offline in sauberer Laufzeitumgebung; fehlende Runner als Blocker ausweisen.
