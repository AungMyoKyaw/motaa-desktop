# MOTAA Desktop — source-parity correction review

**Executed:** 20 November 2026
**Canonical native master:** design/motaa-desktop.figcraft
**Promoted SHA-256:** 1bb0d3ba3274b6f5b00c5a0db9eb2b4952d33640aaf11e5ea1e8d5de85331b99
**Scope:** Static native design and implementation contracts; no runtime numerical parity claim.
**Recovery:** ~/.local/share/motaa-desktop-backups/source-parity-20261120/pre-parity.figcraft
**Edit method:** Five staged FigCraft CLI run passes; every operation succeeded before promotion.

## Corrected source fidelity

| Area | Correction |
|---|---|
| Rahu wheel | Original N0() radii 240 / 220 / 180 / 125 / 60 now share circular mode's 550/480 scale. All five rings remain concentric at (505, 570). |
| Source canvas bases | Page 89 records all I0() output dimensions and the uniform-width scaling contract. Desktop frames remain 1440 × 960 product screens; source output aspect ratios are not implied to be desktop window sizes. |
| Chart palettes | MOTAA Light / Pink / Blue / Monochrome variables now use the recovered gold, pink, blue and black/white resource values. Gold Heritage / UI remains independent. |
| Moon fixture | The independently verified 195.24696056° Moon is consistently Libra sign 7 and Swati across all 18 Rasi renderers and the related Nakshatra/circular references. |
| Western notation | Original Astro font mappings replace Unicode approximations in the aspect matrix and planet legend. The recovered font is packaged at design/fonts/Astro.ttf. |
| Epoch states | N+N, N+T, T+N and T+T each have one native hotspot that sets planet/Ascendant epoch values and opens the correct source-state page. Duplicate overlapping actions were removed. |

## Data and provenance

- Android's two serialized-profile stores, profiles.db and commments.db, are identified in the profile screens. The comments table contains serialized profile records, not a separate free-form notes feature.
- All 534 source towns, coordinates and source indices are exported without invented entries to design/source/locations.v31.1.json. Page 32 states the actual directory size.
- Backup/restore remains available as a clearly labeled desktop extension, not recovered Android V31.1 behavior.
- Obsolete session/member entitlement state was removed. All original modes remain available in the desktop edition.

## Verified current structure

- 104 editable pages and 24,484 document nodes.
- 2,252 valid native navigation actions and 67 native variable-selection actions.
- 31 shared styles and 75 native variables in three collections.
- Exactly one FigCraft master; no missing fonts reported with the packaged Astro font installed.
- All 104 PNG previews, tokens.css, source/screen-manifest.json and index.html regenerated from the promoted master.
- 27/27 FigCraft regression tests pass, including the new Rahu geometry, source-data, fixture, font and epoch-navigation checks.
- Affected screens were rendered and visually inspected, including charts, profile storage, location directory, backup provenance, all four epoch combinations, Western notation and source canvas contract.

## Remaining limits

The master remains a static specification. Illustrative non-Moon positions and unverified advanced outputs are not independently validated astronomy. The location JSON is design data, not proof of a running picker or persistence layer. Original APK calculations, database migration, live variable behavior and desktop runtime controls still require implementation tests.
