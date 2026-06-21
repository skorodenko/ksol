# Ksol Agent Instructions

This repository is a Rust-based media player that integrates with MPD and uses Qt/QML for its GUI.

## Tech Stack & Architecture
- **Language**: Rust
- **GUI**: Qt/QML (via `cxx-qt` bindings)
- **Backend**: MPD (Music Player Daemon) integration, Mpris server.
- **Persistence**: 
  - `settings.toml` for user configuration (XDG config home).
  - `state.bin` for UI state (XDG data home), serialized via `wincode`.
- **Async Runtime**: `tokio`

## Key Directories
- `src/qt/`: Rust implementations of Qt modules (MPD connector, Mpris service).
- `src/qml/`: QML files for the frontend UI.
- `src/service/`: Business logic for MPD actions and Mpris integration.
- `src/utils/`: Global state, persistence logic, and directory initialization.

## Build & Execution
- **Build**: `cargo build`
- **Run**: `cargo run`
- **Styles**: `QT_QUICK_CONTROLS_STYLE` environment variable controls the UI style (defaults to `org.kde.desktop`).
- **Logging**: Uses `tracing`. Default filter is `mpd_protocol=error`. Use `RUST_LOG=debug` for more verbose output.

## Important Quirks & Details
- **Resource Paths**: QML is loaded from `qrc:/qt/qml/github/skorodenko/ksol/src/qml/Main.qml`.
- **MPD Connection**: The application attempts to connect via both TCP and Unix sockets.
- **Native MPD**: If using a native server, it generates an `mpd.conf` from a template in `src/utils/init_hooks.rs`.
- **Directory Setup**: `init_dirs()` ensures XDG configuration, data, and cache directories exist upon first run.

## Verification Steps
- Check `src/qt/mpd.rs` for QML/Rust signal and property mappings.
- Check `src/utils/persist.rs` to see how `settings.toml` and `state.bin` are loaded/dumped.
- Verify `build.rs` for QML module registration and file includes.
