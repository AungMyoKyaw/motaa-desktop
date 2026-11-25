# MOTAA Desktop — source-parity design review

> Historical review. The current correction record is SOURCE_PARITY_20261120_REVIEW.md; counts and preservation claims below describe the earlier artifact.

**Executed:** 17 November 2026
**Canonical native master:** design/motaa-desktop.figcraft
**Status:** Updated, rendered and regression-verified; design-only, runtime pending.
**Original SHA-256:** 14ff1f1d51425dbd768325314751892ede0f6aa9d27624dee23c59346a90b05d
**Current SHA-256:** 5be8da25e8e237f1c294ac08f081f378851a1a2327b7855964fae91d17a4e407
**Backup:** ~/.local/share/motaa-desktop-backups/source-parity-20261117/pre-parity.figcraft

## Implemented native design additions

| Page | Prototype role | Source evidence | Output |
| --- | --- | --- | --- |
| 101 Myanmar lunar calendar | Gregorian monthly layout, Myanmar year/month/phase placeholders, observance inspector | MOTAA Android main.A0 and AbstractC0029c | previews/101-myanmar-lunar-calendar.png |
| 102 Original calculation controls | Gregorian, Julian, Hybrid, source-stored 5D_Aspect/15D_Aspect, original other calculation options | MOTAA Android source and reverse-engineering report | previews/102-original-calculation-controls.png |
| 103 Traditional observances | Sabbath/Eve, Yatyaza, Pyathada, Thamanyo and the remaining traditional day flags | MOTAA main.A0 and C0025a | previews/103-traditional-observances.png |
| 104 Sky map - sibling reference | Explicitly separate 2D/3D iZarTarPro schematic, RA/Dec reference and Star/Name/lat/lon/Sign/Plan/Zod layers | iZarTarPro screenshot/metadata report; not original Android engine | previews/104-sky-map-sibling-reference.png |

## Verified invariants

- **104 editable pages** = 94 app/reference pages + 10 original learning pages; **five primary flows**, not four additional routes.
- **23,622 native nodes**, **23 shared styles**, **53 native variables**, **113 native component masters**, **19 component families**, **505 linked instances**.
- **2,252 valid navigation actions**, **35 native variable-selection actions**; every 1–104 reference page is reachable from the Charts start flow.
- All **22,812 original native nodes retain their properties**; only frame-child lists on nine earlier source pages were extended to add relevant navigation entries. Original chart geometry and lesson pages were not altered.
- Exactly one FigCraft master; no duplicate design documents within the repository. No missing fonts reported.
- **104/104 native PNG previews regenerated** and CSS/gallery re-derived; **19/19 regression tests pass**.
- Gold Heritage remains a light macOS shell, with independent original four chart palettes; no premium/paywall/access check.
- Original page 64 Rasi aspects remains a distinct free chart mode, directly linked from All Tools.

## Remaining product implementation (not asserted by visual design)

Android numeric parity has not been demonstrated. Gregorian/Julian/Hybrid date math, Myanmar leap-month/full-moon and observance classifications, source 5D/15D effect, complete ephemeris/house/aspect computations and output fixtures require a native calculation engine and reference comparisons. The separate iZarTarPro sky map is a **sibling-product reference**, not proof of MOTAA APK functions or live 3D rendering. Advanced promotional modules are designed and free to access but their algorithms remain research-gated for correctness. Accessibility, true AppKit resizing, keyboard behavior and actual file persistence require implementation testing.

**Review:** Original user-facing Gold Heritage direction and source chart geometry preserved; new native designs visually inspected at preview size. User visual signoff not yet recorded. No macOS application code, deployment, Git commit or push performed.
