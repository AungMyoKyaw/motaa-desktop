# MOTAA Desktop GPUI delivery loop

Run `bash docs/gpui/ralph.sh` from the desktop repository. The script reads the FigCraft master, validates the five workspace entries, renders them and the class diagram, and runs formatting, strict Clippy, tests, a locked build, and a bounded native smoke run. It stops at the first failed gate and preserves logs in the printed evidence directory. The smoke run uses an in-memory database and verifies the resulting workspace after each GPUI navigation action. It also exercises all twelve chart selections and the implemented advanced calculation and sky views.

| Iteration | Deliverable | Pass conditions |
| --- | --- | --- |
| 0 | Class diagram and design baseline | FigCraft entry renders, route checks, valid UML and unchanged source hash. Implementation authorized after diagram handoff. |
| 1 | Native GPUI shell | Pin upstream GPUI with a lockfile. Launch on macOS. Navigate all five workspaces by mouse and keyboard. Compare to pages 01, 11, 31, 33, 34 and 108 at their native dimensions. |
| 2 | Profiles and settings | Restart and round-trip saved data. Check independent natal and transit contexts, unknown birth time, location and timezone validation, dirty drafts, explicit scope, deletion undo and transactional backup import. |
| 3 | Time and recovered modes | Verify six step units, four epoch combinations, Dasha drilldown and boundaries. Trace every mode against fixtures. Check geometry, glyphs, aspect filtering, and mode 11 in both display states. |
| 4 | Advanced workflows | Keep all entries accessible. Add a supported calculation only when its formula, input range and evidence are documented. Check retries, preserved drafts and stale-result handling. |
| 5 | Native acceptance | Run the complete build and tests, inspect all shipped states at supported sizes, and record what has achieved original-output parity. |

For each implementation iteration, read the latest evidence and choose one failing or unfinished criterion. Record the FigCraft pages, behavioral fixture and commands. Implement the smallest complete slice. Run focused checks followed by `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` once the GPUI workspace exists. Exercise the flow in the native app and record screenshots when layout changes. Review the diff and record the result and next criterion. On failure, record the first actionable error and change the hypothesis before rerunning. Finish when all applicable criteria have passing evidence.

## Implementation decisions

