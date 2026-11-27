# Microsoft Store Submission Guide (MOTAA Desktop)

This repository includes an MSIX manifest template, Store image assets, a packaging script, and an automated Windows screenshot capture script for submitting **MOTAA Desktop** (`Product ID: 9NRP9CQZHPMZ`) to Microsoft Partner Center.

## 1. Prerequisites in Microsoft Partner Center
1. Register or sign in to [Microsoft Partner Center](https://partner.microsoft.com/dashboard).
2. Open **MOTAA Desktop** (`9NRP9CQZHPMZ`).
3. Verify your Publisher ID (`CN=F9A61C2C-016B-4569-AF3E-D8FB4CBCB150`) in **Product management -> Product Identity**.

## 2. CI/CD Submission Bundle (`windows-store-submission`)
The **Desktop Release** GitHub Actions workflow (`.github/workflows/release.yml`, triggered on `v*` tags or manual `workflow_dispatch`) builds and uploads a complete `windows-store-submission` artifact on `windows-2025`:

- `motaa-desktop-v<version>-windows-x64.msix`: Desktop Bridge MSIX package (signed automatically by Microsoft Store during ingestion)
- `screenshots/01-rasi-chart.png` .. `07-about.png`: Genuine Windows-native screenshots (≥1366×768 PNG, native Windows titlebar, `Ctrl+` shortcuts, zero macOS UI)
- `assets/StoreIcon300.png`: 300×300 Store icon
- `SUBMISSION_MANIFEST.json`: Version, SHA-256 checksums, and screenshot dimensions

To build locally on Windows:
```powershell
.\packaging\windows\build-msix.ps1 `
  -ExePath .\target\release\motaa-desktop.exe `
  -Version 0.2.5 `
  -OutputMsix .\dist\motaa-desktop-v0.2.5-windows-x64.msix

.\packaging\windows\capture-store-screenshots.ps1 `
  -ExePath .\target\release\motaa-desktop.exe `
  -OutputDir .\dist\store-screenshots
```

## 3. Store Listing Fields

| Field | Value |
| --- | --- |
| App Name | MOTAA Desktop |
| Product ID | `9NRP9CQZHPMZ` |
| Category | Utilities & tools (or Reference) |
| Privacy Policy URL | `https://github.com/AungMyoKyaw/motaa-desktop/blob/master/PRIVACY.md` |
| Website / Support URL | `https://github.com/AungMyoKyaw/motaa-desktop` |
| 300x300 Store Icon | `packaging/windows/assets/StoreIcon300.png` |

### Desktop Screenshots (Windows-native, from `windows-store-submission/screenshots/`)

| File | Description |
| --- | --- |
| `01-rasi-chart.png` | Traditional Myanmar Rasi chart with MUNP overlay and contextual inspector |
| `02-tricakka-chart.png` | Tricakka triple-chart view (Rasi, Bhava, Navamsa) |
| `03-dasha-periods.png` | Planetary Dasha periods hierarchy |
| `04-time-analysis.png` | Independent transit time and location navigator |
| `05-profiles.png` | Saved birth profiles with backup and import |
| `06-advanced-tools.png` | Advanced astrological and astronomical tools directory |
| `07-about.png` | About & Credits page with origin and creator links |

Pre-generated Windows-chrome Store screenshots (`2880×1800` PNG) are also saved locally in `packaging/windows/screenshots/`.

> **Policy 10.1.1.3 Compliance**: Never upload macOS screenshots from `docs/screenshots/` to Microsoft Partner Center. Always upload the Windows-captured screenshots from the `windows-store-submission` CI/CD artifact.

### Short Description
> Offline-first Myanmar astrology workspace with traditional square and circular charts, birth profile management, OpenStreetMap location picker, and IANA timezone selector.

### Capabilities Justification
- `runFullTrust`: Native Win32/GPUI desktop application packaged via Desktop Bridge (MSIX).
- `internetClient`: Used only when the user explicitly searches OpenStreetMap Nominatim or opens the interactive map tile picker.
