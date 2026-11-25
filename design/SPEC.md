# Expanded desktop design specification

Status (2026-11-16): 100 native FigCraft pages—98 at 1440 × 960, one at 1280 × 960 and one at 1024 × 900. The original 58 screens, 32 source-state pages and ten astronomy/circular calculation lessons are present. No Tauri runtime or calculation engine is implemented.

## Layout and notation

Follow root DESIGN.md: 208 px rail, 60 px native header, 32 px inset, flat paper surfaces, quiet rules and traditional geometric charts. English interface/digits, source syllables and supplied preferred nakshatra spellings remain mandatory. The All tools directory contains eleven recovered modes and sixteen advanced workspaces. Each uses a content-specific chart, matrix, graph, calendar, map or table layout. Analysis retains calculation context and time navigation. Four theme families are represented.

Smaller 1280 × 960 and 1024 × 900 reference layouts exist; true responsive runtime remains unverified. An illustrative KP wheel or traditional grid is not a mathematical specification.

## Recovered modes

| Mode | Required content | Pages |
| --- | --- | --- |
| Rasi | Square chart, planet positions, independent epochs and MUNP | 01,20 |
| Tricakka | Rasi, Bhava and Navamsa together | 12 |
| Traditional aspects | Source relationship categories; sign/house counting | 03,13 |
| Nakshatra traversal | Three charts, sign/nakshatra/quarter tables, Dasha remainder | 14 |
| LatLon | Source map, projected bodies, house axis, longitude/declination/latitude/motion/AU | 15 |
| W-Style | Western matrix, outer planets, house relationships and source orbs | 16 |
| Natal + transit | Birth/current comparison | 17 |
| Dasha | Three dependent levels, active tracking and out-of-range recovery | 04,39,40 |
| Circular | Source-informed seven-ring wheel, detail and zoom | 02,10,54 |
| Rahu | Distinct wheel, 27/28 grouping and Moon/manual reference | 18,50 |
| Rasi aspects (original mode 11, included for all) | Direct access, planet/Ascendant and house matrices, aspected-only and all-strengths states | 59–64, 111 |\n| Rasi strengths (separate research display) | All/filtered strength views | 19,51,82 |

Traditional categories are aspect, house exchange, mutual aspect, lord aspects, aspected lord and conjunction; source highlights indicate within 5 degrees. Western tolerances: 0/180/120/90 degrees ±8, 60 ±6, 150 ±3, 30/135/72/144 ±2. These systems remain separate.

MUNP changes calculation/rendering branches; its complete explanation remains unresolved. Do not invent its expansion. In Rahu mode, 27/28 changes grouping lookup while the original manual reference selector retains its 27-name list (main.java:806 and :16112).

## Approved interactions

- Planet positions and Ascendant/houses choose Birth/Transit independently. Page20 depicts one mixed state and lists all four combinations. The runtime must implement all four.
- Arrows retain six time units. The source-faithful slider offsets from a fresh current system time by progress minus 60 (src/motaa/L0/u0.java). Pages 34 and 73 demonstrate this without executing date arithmetic.
- Period selection updates children without moving chart time. Explicit Go to start/end actions move time. Outside the calculated range, clear active children and offer recovery. Page40 illustrates the layout; runtime must avoid stale values.
- Calculation settings are temporary workspace overrides until Save to profile. Page33 includes both zodiacs, nine ayanamshas, eight houses and both nodes.
- Birth/current locations retain separate names, coordinates and offsets. Include Myanmar town lookup, decimal/DMS entry, hemisphere controls, historical zones and manual override. Page32 depicts decimal defaults and DMS equivalent; advanced editing logic remains runtime work.
- Validate impossible dates, retain normal names and seconds. Unknown time suppresses Ascendant/houses and warns about time-sensitive Moon results.
- Library includes create/open/edit/duplicate, search, undo deletion and unsaved-change review. Backup import reviews duplicates and excludes invalid records. Original database migration awaits sample data.
- Original Android mode 11 used member restriction. In this desktop edition it is included for all: pages 59–63 now show free-access discovery, no-account, offline availability, and mode-ready states; page 64 preserves the original mode 11 chart. No password, device ID, account or activation is required.

