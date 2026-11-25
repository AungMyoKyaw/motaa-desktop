> **FigCraft design update, 2026-11-16:** One native master now has 100 screens, 27 variables, six styles, two components and 1,332 prototype actions. See SOURCE_TRACEABILITY.md. The application implementation stages below are still future work, not a completed Tauri or calculation engine.\n\n# MOTAA desktop implementation plan

The user accepted all sixteen recommended decisions, with one scope change: advertised ZarTarPro modules are included. Design the complete set together; implement in stages. The FigCraft file is the visual specification. Reference values are fixtures, not calculation results.

## Stage 1: Local application and calculation foundation

Build the Tauri desktop shell, structured local profile storage, date/time and location models, saved calculation settings, temporary workspace overrides, and reusable chart/table renderers. Support normal names, exact seconds and unknown birth time. Unknown time suppresses Ascendant/houses and flags time-sensitive Moon information; do not silently supply a birth time.

Provide birth/current locations independently, bundled Myanmar town lookup, decimal and DMS inputs, historical timezone resolution with manual offset override. Establish supported date range, timezone data version, coordinate conventions, Gregorian calendar assumptions and numerical tolerances before shipping calculations.

Trace recovered `p023L0/AbstractC0297z` planetary, node, retrograde and house routines. Compare against original MOTAA outputs before claiming compatibility. Retain distinctions such as year-length constants and source planet code 0 until their effects are understood. Decompiled Java is not guaranteed compilable; original Smali resolves ambiguous branches.

## Stage 2: Complete recovered MOTAA workflows

Implement all eleven modes in FEATURE_MATRIX.md, including independent planet/Ascendant epochs, MUNP, sign/house aspect counting, all-strength filtering, Rahu 27/28 and Moon/manual reference. Preserve distinct traditional relationship categories, Western angle tolerances and Rasi-strength arithmetic.

Time stepping supports day/month/year/hour/minute/second. Scrubbing is intentionally relative to the selected date; Now is explicit. Period parent selection refreshes descendants without moving chart time. Separate Go to start/end actions move chart time. Out-of-range selections show recovery rather than stale child periods.

Implement profile create/edit/open/duplicate, undo deletion, unsaved-change handling, search, JSON backup/restore with duplicate review and invalid-record recovery. Design does not imply executable import/export. Implement gold, pink, blue and monochrome themes and chart zoom/pan.

## Stage 3: Advanced numeric workspaces

Implement Ashtakavarga, KP subdivisions, divisional charts, strength timelines, heliocentric coordinates, ephemeris, eclipse contacts/local visibility and transit-event searches. For each, record inputs, mathematical tradition, source references, supported ranges, output units and comparison fixtures. The advertised catalog is capability evidence, not a complete formula specification.

The catalog names Hora, Drekkana, Chaturthamsa, Panchamsa, Shashthamsa, Saptamsa, Navamsa, Dashamsa and Dwadashamsa. Final divisional conventions must be specified explicitly. KP must distinguish star, sub and sub-sub lords for planets and house midpoints. Heliocentric and geocentric coordinates require clear units and centre selection. Ephemeris must resolve Myanmar dates and transition times rather than copying calendar fixtures.

## Stage 4: Traditional advanced systems and sourced interpretation

Implement ten pillars/Ayu Moon, nine Tara/six reference groups, daily Tatkalika, Satapati, Sarvatobhadra and yoga/candidate-time comparisons. Confirm term translations, letter order, reference-point rules, 27/28 treatment, vedha overlays and classification tables with authoritative sources. The proposed grid/wheel geometry is a design fixture and cannot serve as the calculation algorithm.

Interpretation text must come from reviewed traditional sources with English translations and visible calculation context. No predictions or source passages are invented by the design. Electional timing combines rise/set and tithi/nakshatra/yoga/karana transitions with specified strength factors. Candidate-time comparison exposes included factors and their values; scoring requires a documented rule set.

## Acceptance and review

- Verify original-output comparisons for birth dates, time zones, retrogrades, node settings, house systems, all four epoch combinations, nakshatra boundaries and period transitions.
- Verify profile round trips, invalid inputs, unknown time, duplicate imports and undo recovery.
- Inspect every implemented mode at 1440 × 960 and the proposed 1180 × 800 minimum; smaller-window behavior remains unverified by the FigCraft file.
- Verify keyboard focus/order, table semantics, script fallback, theme contrast and zoom/pan in the running application.
- Keep the feature matrix current: design coverage, executable support and verified calculation parity are separate acceptance states.


## Application navigation consolidation

The final app navigation is **five workspaces**, not 100 separate screens: Charts (01), Profiles (31), Calculation Setup (33), Time & Analysis (34) and Advanced Analysis (11). APP_ROUTES.json accounts for all 90 detailed reference frames; ten astronomy teaching frames are design documentation only. Functional state transitions (empty/error/import, included-feature discovery, independent natal/transit modes, time steps, Dasha) stay inside these routes. The native FigCraft master already records five flow starts; do not recreate those states as redundant primary screens during implementation. Original chart drawings and option behavior remain the reference output.

## All-access implementation requirement

Do not implement the original membership/activation/device-ID branch in the new desktop app. Every recovered mode is selectable without entitlement checks. Original mode 11 remains a distinct source-calculated mode, while advertised-only tool calculations require independently verified implementations. No fake activation service or premium subscription logic should be created.

## Source-parity runtime gates (2026-11-17)

- Implement exact Myanmar leap-month/watat/full-moon corrections, lunar phase and weekday classifications from main.A0, AbstractC0029c and C0025a; compare original APK output fixtures.
- Keep Gregorian/Julian/Hybrid (JDN 2361222 reform) and source-stored 5D_Aspect/15D_Aspect options, without inventing behavior absent direct source evidence.
- Keep five application routes; source error/access pages are conditional runtime states, now prototype-reachable for review.
- The iZarTarPro 2D/3D sky reference is a separately researched future roadmap module, not a claimed MOTAA V31.1 recovered calculation.
- Original and proposed advanced feature entries remain free; working engine correctness and numerical parity are distinct release gates.
