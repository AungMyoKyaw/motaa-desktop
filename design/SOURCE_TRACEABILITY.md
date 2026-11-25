# MOTAA V31.1 · Source evidence and FigCraft correspondence

**Updated:** 2026-11-16 · **Evidence:** <source-research-repo>

The original source project contains 1,104 decompiled Java files and an APK. The design repository root has 29 original application screenshots. Decompiled source is evidence of behavior; screenshots show appearance and specific states. Neither alone proves numerical parity. Original Java files have not all been audited line by line.

## Screenshot map

| Original screenshot | Visible feature | Corresponding native FigCraft screen(s) |
| --- | --- | --- |
| S01 · 2:57:26 | Birth/profile editor | 06, 72 |
| S02 · 2:57:42 | Birthplace and coordinates | 32, 69 |
| S03 · 2:58:07 | Current location and offset | 32, 70, 71 |
| S04 · 2:58:17 | Tropical/Sidereal | 65 |
| S05 · 2:58:22 | Nine ayanamshas | 66 |
| S06 · 2:58:33 | Eight house systems | 67 |
| S07 · 2:58:36 | True/Mean Rahu | 68 |
| S08 · 2:58:41 | Original Android local member-code/password dialogue (archival evidence, not this edition) | historical 59–64; redesigned |
| S09 · 2:58:50 | Rasi square | 01 |
| S10 · 2:59:02 | Tricakka charts | 12 |
| S11 · 2:59:19 | Traditional aspects | 03, 81 |
| S12 · 2:59:26 | Nakshatra diagrams | 14, 83 |
| S13 · 2:59:32 | LatLon map and planetary data | 15, 69–70 |
| S14 · 2:59:38 | W-Style chart | 16, 84 |
| S15 · 2:59:42 | Natal/transit comparison | 17, 74–77 |
| S16 · 2:59:47 | Dasha levels | 04, 39, 40, 80 |
| S17 · 2:59:53 | Circular wheel | 02, 10, 54, 89 |
| S18 · 2:59:59 | Circular alternate state | 02, 10, 54 |
| S19 · 3:00:02 | Rahu wheel | 18, 78 |
| S20 · 3:00:07 | Rahu/manual selection | 50, 79 |
| S21–S29 · 3:00:33–3:01:13 | ZarTarPro promotional catalog | 21–30, 41–46, 85 |

This table maps coverage, **not pixel-perfect visual equivalence**. S21–S29 advertise features but do not prove those algorithms execute in MOTAA.

## Current extracted source paths

| Actual code location | Verified relevance | Design |
| --- | --- | --- |
| src/motaa/motaa/myanmar/main.java, around lines 405–408 | Original and member-expanded chart arrays; separate mode 11 | 59, 64 |
| src/motaa/L0/u0.java, onProgressChanged | Current Calendar, progress minus 60, Day/Month/Year/Hour/Minute/Second | 34, 73 |
| src/motaa/motaa/myanmar/UserDBActivity.java, around lines 570–590 | Delete/edit/open-chart actions | 31, 47, 48 |
| src/motaa/motaa/myanmar/ZoomableImageView.java | Zoomable chart viewer class | 10, 54, 89 |
| src/motaa/motaa/myanmar/main.java, around 15484 | Gold/pink/blue/black-white themes | 38, 56–58, 86 |
| src/motaa/motaa/myanmar/paidversion.java, around 232 | Advertised advanced functionality | 21–30, 41–46, 85 |
| src/motaa/L0/AbstractC0055z.java | Astronomical calculation facade | Context only; engine not reproduced |
| src/motaa/L0/AbstractC0029c.java | Myanmar calendar computation | Context only; engine not reproduced |

Older documents refer to a vanished p023L0 decompilation tree. Those aliases and previous line references are historical and must not be cited as current paths.

## Approved decisions and boundary

