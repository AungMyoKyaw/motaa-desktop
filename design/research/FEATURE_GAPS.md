> Historical research/review record. The current single source of truth is design/motaa-desktop.figcraft (58 pages, 882 native navigation links). References below to legacy staging documents or rebuild scripts describe the retired workflow; those artifacts are available in external dated recovery archives.

# MOTAA V31.1 feature gap investigation

Investigation date: 2026-11-11. Design baseline: `design/motaa-desktop.figcraft`, ten pages and 80 navigation hotspots. Sources below are relative to `<research-archive>/motaa-v31.1/decompiled/sources/`.

This report documents static code and screenshot evidence. Later captures in `runtime/` cover individual behaviors and leave full MOTAA V31.1 numerical parity open.

## Display modes

`motaa/myanmar/main.java:743` and `:746` define the menu. `main.java:20744` begins display dispatch; `:15788` sizes each mode separately.

| Mode | Source behavior | Design gap |
| --- | --- | --- |
| 1 Rasi | Traditional square chart | Present as static fixture; mode-specific state combinations absent |
| 2 Tricakka | Multiple chart arrangement | Missing |
| 3 Planetary aspects | Traditional planetary/house relationship tables | Existing design uses illustrative Western symbols; source notation and counting control incomplete |
| 4 Nakshatra traversal | Three round diagrams and dense tables, shown at 2:59:26 | Missing |
| 5 LatLon | Map and detailed planetary data, shown at 2:59:32 | Missing |
| 6 W-Style | Western glyphs and aspect matrices including outer planets | Button only; dedicated content absent |
| 7 Natal + transit | Explicit comparison of two dates/positions | Missing |
| 8 Dasha | Dedicated periods view | Static three-column drawing; drilldown states absent |
| 9 Circular | Source wheel | One static fixture plus detail page; state combinations absent |
| 10 Rahu | Distinct wheel with Janma selection and 27/28 divisions | Missing |
| 11 Rasi aspects | Additional membership-gated aspect mode | Missing |

## Controls and calculation context

`p023L0/C0284s0.java` cases 1 and 4 independently select natal/transit planets and natal/transit Ascendant. All four combinations need explicit state design. Case 0 switches Rasi/Bhava relationship counting. Case 5 controls MUNP; case default controls all-strength output. Cases 2 and 3 control Rahu divisions and Moon-derived/manual Janma selection. `main.java:17683` exposes the Rahu controls only for mode 10.

Time units are Day, Month, Year, Hour, Minute and Second. Previous/next step the selected calendar (`p023L0/ViewOnClickListenerC0282r0.java:663`). The slider applies progress minus 60 to a fresh current-time calendar (`p023L0/C0288u0.java:23`); it does not scrub around the selected date. NOW resets to current time. SET edits all six date/time components. Original date input can normalize invalid dates through lenient Calendar behavior. The design currently shows one Day control and one validation state without these workflows.

Periods are interactive in the source. `main.java:40750` binds ten major and nine secondary rows. `main.java:19967`, `:20250` and `:20512` calculate major periods and child levels. Source handlers select parents and rebuild descendants; the design has arrows without drilldown links.

## Settings, location and profiles

`p023L0/ViewOnClickListenerC0245Y.java:47` implements Tropical/Sidereal, nine ayanamshas, eight house systems and True/Mean Rahu. Ayanamshas: FAGAN_BRADLEY, LAHIRI, DELUCE, RAMAN, USHASHASHI, KRISHNAMURTI, DJWHAL_KHUL, YUKTESHWAR, JN_BHASIN. Houses: Vehlow_Placidus, Placidus, Koch, Porphyrius, Regiomontanus, Campanus, Equal, Vehlow_Equal. The design lists houses but lacks selection states and full ayanamsha options.

The demo shows separate birth and current location editors, hemisphere controls, degree/minute/second coordinate entry, decimal coordinate option, manual offset and location-related checkboxes. Current FigCraft depicts a simplified form without these states.

`motaa/myanmar/UserDBActivity.java:583` creates delete, edit and chart-opening actions. The design library shows a single openable profile and no delete/edit workflow or confirmation state.

Four source themes are gold, pink, blue and black/white (`p023L0/ViewOnClickListenerC0280q0.java:23`). The design has one cream/gold theme.

## Membership and promoted modules

The MOTAA login dialog is local membership activation: copyable device/member code plus password (`p023L0/ViewOnClickListenerC0263i.java:48`). Its result is stored and the activity restarts (`p023L0/DialogInterfaceOnClickListenerC0229H.java:112`). `main.java:40425` validates activation and `:40500` selects the expanded menu. This evidence does not support treating it as a cloud account login.

`motaa/myanmar/paidversion.java:228` contains promotional paragraphs and images for ZarTarPro. Screenshots from 3:00:33 onward show that catalog. Advertised KP, heliocentric and other modules require separate behavior specifications; catalog presence alone does not prove executable support in this APK.

## Actual application boundary

This repository currently contains design-generation Python, SVG artwork, FigCraft files and documentation. `design/source/finalize_design.py` creates navigation hotspots; it does not implement forms or calculations. The recovered Android implementation includes actual numerical routines in `p023L0/AbstractC0297z.java`, including planetary longitude at line 659, retrograde at 834 and house helpers at 194, 406 and 724. Reusing or reconstructing these routines requires numerical comparisons before claiming parity.

The source supports chart zoom and pan (`motaa/myanmar/ZoomableImageView.java`). The design offers a circular detail page but no shared zoom/pan states for other dense charts or tables.

## Established design preferences

Existing project documents record English interface text, English digits, preserved source syllables, preferred supplied nakshatra spellings and cream/gold desktop composition. These remain the baseline for the scope interview.
