# Changelog

All notable changes to MOTAA Desktop are documented in this file.

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
