# Security Policy

## Local-First Architecture & Network Behavior

MOTAA Desktop stores birth profiles, calculation preferences, and revision history locally in SQLite (`~/Library/Application Support/MOTAA Desktop GPUI/profiles.sqlite3` on macOS, `$XDG_DATA_HOME/motaa-desktop` on Linux, `%LOCALAPPDATA%\MOTAA Desktop` on Windows).

All chart calculations, ephemeris lookups, timezone resolution (`tzf-rs` + `chrono-tz`), and the 534-town Myanmar directory run **offline** by default.

Optional network requests occur **only** when you explicitly use:
1. **Search OpenStreetMap**: queries `nominatim.openstreetmap.org` with your search text.
2. **Pick on map**: fetches map tiles from `tile.openstreetmap.org` (cached locally in `~/Library/Caches/MOTAA-Desktop/osm-tiles/`) and performs reverse geocoding on pin drop.

No telemetry, analytics, accounts, or crash reporters are included.

## Reporting a Vulnerability

If you discover a security issue (such as unsafe backup/SQLite parsing or path traversal), please report it privately using [GitHub Security Advisories](https://github.com/AungMyoKyaw/motaa-desktop/security/advisories/new) rather than opening a public issue.
