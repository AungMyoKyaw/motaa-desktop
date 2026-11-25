> Historical research/review record. The current single source of truth is design/motaa-desktop.figcraft (58 pages, 882 native navigation links). References below to legacy staging documents or rebuild scripts describe the retired workflow; those artifacts are available in external dated recovery archives.

# MOTAA design extension notes

This is an additive record of the approved established-world expansion. Root [DESIGN.md](../DESIGN.md) and `.impeccable/design.json` are preserved; the sidecar's pre-existing drift is outside this task. The expansion changes scope and theme coverage, not the desktop chart workbook identity.

## Evidence and retained conventions

Compared PRODUCT.md, DESIGN.md, EXPANSION_BRIEF.md, SPEC.md and IMPLEMENTATION_PLAN.md against `source/expanded_common.py`, `expanded_modes.py`, `expanded_tools.py`, `expanded_special.py`, `expanded_support.py`, `expanded_states.py` and representative shared drawing code. Visually inspected the current pink, blue and monochrome exports plus KP and Sarvatobhadra. The generated `source/screen-manifest.json` contains 58 screens and 879 links; `staging/v3/motaa-expanded.figcraft` exists. Those counts describe the manifest, not an independent inspection of every native object or link.

The expansion retains flat paper surfaces, thin rules, line icons, geometric charts, restrained selected fills, Helvetica Neue interface text, Myanmar Sangam MN for linguistic notation, and Baskerville for the wordmark. English interface labels and digits remain the convention; Myanmar syllables and letters remain chart data. Controls retain 38 px height / 6 px corners, fields 44 px / 5 px, and selected rail items 7 px corners. Tables keep shared rows and separators; the advanced analysis panels group controls and related data without replacing each row with a card.

The verified delivery scope remains 1440 × 960, a 208 px rail, 60 px header, 32 px content inset and an optional 98 px exploration footer. The proposed 1180 × 800 minimum and application breakpoints remain unverified. Eleven recovered modes and sixteen advanced workspaces now extend the original screen set. Module-specific matrices, calendars, maps and graphs are additions within the same shell.

## Theme additions

Actual source values below use the source keys: `gold` serves the primary-action role, `bg` the background role. Pink and blue are approved adaptations of source theme families; these values should not be described as exact original-app palette recovery.

| Source key | Gold / incumbent | Pink | Blue | Black and white |
| --- | --- | --- | --- | --- |
| bg | #FAF8F0 | #FFF3F6 | #EFF5FB | #FFFFFF |
| paper | #FFFEF9 | #FFFCFD | #FCFDFF | #FFFFFF |
| rail | #EEEADD | #F3E3E9 | #E0EAF4 | #EEEEEE |
| ink | #282B22 | #35252C | #22313F | #222222 |
| muted | #666958 | #725B66 | #566B80 | #555555 |
| line | #DAD9CB | #DCC6D0 | #C5D5E3 | #CCCCCC |
| gold | #756020 | #8E4564 | #355D87 | #222222 |
| selected | #E8DDAF | #EBC8D8 | #CCDFF0 | #DDDDDD |
| soft | #F3EEDB | #F7E7EE | #E4EEF8 | #F0F0F0 |

`red` #A33F36, `green` #326C4E and `blue` #315F9D remain shared data/error tokens. The black-and-white variant applies to interface surfaces and action colors; chart data colors and window controls remain colored in the inspected export. Root DESIGN.md's gold-only action wording therefore describes the default family. Theme variants substitute the primary and surface roles while retaining semantic data distinctions. The circular renderer also contains fixed cream/gold fills and its source marker palette; the three themed square-chart exports do not establish full theme coverage of every chart.

## Chart geometry and provenance

The shared square chart retains the traditional square, thirds and corner diagonals. `source/circular_chart.py` retains source-informed radii 240/220/145/125/95/78/60, 27 nakshatra sectors, 108 cells, aligned #B7B39F 0.4-unit separators in the numeral and syllable rings, and the cusp/midpoint zigzag. These are recorded rendering conventions; numerical parity remains unverified.

New Rahu and KP wheels are distinct diagrams. KP's equal visual subdivision scaffolding is illustrative and must not define actual sub-lord proportions. The advanced name-letter wheel, Sarvatobhadra grid/vedha overlays, timeline curves, division comparisons and candidate-time tables are proposed geometry and fixtures. Advertised capability establishes inclusion, not formulas, authoritative letter order, classifications, computed strengths, rankings or interpretation passages. SPEC.md and IMPLEMENTATION_PLAN.md retain those research and validation boundaries.

## Review gallery and runtime boundaries

`source/build_gallery.py` adds utility styling for an offline review wrapper: a 248 px screen directory, 21 px heading, 5 px utility-control corners, gold selection, cream surfaces, and #315F9D hover/link overlays and 3 px focus-visible outlines. Its 900 px and 600 px media rules adapt the wrapper, while the embedded screen remains an image. These are gallery conventions, not application rail, responsive-layout or focus tokens established by the FigCraft design.

Gallery screen search, navigation, scaling and hotspot visibility execute. Native hotspots and mirrored gallery links show representative transitions. Arbitrary editing, filtering chart data, saving, importing/exporting, theme persistence, time arithmetic and chart pan/zoom logic remain future application work. Recovery screens specify inspectable states; they do not prove automatic validation or persistence. Independent epochs, dependent period selection, unknown time and workspace/profile settings need stateful implementation and original-output tests.

No contrast ratios, keyboard completeness, screen-reader semantics, Myanmar fallback behavior or smaller-window usability were verified here. Theme contrast and all dynamic-control states require validation in the running application. These notes document the extension without upgrading static fixtures into calculation or accessibility claims.
