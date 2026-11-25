> Historical research/review record. The current single source of truth is design/motaa-desktop.figcraft (58 pages, 882 native navigation links). References below to legacy staging documents or rebuild scripts describe the retired workflow; those artifacts are available in external dated recovery archives.

# First design review

## English numerals and preferred nakshatra spellings

Converted every visible numeric label to English digits, preserving source values such as Saturn code 0. Increased quarter-code and index text, strengthened numeral weights, used Arial Narrow for dense ring names, and improved planet-marker text contrast. The detail view now includes all 27 full names at 16 px, matching the user's supplied contents image, including Ashvini and Jyeshta.

An independent reviewer inspected all ten final previews and both supplied reference images and returned **ship**, finding no material overlaps and confirming name spelling and numeral consistency. Mechanical inspection found no Burmese numeral characters in any generated screen and confirmed all 27 preferred full names in the detailed chart. Fit-view text remains small; the 760 px detail view is the readable alternative. The 108 phonetic syllables remain linguistic source data.

## Source-informed circular correction

The user's original circular-chart screenshot superseded the earlier circular-design approval. Investigation traced mode 9 to `main.m0`, read its complete Java renderer, checked the house cusp/midpoint helpers, and verified the duplicated Java drawing loop against authoritative Smali.

The generic circular wheel was replaced with the source's seven ring radii, 27 nakshatra sections, 108 syllable subdivisions, black house cusp dots, colored numbered house midpoints and their 24 connecting segments. Planet coordinates and colors now use the source orientation, quarter placement and color-classification table for an explicitly recorded fixture. Added a linked 760 px detail view with English explanations.

A fresh independent reviewer compared the user's reference with both revised circular previews and returned **ship static design**, with no blocking clipping or overlap. Small glyph readability in the normal view remains a limitation; the detail view provides more room. Structural checks verified all layer counts, 12 cusps, 12 midpoints, 24 connectors, 10 body markers and the source longitude orientation. Numerical parity remains unverified.

The current artifact has ten pages, 80 prototype links, 17 variables and no missing fonts. See [the evidence report](research/CIRCULAR_CHART.md) for source locations, fixture provenance and notation details.

## English interface revision

All nine screens were rebuilt with English navigation, headings, form labels, actions, settings, validation and Romanized place names. Burmese characters remain only as traditional planetary numerals. The previous Burmese design is preserved at `staging/motaa-desktop-burmese-original.figcraft`.

Visual inspection caught and corrected the Ascendant label crossing a diagonal, aspect chords crossing the circular chart title, and inconsistent chart-tab order. An independent reviewer inspected all nine English exports and returned **ship**, finding no material clipping or overlap at 1440 × 960. A subsequent contrast check found selected navigation helper text at 4.13:1; the selected helper now uses primary ink for stronger contrast. Main muted text measures 5.30:1, primary button text 6.03:1 and validation text 5.95:1 against their backgrounds.

Source inspection confirms no Burmese interface words remain. The final design retains nine pages, 17 variables, 72 navigation links and no missing fonts. Prototype arithmetic and runtime behavior remain unverified.

## Original review

Reviewed nine final 1440 × 960 FigCraft screen exports. An independent reviewer requested explicit reference-data labeling in both birth form previews. This was added to the source, the design was rebuilt, and the reviewer scored the fix resolved with disposition **ship the first design**.

Verified mechanically: all recorded CLI commands succeeded; nine page frames; 17 native variables; 72 prototype navigation links; no missing fonts; all nine previews have the expected dimensions. This establishes design structure and export success, not working application behavior.

The review found no material clipping or overlap at the supplied canvas size. Keyboard interaction, runtime forms, responsive layouts and arithmetic have not been verified.

## Scope carried forward

The first design covers the central chart/profile/aspect/period/settings workspace. The live demo also includes multiple-chart arrangements, nakshatra tables, a LatLon map/planet table, natal/transit comparison and an original MOTAA login dialog. These need additional screen designs and behavior specifications before claiming complete V31.1 coverage. Login must be understood before choosing whether the desktop version needs accounts.

Screenshots from 3.00.33 onward show the ZarTarPro promotional catalog. Its advertised modules are distinct from the live MOTAA demo and are outside this first design.

All source screenshots and research files were preserved. No application source, user database or remote service was modified.