- The GPUI workspace lives in this repository. Its engine, store and Java oracle fixtures were copied from the local `motaa-desktop--mcp` working tree based on `26b3553`, including local routine and fixture changes. The app uses a dedicated GPUI SQLite path. The upstream SQLite schema and fixtures exercise compatibility in temporary databases. Third-party notices live under `third_party/`.
- Use `design/motaa-desktop.figcraft` for visual fidelity and `APP_ROUTES.json` for the five workspaces. A shared chart scene feeds live GPUI drawing and hit testing. Keep the Gold Heritage shell and the four original chart palettes independent.
- Page 34 depicts scrubbing around system time. PRODUCT.md and the implementation plan specify scrubbing relative to a selected date. The diagram uses a selected-date anchor and keeps Now explicit.
- The compact visual baseline is page 108 at 1280 × 800. The app initially opens at 1280 × 800. The 1024 × 680 setting is a resize floor. Full visual and input acceptance at the floor is open.
- Store original-output evidence separately from independent reference calculations. Test historical offsets, coordinate limits, unknown time, source-recovered calendar options and the availability of verified results. Keep the zone, step unit and time anchor explicit in requests and expected outputs.
- GPUI references consulted through find-docs and Context7 cover `App::open_window`, `Render`, entity-backed views and `Context::notify`. The app is locked to GPUI 0.2.2. [GPUI](https://gpui.rs/) · [Render](https://docs.rs/gpui/latest/gpui/trait.Render.html)

## Current delivery status

Run `bash docs/gpui/ralph.sh --production` to enforce the full release criteria. The default runner logs their status alongside the development checks. [Production criteria](production-criteria.json)

| Area | Verified or implemented | Remaining |
| --- | --- | --- |
| Shell | Native macOS launch, five workspace actions, keyboard focus traversal and `Enter`/`Space` activation, compact 1024 × 680 resize, twelve chart selections, retrograde (`R`) indicators, MUNP across all five Android `Y0` modes, all four chart palettes across all chart views, advanced destinations, fixed toolbar, chart zoom/pan, and 15-state live visible-window screenshot QA (`--visual-capture`). | Physical mouse hit-testing (blocked in hidden smoke by GPUI's private `DispatchEventResult`) and native macOS file-picker automation. |
| Profiles | SQLite create, edit, duplicate, search (including empty-match state) and deletion undo. JSON v2 restores chart records, settings, favorites, tags and revision history transactionally; malformed backup files are rejected and valid backups pass review/apply. v1 imports remain supported. Offline Myanmar town directory, DMS input, IANA timezone or manual offset and unknown-time handling. | Native macOS file-picker dialog interaction checks. |
| Time and settings | Six calendar step units, date-relative mouse and keyboard scrubber, saved profile settings, dual planet/Ascendant epochs, eight house systems, nine mean ayanamshas, two nodal methods, all three original Myanmar-calendar day-number conventions, and V31.1-verified Myanmar calendar + `main.A0` daily observances (31 watat/era boundary years and 20 civil dates). | Broader epoch-sensitive visual comparisons. 5D/15D persists as a display preference (confirmed dead code in V31.1). |
| Charts | Corrected eastern Ascendant, reference Rahu Moon/manual options, recovered circular radii and syllables, three-level Dasha drilldown (all 140 major starts, 1,260 Antardashas, and 11,340 Pratyantardashas within 0.5 days of Android Moon across 14 cases), traditional relationships and score views, MUNP across Rasi/Tricakka/LatLon/Natal+transit/Rasi aspects, and Tricakka. Fourteen shipped-APK cases cover all ayanamshas and eight house systems. | Full original geometry, glyph, true-node state vector, and visual output parity. The reference planet model covers 1800–2100, with the Pluto series published for 1885–2099. |
| Advanced | Ashtakavarga, D2–D12 variants specified by the design, KP, heliocentric coordinates, Panchanga transitions and rise/set, lunar and solar eclipse catalog and local circumstances (including high-latitude grazing and below-horizon central phase classification), ephemerides, stations and ingresses, Tara/Special groups, Tatkalika, Sarvatobhadra, strength and candidate comparison, interpretations, and rotatable sky projections. | Verified Ten pillars/Ayu Moon and Satapati-specific rules. Full product parity for approximated statistics, local eclipse contacts, interpretations and sky layers. |
| Distribution | Ad-hoc-signed macOS `.app` bundle (`bash scripts/build-macos.sh`), strict signature verification, and native smoke. Optional `MOTAA_DEVELOPMENT_ID` / `MOTAA_DEVELOPER_ID` supported. | None (ad-hoc signing selected; no Apple Developer account). |

## Original runtime evidence

The Android oracle ran the numeric entry points in the original MOTAA V31.1 APK on an isolated Android 35 emulator. The installed base APK matched SHA-256 `f68d54931c721d82c8056a919e679c26c36113c3b191a575f32dd227b8f31734`. The test cases include 168 longitude comparisons, 1,344 cusp comparisons and fourteen Ascendants. Peak absolute angular errors were 0.016694° for bodies, 0.010401° for house cusps and 0.005041° for Ascendant. The Pakhangyi Dasha fixture reproduces ten visible Android start dates when supplied the original Moon longitude. Using the desktop-computed Moon shifts the first displayed start by two days in this same profile. [Runtime test](../../crates/motaa-engine/tests/apk_runtime.rs) · [Capture script](../../scripts/capture-apk-oracle.py)

## Calculation references

- Dasha uses the actual `main.W` source rather than the earlier report paraphrase. Backdating uses the first lord's period weight. The displayed balance uses a separate day/year constant, major boundaries use civil anniversaries and the final subdivision ends at its parent boundary.
- KP classification starts each subdivision cycle at the parent lord. The tool uses the selected sidereal ayanamsha and house system for natal and transit positions and house centers. Its original-output acceptance remains open. [KP proportional rule](https://vedicreader.com/j/learn/kp) · [Reference boundary table](https://www.vedicdream.com/home/kp-sub-lord-table)
- The production gate remains open until the remaining tools have verified results and the original chart, calendar, persistence, and native acceptance criteria pass. The ad-hoc-signed bundle is a local preview.

- Ashtakavarga uses the Brihat Jataka convention. The implementation checks every cell in the published Libra-Ascendant example and the fixed total of 337. [Worked example](https://jatakaveda.com/library/study/ashtakavarga/)
- Divisional charts follow Parashara with Narasimha Rao conventions for D5 and D6. [Parashara, chapter 6](https://vedicpupil.in/library/books/brihat-parashara-hora-shastra/chapter-6) · [D5](https://jagannathahora.com/panchamsa) · [D6](https://jagannathahora.com/shashthamsa)
- Local solar eclipse contacts use topocentric disk geometry. Lunar penumbral bounds use catalog duration around greatest eclipse. The reference window is 2001–2100. [NASA solar catalog](https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html) · [NASA lunar catalog](https://eclipse.gsfc.nasa.gov/LEcat5/LE2001-2100.html)
- Electional timing and Tatkalika calculate local solar horizon crossings and use the documented angular Panchanga divisions. [Astronomical conventions](https://cran.r-project.org/web/packages/VedicDateTime/vignettes/VedicDateTime.pdf)

## Live Android parity iteration

Run `MOTAA_ANDROID_SERIAL=emulator-5554 MOTAA_ANDROID_D8=/path/to/build-tools/d8 bash docs/gpui/ralph.sh --android` with the isolated MOTAA emulator online. Use the serial reported by ADB. The runner verifies the installed V31.1 APK hash and captures synthetic inputs into its evidence directory. It then compares the capture against the desktop engine under the existing angular thresholds. The live corpus adds a high-latitude case, two dated range edges, and a fractional-hour offset. The committed baseline still runs on every normal build. The live runner also creates a synthetic profile database with the emulator SQLite executable and the original table definitions. It pulls the file into the evidence directory and verifies import, duplicate handling, reference location, and desktop backup serialization. This fixture contains synthetic data. A database written by the original Android Save workflow has also passed the native picker and chart restoration. The optional live gate checks that same sample when `MOTAA_ANDROID_ORIGINAL_SAVED_DATABASE` points to its copied `commments.db`.

A live discrepancy is a failing gate. Inspect its evidence, reproduce with the corresponding input, fix the affected calculation or explicitly document the unsupported behavior, and rerun. A passing numeric gate certifies only its tested inputs. Keep migration, native interaction, and premium acceptance open until those have their own evidence.

Combine `--android --production` to run live checks and fail on unfinished release criteria. The calendar gate invokes the installed APK's `main.w0` across the 1582 boundary, leap-year examples, and supported range endpoints. The captured 15-case calendar baseline also runs without an emulator.

Use `--background` to run the design, build, test and optional Android gates without launching the desktop window. A background run cannot certify production because it skips native smoke.

The native smoke test runs with its GPUI window hidden and does not request application activation. It explicitly draws workspace, chart and advanced views. Visible interaction and screenshot acceptance remain separate release checks.
