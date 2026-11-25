# MOTAA desktop · Native FigCraft design

**Single authoritative design:** [motaa-desktop.figcraft](motaa-desktop.figcraft).

The current master contains **111 pages**: 101 application/feature-state references and **10 astronomy lessons**. The original 58 screens, 32 source states, and pages 101–104 are preserved in order, followed by the six UX/parity states and the mode 11 all-strengths scenario. It has **134 native variables** across four collections (chart themes, Gold Heritage semantics, birth-details privacy, and MOTAA primitives), **31 styles**, **2,456 native navigation actions**, and **71 native variable selections**.

This is an editable design prototype, not a running astrology engine. Every original feature entry is free to access in this proposed desktop edition; algorithms require implementation and verification.

## Full-screen audit · 21 November 2026

All 110 incoming screens were visually inspected and programmatically checked. The revised Rasi aspects view and its new all-strengths scenario are pages 64 and 111. Every preview was regenerated from the master. See [the full-screen review](FULL_SCREEN_REVIEW.md).

## Desktop workflow review · 21 November 2026

Settings now opens from the sidebar and includes three text sizes with a live FigCraft preview and a size-aware planetary coordinates table. Time & Analysis links directly to the reference date and location editors. See [the source coverage and workflow review](DESKTOP_WORKFLOW_REVIEW.md).

## Gold Heritage v3 · implemented (17 November 2026)

The **single native FigCraft master** implements Modern Heritage Gold using 26 editable UI tokens (antique gold, ivory, bronze, controlled state colors) and keeps the 27 existing chart/session variables unchanged. This retains 100 native pages, the five app entry flows, all 90 UI/state references, 10 unchanged algorithm lessons, 19 reusable component families and all 2,195 prototype actions. Review evidence: [GOLD_HERITAGE_V3_REVIEW.md](GOLD_HERITAGE_V3_REVIEW.md). Source and review contracts: [planning/gold-heritage-v3/](planning/gold-heritage-v3/). Design-only; no application code or deployment performed.

### Historical v2 reference

## Native Precision v2 · Lucide execution (17 November 2026)

The approved Native Precision v2 milestone was applied to the one master; its UI palette was subsequently superseded by Gold Heritage v3. At that earlier milestone, all 90 product/state views used System Blue/Slate light appearance; original mathematical chart colors, four independent chart themes and ten teaching pages remain. Real editable Lucide SVG path components, live color-variable bindings, 19 reusable component families and 450 icon instances are present. All 100 pages were rendered; the current design regression suite passes 17/17 tests.

See [NATIVE_PRECISION_V2_REVIEW.md](NATIVE_PRECISION_V2_REVIEW.md) for verification, screenshot evidence, recovery and limitations. Earlier [MACOS_DESIGN_REVIEW.md](MACOS_DESIGN_REVIEW.md) describes the historical pre-v2 state.

## macOS application direction: 17 November 2026

The one editable master now uses cohesive macOS-style chrome across all 90 application-facing pages; the remaining ten calculation lessons form a separate reference appendix. The five-workspace navigation, compact system toolbar, macOS blue primary controls, native-style input rows, and contextual panels replace the reconstruction/workbook presentation. All 100 pages were rendered and audited. See [MACOS_DESIGN_REVIEW.md](MACOS_DESIGN_REVIEW.md) and [QA_REVIEW.md](QA_REVIEW.md).

Open design/motaa-desktop.figcraft in FigCraft; view generated previews in design/index.html. This remains a design artifact, not an executable native application.

## Consolidated application direction: 5 real workspaces

The complete Android-derived design is organized into **five native FigCraft application flows**. The current main screens are the expected output; no generic dashboard replaces the original chart or form layouts.

| Product workspace | Actual native FigCraft starting screen | Detailed state references |
| --- | --- | ---: |
| Charts and modes | 01 Natal chart | 22 |
| Profiles and locations | 31 Profile library | 19 |
| Calculation setup | 33 Calculation options | 15 |
| Time and analysis | 34 Time navigator | 15 |
| Advanced analysis | 11 All tools | 23 |

**94 source-informed feature/state pages** are mapped to those five workspaces; the **10 algorithm lessons** remain teaching material. Pickers, modal dialogs, validation, error/empty screens, four natal/transit combinations, mode variants and included-feature states are application **states**, not distinct top-level navigation destinations.