Q001–Q084 = A, no overrides, approved on 2026-11-16. Behavioral correctness follows source evidence; the native FigCraft document is authoritative for desktop design. Source-faithful time scrubbing is relative to fresh **current system time** rather than previously selected time. The original app member gate is historical source evidence; the 2026-11-17 desktop edition replaces pages 59–63 with free-access states. The 11th original mode remains distinct from Rasi strengths and is included for everyone.

English UI/digits and traditional Myanmar notation remain, with personal sample names and dates anonymized. Exactly one native FigCraft master. No desktop application code, Git commit or deployment is authorized as part of this design execution.

## Remaining work outside the FigCraft prototype

1. Run original APK numerical fixtures across epoch, ayanamsha, house, node, Dasha, MUNP and boundary conditions.
2. Compare native-resolution mobile screenshot crops against the final screens before claiming pixel-level parity.
3. Verify interactive variable selection visually inside an actual user-driven FigCraft prototype session. Native action records and static renders alone do not prove every interactive transition.
4. Verify real keyboard navigation, screen-reader behavior, resizing and persisted data in a future desktop application.
5. Check licensing and asset provenance before shipping source-derived astronomy code or imagery.

## Circular wheel lessons 91–96

The original drawing code is in motaa-app/src/motaa/motaa/myanmar/main.java, method m0(), with G0(), H0(), x0() and AbstractC0055z.g()/m() for its input calculations.

The recovered drawing center is (240,240); ring radii are 240,220,145,125,95,78,60. Static divisions are 12 signs of 30 degrees, 27 Nakshatras of 13 1/3 degrees, and 108 Padas of 3 1/3 degrees.

Lesson 92 shows a 195.24696056 degree **independent reference** Moon longitude (not an original APK output), quantized to bin 58, then mapped using angle = bin × (360/108) + 1.6 degrees = 194.933333 degrees, with the original coordinate helper's 75-degree orientation.

Lessons 95–96 use an illustrative Equal-house Ascendant 154.61432908 degrees to demonstrate 12 cusp-to-midpoint connectors. Planet placement at radius 210 and collision spacing 12 are from the original renderer. No active ephemeris engine or Myanmar-calendar calculation exists in these educational screens.

## Astronomy reference derivation (FigCraft page 97)

The 195.247-degree value in page 92 is an independent astronomical reference, not a hardcoded original APK output. The new page 97 shows Julian Day UT 2451545.0 and the exact algorithm call: set SIDM_LAHIRI, then call calc_ut(JD, MOON, FLG_MOSEPH | FLG_SIDEREAL). Verified with pyswisseph 2.10.03: tropical 219.03172767223663 degrees; matching sidereal correction from get_ayanamsa_ex_ut(JD, FLG_MOSEPH) is 23.784767109555524 degrees; final sidereal longitude 195.24696056268104 degrees. The tracked page contains neither the identifying name nor the birth date/time. Numerical agreement with original MOTAA V31.1 remains unverified.

## Geographic house calculations (FigCraft pages 98–100)

For a fixed UT Julian Day of 2451545.0, an **independent** Swiss Ephemeris 2.10.03 calculation with the Lahiri sidereal setting and Equal houses returned:

- Latitude 20.00° N, longitude 96.17° E: Ascendant / Equal cusp 1 = **154.61432907775307°**, Midheaven = **64.65485545985467°**, Equal cusp 2 = **184.61432907775307°**.
- Circular midpoint between cusp 1 and cusp 2 = **169.61432907775307°**.
- For the hypothetical longitude 100.55° E with the same latitude and Julian Day, the Ascendant becomes **159.3209112105056°**. The geocentric sidereal Moon longitude at that same instant remains **195.246960562681°**.

The original source (motaa-app/src/motaa/L0/AbstractC0055z.java method m()) computes house cusps and circular cusp midpoints; the original Mode 9 source (motaa-app/src/motaa/motaa/myanmar/main.java method m0()) quantizes their position to the 108-cell grid and draws black cusp markers at radius 150 and colored house midpoints at radius 160. The source call differs for custom Vehlow modes; these figures are explicitly an Equal-house demonstration, not proof of original APK numerical parity.