## Advanced modules

Pages21–30 and41–46 cover strength timelines, Ashtakavarga, KP, heliocentric coordinates, divisional charts, interpretations, electional timing, ephemeris, eclipses, transit events, ten pillars/Ayu Moon, Tara/reference groups, daily Tatkalika, Satapati, Sarvatobhadra and yoga/candidate-time analysis. The user explicitly included these in scope.

These are proposed substantive layouts with illustrative tables/diagrams. Advertisements establish scope, not calculation rules. No prediction, interpretation passage or candidate ranking is invented. IMPLEMENTATION_PLAN.md records further research and acceptance requirements.

## Prototype boundary

Transparent hotspots demonstrate representative screen/state transitions. Gallery hotspots mirror them. Controls without representative destinations remain visual specifications. Arbitrary editing, filtering chart data, downloads, imports, theme persistence, time arithmetic and chart panning do not execute. Gallery screen search, navigation, preview scaling and link overlays do execute.

Empty-library, no-results, invalid-date and invalid-backup screens are inspection states selectable in the gallery; not all conditions automatically trigger in FigCraft. Figures do not establish calculation parity or source fidelity for advertised modules. Repeated division/comparison charts intentionally use the reference fixture.


## Five-route desktop application contract (2026-11-17)

The 100 FigCraft pages represent five real application workspaces, 90 detailed source/state reference pages, and 10 educational pages. Native FigCraft starting flows for Charts, Profiles, Calculation Setup, Time & Analysis and Extensions & Access point to original pages 01, 31, 33, 34 and 11 respectively. Related pickers, transient dialogs, errors, theme variants and chart submodes must be represented as states of their owning route, not independently added to the product navigation. The UI remains pixel-identical to the approved source-informed layouts. See APP_CONSOLIDATION.md for the complete ownership mapping and the release/parity conditions.

## Universal access in the desktop edition — 2026-11-17

Every original chart mode, including the additional mode 11 Rasi aspects, is free and accessible to all users. The original source's two mode lists describe historical access differences. The desktop will have no account, license, password, activation or device check. All tools links directly to mode 11. Designed advanced feature entries are also included; advertised-only computations remain unverified, and UI access must not be confused with a working engine.

## macOS application visual contract

All 90 application-facing frames use a consistent macOS-style visual language. Birth and Current Location (69 and 70) show labeled controls, their original example coordinates and time-zone values, and useful contextual guidance rather than engineering notes. Narrow layouts at 1024 and 1280 pixels use the same window treatment. All chart modes and settings remain represented, with ten educational pages in a separate reference section. Live native controls and calculation parity remain implementation requirements, not claims of this visual design.


## Current Gold Heritage v3 execution result

The single editable FigCraft master now uses the approved ivory/gold/bronze UI palette, with 75 native variables in three collections and retained 19 component families, 35 source Lucide icon components and 450 links. Native Precision v2 remains the unchanged layout baseline; 10 teaching pages and all four source chart modes are preserved. See GOLD_HERITAGE_V3_REVIEW.md and SOURCE_PARITY_20261120_REVIEW.md.

## Historical Native Precision v2 execution result

The approved light-first System Blue/Slate design system has now been applied in the canonical master (100 pages). Real Lucide SVG-derived editable components are used for the five workspaces; action blue is #1D61D6, while original chart classification colors and four chart themes remain separate. The master includes 19 reusable native component families with four variants each, 450 linked Lucide instances, and two semantic variable collections. Design QA and remaining runtime/approval boundaries: NATIVE_PRECISION_V2_REVIEW.md. This v2 record is superseded for desktop chrome by the verified Gold Heritage v3 master; original chart semantics remain authoritative.

## Source-parity addendum (2026-11-17)

The current editable master has 104 pages: 94 app/state references and 10 original educational pages. Source fidelity additions 101–103 cover Myanmar lunar date, traditional day classifications, source calendar modes and stored aspect preference strings. The separate iZarTarPro sky reference is page 104. No invented positions, lunar values or predictive statements are supplied. All original modes and research entries are free to access in the desktop design; numerical and live AppKit parity remain unimplemented.