See [APP_CONSOLIDATION.md](APP_CONSOLIDATION.md) for the source evidence, complete page-by-page assignment, feature retention rules, unverified methods and acceptance gates. [APP_ROUTES.json](APP_ROUTES.json) provides machine-readable coverage enforced by regression tests.

The [offline walkthrough](index.html) now opens the actual natal-chart workspace and groups reference states under the five application destinations. All original 100 native FigCraft pages are retained and visually unchanged.



## Learn the circular-chart algorithm in FigCraft

Open the native master on page **91 Lesson - Anatomy of the wheel**, or navigate from **02 Circular chart** using the on-screen **How this wheel works** link.

1. Seven source radii, 12 signs, 27 Nakshatras and 108 Padas
2. Longitude 195.247 degrees to grid 58, angle 194.933 degrees, and pixel coordinates
3. Nakshatra and Pada subdivisions
4. Planet markers and overlap collision radii 210, 198, 186
5. House cusps, circular midpoints and 12 colored connections
6. Final assembly showing fixed source tables versus computed data

Each diagram uses native editable FigCraft vector nodes, not imported artwork. Values are source-informed **illustrative examples**, not live ephemeris results or verified MOTAA APK outputs. Myanmar calendar calculation is intentionally excluded from these six lessons.

## Swiss Ephemeris: where 195.247 degrees comes from

In the canonical FigCraft master, open **97 Lesson - Swiss Ephemeris to Moon**, or follow the new link from **92 Lesson - Degrees to positions**. The seventh native educational screen explains the complete calculation:

- Convert local civil time to UTC and the full-precision Julian Day UT (2451545.0).
- Call pyswisseph 2.10.03, sidereal Lahiri, with the Moshier ephemeris and MOON body.
- Independently verified tropical Moon 219.03172767 degrees, Lahiri adjustment 23.78476711 degrees and final sidereal Moon 195.24696056 degrees.
- The Moon longitude is returned by the astronomical library; the MOTAA circular renderer subsequently converts it to the 108-cell chart grid.

The personal identifying birth record remains only in the private Git-ignored example. The verified reference is not a captured output from the original MOTAA APK and there is no live calculation engine in FigCraft.

## Location and 12 house cusps: visual lessons 98–100

Start at **97 Lesson - Swiss Ephemeris to Moon** and choose **Why location changes houses**. The canonical master contains three additional native, editable FigCraft screens:

- **98 Location affects houses:** identical Julian Day and Moon, but changing longitude from 96.17° E to 100.55° E at 20.00° N moves the Equal-house sidereal Ascendant from 154.614329° to 159.320911°.
- **99 Swiss house cusps:** all 12 computed cusp values with Ascendant and Midheaven. Exact method: after setting Lahiri, call the Swiss Ephemeris sidereal Equal house API using Julian Day UT 2451545.0, 16.84° N, 96.17° E.
- **100 Cusps to connectors:** for Equal houses, cusp 1 = 154.614329°, cusp 2 = 184.614329° and their midpoint = 169.614329°. The native vector drawing shows 12 black cusp dots, 12 colored midpoints, and their connections on the 108-cell wheel.

The sample location and second hypothetical longitude are teaching coordinates, not live location tracking. All values were independently reproduced with Swiss Ephemeris 2.10.03 / pyswisseph in an external reference fixture. Original MOTAA V31.1 numerical parity is unverified; the FigCraft master remains a **design prototype**, not a running astronomy engine. The original birth name/date were deliberately not placed into these screens.

## Source-led improvements

- Free access: the original 11th mode (Rasi aspects) opens directly from All tools for every user. Pages 59–63 now document the included-features edition, not credential or activation flows.
- Source settings: Zodiac, all nine ayanamshas, all eight house systems, True/Mean Rahu; selectable prototype variables.
- Birth/current locations, invalid input, full date/time editor and source-faithful slider centered on current system time.
- Four natal/transit planet and Ascendant combinations, Rahu 27/28 reference, Dasha details, source aspect counting, MUNP/strength and focused chart inspection.
- Responsive design **references** at 1280 × 960 and 1024 × 900, four native color-variable modes and shared native components/styles.
- Source evidence labels for the 16 advertised-only ZarTarPro workspaces; illustrative sample names and birth timestamps are anonymized.