A project-external generated reference file is at ~/.local/share/motaa-desktop-backups/location-houses-lesson/reference.json. No birth name/date, personal live geolocation, executable astronomy engine, or external map service is included in the tracked master.


## Five consolidated product workspaces (2026-11-17)

Native FigCraft flow starts: 01 Natal chart = Charts/modes; 31 Profile library = Profiles/locations; 33 Calculation options = Settings/house/zodiac/ayanamsha/node; 34 Time navigator = Time/Dasha/epochs; 11 All tools = Advanced analysis, all modes included. The 90 original feature and state frames have a unique owning route in APP_ROUTES.json, ten algorithm lessons remain separate. Reference source for mode dispatch is motaa-app/src/motaa/motaa/myanmar/main.java S0(), with setting and house behavior from main.java and L0/AbstractC0055z.java; time handling in L0/u0.java; record handling in UserDBActivity.java; member advertising/gating in paidversion.java and main.java. Advertised ZarTarPro modules do not establish original Android numerical parity. See APP_CONSOLIDATION.md for the exact per-state evidence classification.

## Original Android gate versus this edition — 2026-11-17

Original source evidence: main.java defines f3306o, a shorter mode list excluding Rasi aspects, and f3307p, the expanded list with original mode 11. The T0 == 11 branch displays that additional mode. Android ID setup and paidversion.java are original access/marketing evidence. For this new desktop edition, all modes are included for every user. The All tools FigCraft page now links directly to mode 11; the previously gated state pages 59–63 were rewritten as included-feature product states. Source files and screenshots in motaa-app were not modified. This changes access policy, not source calculation algorithms; advertised-only mathematical parity is not established.

## November 2026 source-parity discoveries

| Actual evidence | New design |
| --- | --- |
| src/motaa/motaa/myanmar/main.java main.A0 and src/motaa/L0/AbstractC0029c.java lunar-era algorithm | 101 |
| src/motaa/L0/C0025a.java and main.A0 traditional calendar flags | 103 |
| REVERSE_ENGINEERING.md Sections 3, 7; main.java stored options | 102 |
| IZARTARPRO_REVERSE_ENGINEERING.md Sections 5.3–5.5, sibling screenshot/version-history evidence | 104 and 107, free enabled sibling-inspired 2D/3D workflows |

The original Android membership check is historical evidence only: all eleven original modes including Rasi aspects remain open. New astronomical, astrological and calendar figures are explicitly illustrative until verified against original runtime fixtures.

## UX, design-system, and free-access pass — 21 November 2026

This pass modifies the native FigCraft master only through FigCraft CLI staging. It preserves the original 104-page sequence, chart references, five application flows, Gold Heritage light UI, and ten lessons. New pages 105–110 add first-run, independent epoch assignment, 3D sky, compact desktop, recoverable failure, and visible component-state coverage.

| Gate | Detail |
|---|---|
| Free means enabled | Every original and advertised/sibling-inspired entry remains available without account, activation, payment, membership, or research gate. |
| Honesty remains mandatory | Unverified formulas and sky positions are labeled illustrative. The design does not invent a successful calculation or claim runtime/numerical parity. |
| Design-system architecture | `MOTAA / Primitives` contains raw color, spacing, sizing, radius, focus, and motion values. Existing Gold Heritage variables retain their IDs and resolve through semantic aliases; component tokens consume those semantics. Chart-only palettes remain independent. |
| Navigation evidence | 104 ↔ 107, 01 ↔ 108, 31 → 106, 08 → 105, 28 → 109, and 86 ↔ 110 are native links. First-run create/import actions target the real Birth profile and Backup restore frames. |
| Responsive evidence | Page 108 is a genuine 1280×800 native frame, not a resized screenshot. Its square chart and planet table remain editable native nodes. The duplicated square chart is scaled uniformly to 80% and starts below the mode controls, with its notes visible above the time footer. |

Static renders validate appearance and prototype navigation only. Runtime calculations, data persistence, keyboard behavior, actual window resizing, and numerical parity remain implementation verification gates.
