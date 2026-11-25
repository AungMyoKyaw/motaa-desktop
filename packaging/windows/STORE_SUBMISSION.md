# Microsoft Store Submission Guide (MOTAA Desktop)

This repository includes an MSIX manifest template, Store image assets, and a packaging script for submitting **MOTAA Desktop** to Microsoft Partner Center.

## 1. Prerequisites in Microsoft Partner Center
1. Register or sign in to [Microsoft Partner Center](https://partner.microsoft.com/dashboard).
2. Reserve the app name **MOTAA Desktop**.
3. Copy your Publisher ID (`CN=...`) from **Product management -> Product Identity**.

## 2. Build the MSIX Package
The GitHub Release workflow (`v*` tags) automatically builds `motaa-desktop-v<version>-windows-x64.msix` alongside the portable `.zip`.

If your Partner Center Publisher ID differs from `CN=Aung Myo Kyaw`, pass it when building locally on Windows:
```powershell
.\packaging\windows\build-msix.ps1 `
  -ExePath .\target\release\motaa-desktop.exe `
  -Version 0.2.0 `
  -Publisher "CN=YOUR-PARTNER-CENTER-GUID" `
  -OutputMsix .\dist\motaa-desktop-0.2.0-windows-x64.msix
```
Microsoft Store signs the uploaded `.msix` with its own Store certificate during ingestion, so no paid EV code-signing certificate is required for Store submission.

## 3. Store Listing Fields

| Field | Value |
| --- | --- |
| App Name | MOTAA Desktop |
| Category | Utilities & tools (or Reference) |
| Privacy Policy URL | `https://github.com/AungMyoKyaw/motaa-desktop/blob/master/PRIVACY.md` |
| Website / Support URL | `https://github.com/AungMyoKyaw/motaa-desktop` |
| 300x300 Store Icon | `packaging/windows/assets/StoreIcon300.png` |

### Desktop Screenshots (`docs/screenshots/`)

| File | Description |
| --- | --- |
| `01-rasi-chart.png` | Traditional Myanmar Rasi chart with MUNP overlay and contextual inspector |
| `02-tricakka-chart.png` | Tricakka triple-chart view (Rasi, Bhava, Navamsa) |
| `03-dasha-periods.png` | Planetary Dasha periods hierarchy |
| `04-time-analysis.png` | Independent transit time and location navigator |
| `05-profiles.png` | Saved birth profiles with backup and import |
| `06-advanced-tools.png` | Advanced astrological and astronomical tools directory |

### Short Description
> Offline-first Myanmar astrology workspace with traditional square and circular charts, birth profile management, OpenStreetMap location picker, and IANA timezone selector.

### Capabilities Justification
- `runFullTrust`: Native Win32/GPUI desktop application packaged via Desktop Bridge (MSIX).
- `internetClient`: Used only when the user explicitly searches OpenStreetMap Nominatim or opens the interactive map tile picker.