See [SOURCE_TRACEABILITY.md](SOURCE_TRACEABILITY.md) for screenshots, code evidence, uncertainty and remaining QA.

## Visual QA (2026-11-16)

Reviewed all 100 native pages. Corrected 33 obscured birthplace labels, three covered theme captions, and nine off-canvas separator endpoints. The 39 affected screens were rendered and compared; the layout now has zero text overlaps and out-of-frame elements. See [QA_REVIEW.md](QA_REVIEW.md) for evidence, recovery, and limitations.

## Export and verify

Make native design edits only in the canonical FigCraft master. All temporary candidate FigCraft files must be outside this repository. The old Python/SVG rebuild pipeline has been retired.

The Western source-reference screens use 24 native vector outlines exported from the recovered Android Astro font. The master renders these symbols without a font installation. The original asset remains at design/fonts/Astro.ttf for provenance. The complete recovered 534-town directory is design/source/locations.v31.1.json.

Use these commands from the repository root:

    figcraft-cli info design/motaa-desktop.figcraft --json
    python3 design/source/figcraft_master.py
    python3 design/source/export_previews.py --jobs 4
    python3 design/source/export_tokens.py
    python3 design/source/build_gallery.py
    python3 -m unittest discover -s design/tests -v

- [previews/](previews/): generated PNG exports for all 111 pages, preserving individual frame dimensions.
- [tokens.css](tokens.css): generated CSS values from native Light, Pink, Blue and Monochrome variable modes.
- [index.html](index.html) and [source/screen-manifest.json](source/screen-manifest.json): native navigation-derived static walkthrough. The HTML supports navigation, **not interactive variable changes**.
- Historical SVG artwork in source/ is not an input for rebuilding the master.

## Recovery and limitations

The original 58-page design and subsequent candidate/command logs are backed up outside this repo at ~/.local/share/motaa-desktop-backups/figcraft-redesign-20261116-205942/. Exactly one native FigCraft document is allowed in the project.

The original chart geometry is preserved. Advanced results, calculation output and newly drawn supporting diagrams are illustrative unless independently verified. The native prototype does not implement authentication, data persistence, numerical parity, keyboard runtime or actual responsive app resizing. Its smaller-screen pages are reference layouts.

## No premium or membership tier

Every original chart mode and designed tool entry is included for every user. The original Android main.java has a shorter ordinary mode list and a longer expanded list; the desktop directly exposes the additional Rasi aspects mode 11 from All tools. FigCraft pages 59–63 now represent open access, and page 64 retains mode 11 itself. No activation, password, account or device code is required. This is a UI/release-policy decision and does not prove advanced computational outputs.

## Source parity extension: 2026-11-17

The single editable native master now contains 101 Myanmar lunar calendar, 102 Original calculation controls, 103 Traditional observances and 104 Sky map - 2D. The first three cover recovered MOTAA Android behaviors. The fourth is explicitly a separate iZarTarPro reference. The five workspaces, original source chart geometry, ten astronomy lessons and free original/advanced navigation remain. Historical 100-page Gold Heritage review statistics above describe the preserved milestone baseline.

See [SOURCE_PARITY_20261117_REVIEW.md](SOURCE_PARITY_20261117_REVIEW.md).

## UI component audit: 17 November 2026

A screenshot of page 61 exposed top-aligned button labels and repetitive content. The native FigCraft master now corrects 138 option labels on 32 reviewed screens, preserves 12 two-line source fields and all original chart geometry, and consolidates the five access-information reference layouts. Every affected page was rendered. The expanded regression suite passes 20/20 tests. Review UI_COMPONENT_AUDIT_20261117.md and reviews/ui-audit-before-after-61.png.

## UX parity and design-system extension: 21 November 2026

Pages 105–110 add an anonymous first launch, independent Natal/Transit assignment, free 3D sky, a native 1280×800 compact chart, recoverable advanced-calculation failure, and visible component states. Pages 104 and 107 are linked 2D/3D sky workspaces; their illustrative geometry is not live ephemeris output. Every advanced entry is enabled free, with no research gate or fake result.

The design system now exports raw MOTAA primitives, preserved Gold Heritage semantic aliases, and component tokens alongside the four independent chart themes. All 110 previews, the manifest, tokens, and gallery are generated from the canonical master. See [UX_PARITY_20261121_REVIEW.md](UX_PARITY_20261121_REVIEW.md).
