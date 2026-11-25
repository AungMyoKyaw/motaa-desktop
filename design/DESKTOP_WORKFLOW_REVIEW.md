# MOTAA desktop workflow and source coverage review

21 November 2026 · FigCraft CLI 0.15.0

| Evidence | Design coverage and changes |
| --- | --- |
| Android V31.1 chart dispatch and calculation outputs | All 11 original modes are represented. The existing square and circular chart geometry, Western aspect legend, Dasha drilldown, Rahu and strengths states remain available. |
| Android coordinate, calendar and profile evidence | The existing design covers signed coordinates and offsets, independent birth/reference contexts, the 534-town directory, Myanmar calendar and observances. LatLon now spells out the five numeric fields, uses explicit units and right-aligns values. |
| Android reference time workflow | The navigator now opens the exact timestamp and current-location editors directly. Both return to the navigator. The three related screens use the same Yangon reference fixture and UTC offset. The scrubber retains the source-derived current-time midpoint and ±60-unit span. |
| iZarTarPro Font Size popover and `FontSizeSettingsView` | Added Standard, Large and Extra large (14, 17 and 20 px) plus reset. The FigCraft variable resizes the English/Myanmar specimen and the eight-row planetary coordinate table. Sidebar Settings now opens the display controls while the Calculations workspace opens chart setup. |
| Android calendar basis and stored aspect value | The original calculation controls explain the calendar basis and preserve imported `5D_Aspect` / `15D_Aspect` as legacy data. Their effect remains unconfirmed. |
| Advertised ZarTarPro modules and sibling sky capabilities | All 16 existing advanced entries, the two sky designs and the evidence register remain accessible. The source material establishes feature evidence and selected labels. Formula parity, iOS saved-note behavior and original Android database migration remain unverified. |

The FigCraft prototype exercises navigation and text sizing. Date/location entry, persistence, calculations and numeric parity require application work. The static HTML walkthrough reproduces native navigation and the standard-size renders.

Verified 110 editable pages, 2,431 navigation actions, 71 variable actions and 134 variables. Exported all 110 previews, the token CSS and walkthrough from the promoted master. Visually checked pages 07, 15, 34, 70, 72 and 102, plus pages 07 and 15 at 20 px.

Source evidence is recorded in the sibling [Android report](../../motaa-app/REVERSE_ENGINEERING.md), [algorithm dissection](../../motaa-app/MASTER_REVERSE_ENGINEERING.md), and [iZarTarPro report](../../motaa-app/IZARTARPRO_REVERSE_ENGINEERING.md).

Master SHA-256 `096828ad85c3d212300e6e6be71f964ff44aaee55b25029b35df2b9494ea0415`

Recovery copy `~/.local/share/motaa-desktop-backups/desktop-workflow-20261121.faaE8h/baseline.figcraft`
