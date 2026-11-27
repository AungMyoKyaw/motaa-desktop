# Changelog

All notable changes to MOTAA Desktop are documented in this file.

## [0.2.3] - 2026-11-28

### Changed
- **Updated Windows Store & MSIX App Icons**: Regenerated all Windows tile, splash screen, and Store icon assets from the new `motaa-icon-1024.png` artwork.

## [0.2.2] - 2026-11-28

### Added
- **About & Credits Workspace**: Added a dedicated About & Credits page with origin story, attributions, and creator links (`07-about.png`).
- **Dedicated Modal Pickers**: Added modal popups for the calendar date picker, 3-column time picker (with keyboard entry, nudges, and Cancel/Done), Myanmar 534-town directory (with full scrollable list), timezone chooser, and OpenStreetMap search/pin-on-map picker.
- **Cross-Platform Text Input Polish**: Added double-click select-all, word navigation, and line delete across macOS, Windows, and Linux.
- **Windows 11 Store Screenshots**: Refreshed all 7 app screenshots including high-resolution Windows 11 Store screenshots (`packaging/windows/screenshots/`).

### Changed
- **Timezone Engine**: Migrated from `chrono`/`chrono-tz` to `jiff 1.0.3` with bundled timezone database.
- **License**: Switched project license to `AGPL-3.0-only`.
- **UI Polish**: Removed residual engineering/provenance labels, chart color palette switcher, and in-element shortcut clutter; blocked scroll-through on modal backdrops.

## [0.2.1] - 2026-11-27

### Added
- **Automated Windows Store Screenshot Capture**: Added `--store-screenshots <dir>` mode and `packaging/windows/capture-store-screenshots.ps1` to capture genuine Windows-native screenshots (≥1366×768) in CI/CD alongside the MSIX submission bundle.
- **Manual Release Pipeline Dispatch**: Added `workflow_dispatch` support to `.github/workflows/release.yml` for generating and validating Store submission artifacts on demand.

### Fixed
- **Native Windows Chrome & Shortcuts**: Replaced macOS-only `⌘`/`Cmd` UI labels and `cmd-` keybindings with platform-native `Ctrl+` shortcuts on Windows/Linux, and switched non-macOS windows to a standard native OS titlebar without macOS traffic-light spacing.

## [0.2.0] - 2026-11-25

### Added
- **Observatory Manuscript UI**: 3-column desktop workbench with collapsible navigation rail, contextual right inspector, and `Cmd+K` command palette.
- **Interactive OpenStreetMap Location Picker**: Global place search via OpenStreetMap Nominatim and a native GPUI tile map (`z2` to `z16`) with pan, zoom, and click-to-drop-pin reverse geocoding.
- **Searchable IANA Timezone Selector**: Filterable `chrono-tz` directory with common-zone quick picks, offline coordinate-to-timezone detection (`tzf-rs`), and live historical UTC offset preview for the chosen birth or reference date.
- **Public Release & CI Workflows**: GitHub Actions CI for `cargo fmt`, `clippy`, and `cargo test` on macOS and Ubuntu, plus public cross-platform release packaging.

### Fixed
- Expanded `TextInput` inside flex containers so search bars and inline fields keep full available width.
- Aligned table headers and clipped long place names across profile and tabular data rows.
- Kept active sidebar labels single-line and prevented `Tab` focus leakage behind the command palette.

## [0.1.1] - 2026-11-24

### Added
- Initial desktop workspace with 5 navigation routes, 12 chart modes, 16 advanced tools, 534-town offline Myanmar directory, SQLite profile storage, and JSON / Android backup import-export.
