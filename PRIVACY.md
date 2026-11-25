# Privacy Policy — MOTAA Desktop

**Effective Date:** November 25, 2026

MOTAA Desktop is an offline-first desktop application for Myanmar astrology calculations and chart visualization.

## Data Storage
- **Local-Only Storage:** Birth profiles, calculation settings, and revision history are stored locally on your device using SQLite (`~/Library/Application Support/MOTAA Desktop GPUI/profiles.sqlite3` on macOS, `$XDG_DATA_HOME/motaa-desktop` on Linux, and `%LOCALAPPDATA%\MOTAA Desktop` on Windows).
- **No Accounts or Cloud Sync:** MOTAA Desktop does not require an account and does not upload your saved profiles or charts to any developer server.

## Calculations & Offline Features
- Ephemeris calculations, Myanmar calendar conversions, the 534-town Myanmar directory, and coordinate-to-timezone resolution (`tzf-rs` and `chrono-tz`) execute entirely locally on your device.

## Optional Network Requests
Network requests are made **only** when you explicitly invoke the following optional location tools inside the Profile or Reference Location editor:
1. **Search OpenStreetMap:** Sends your location search query to `https://nominatim.openstreetmap.org` to retrieve matching coordinates and place names.
2. **Pick on map:** Downloads map tiles from `https://tile.openstreetmap.org` (cached locally on your device) and queries `nominatim.openstreetmap.org` for reverse geocoding when you drop a pin.

These requests are governed by the [OpenStreetMap Foundation Privacy Policy](https://osmfoundation.org/wiki/Privacy_Policy).

## Telemetry & Third-Party Tracking
- MOTAA Desktop contains **no** analytics, crash reporters, advertising SDKs, or telemetry.

## Contact
For privacy questions or issues, please open an issue at [github.com/AungMyoKyaw/motaa-desktop/issues](https://github.com/AungMyoKyaw/motaa-desktop/issues).
