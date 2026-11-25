# MOTAA Gold Heritage v3 — Native Component Contracts

**Status:** Approved Gold Heritage v3 component contract; native FigCraft palette migration executed. Runtime interaction semantics still require application implementation.  
**Baseline:** retain the 19 named v2 component families, 76 native variants (four foundational variants each), 35 source-backed Lucide components and 450 existing icon instances. Existing links and variable selections remain untouched until separate authorized execution.

## Shared foundation

- All shell component fills, strokes and typography must resolve through TOKENS.json ui.* semantic tokens, not hardcoded per-screen hex values. Existing chart.* is never reparented to the UI tokens.
- Gold selection: pale gold #EDE0BF, selected foreground #614719, plus visible 3px leading indicator #7C5B21 where rows/tabs allow. Selected != hover (#F3EADB). Primary CTA uses #7C5B21 / #FFFFFF. Focus uses 2px bronze ring at 2px offset, never color-only.
- Native visual sizes: 32px compact, 36px regular, 40px prominent; sidebar rows 36–40px; Lucide at 16/20/24px and 1.75px round stroke. Text and icon alignment checked at 1x and 2x; arrows and chevrons remain aligned to text baselines. Interactive boundaries on pale surfaces must use at least 3:1 against adjacent surfaces when sole identifying outline.
- Each of the 19 existing native families retains real editable Default/Hover/Focus/Disabled assets. Extra states below are logical/UI contracts; implement extra native variants only when needed, and document exact changes in the component registry.
- Avoid chart-data color tokens in generic buttons, navigators, dialogs and form validation.
- Status meaning always has text/icon, not color alone. Disabled controls must remain identifiable, with disabled purpose and alternative path documented where essential. Source-accurate chart text receives zoom/legend/table fallback if too small.
- Tab order, Escape/Enter semantics and VoiceOver descriptions are future application requirements, not capabilities proven by static FigCraft output.

## Family-by-family specification

| Native v2 family | Gold visual contract | Required logical states / behavior | No-regression invariant |
|---|---|---|---|
| WindowShell | Ivory #F8F5EE window, warmer #F1ECDF sidebar, flat chrome | active/inactive/fullscreen/resized reference; inactive must not fake focus | preserve macOS window proportions and canvas |
| SidebarNavigationItem | neutral idle, #F3EADB hover, #EDE0BF selected, 3px bronze leading bar | idle/hover/focus/active/disabled/collapsed/expanded; active destination persists | five workspaces and original destinations |
| ToolbarAction | neutral surface, bronze icon/action text only for priority | idle/hover/focus/pressed/disabled/loading/icon-only/icon+text | up to 3 main groups; original tool handlers/links |
| InspectorPanel | near-white contextual surface with warm dividing rule | visible/collapsed/compact overlay/no selection | existing inspector width 288px |
| Button | white-on-bronze primary, outlined neutral secondary, inline bronze tertiary, red destructive | default/hover/focus/pressed/disabled/loading/icon-only; no gold destructive | role, button label, hit target and meaning unchanged |
| TextField | light canvas, #8E7F69 interactive boundary, bronze focus, red error | idle/focus/valid/invalid/disabled/read-only/required/optional/unit | retain all data labels and units |
| CoordinateInput | neutral coordinate rows and unit fields, not gold numerical data | decimal/DMS/N/S/E/W/invalid/manual override | source data precision and hemispheres |
| BirthDateTimeInput | clear birth/current groups and manual timezone controls | known/unknown time/offset/historical/manual/invalid | no invented birth time or fallback value |
| ChoiceControl | pale-gold selected track/cell, bronze text/indicator | select/segmented/radio/checkbox/selected/disabled/focus | choice sets: zodiac, house, node, ayanamsha etc. |
| SearchAndFilter | neutral bordered search; bronze on active applied filter | idle/query/filtered/loading/empty/cleared | filtering and search states preserved |
| ProfileListRow | flat row with gold selected highlight and clear action reveal | default/selected/editing/duplicate/deleted+undo/import conflict | profile content and undo behavior maintained |
| DataTable | crisp canvas, neutral dividers, right-aligned tabular numbers | default/sorted/selected/loading/empty/horizontal overflow | values/units/alignment, source table semantics |
| ChartContainer | pure #FFFEFC visual work area; no global gold recoloring | square/circular/LatLon/Rahu/Nakshatra/zoom/pan/legend/export | exact chart vectors and data-color classes |
| ChartModePicker | warm-neutral selectable list with gold active mode | mode1–11/selected/search/help | all modes reachable, including Rasi aspects |
| TimeNavigator | bronze active step/control chrome, not period-data rewriting | six units/epoch combinations/out-of-range | period calculations and time-step logic unchanged |
| Overlay | quiet white sheet and warm shadow/border (restrained) | menu/popover/sheet/dialog/dismissed/keyboard-focus | close, escape, return-focus and focus bounds |
| StatusFeedback | semantic green/amber/red/blue + Lucide + readable status text | success/warning/error/info/undo/inline | no success claim for unverified calculation |
| EmptyLoadingErrorState | readable action/next step; no decorative gold banners | empty profiles/results/invalid backup/pending/unverified/error | correct recovery and source-uncertainty copy |
| ChartLegend | original source color swatches with text and zoom support | colored/text-alternative/zoomed | data classifiers remain intact |

## Specific patterns

### Sidebar and tabs
- Active state needs 2 distinct visual signals: fill + 3px marker or label/icon weight. Hover must disappear when mouse exits; keyboard focus ring can coexist with active fill. At 1024 width, show icon-only rail with tooltip and accessible name; do not shrink the actual SVG geometries.
- Sidebar remains 216px at 1440, 200px at 1280, 64px at 1024. No extra top-level nav items without distinct user approval.

### Toolbar, actions and controls
- Primary CTAs use #7C5B21 fill with #FFFFFF text; hover #694A17; pressed #523A13; visible focus #7C5B21 outside control. Brand #8B6B2E can be used for text/decorative brand moments on near-white canvas but not on the sidebar where it may fail normal-text contrast.
- Champagne #B68A3C is unsuitable as ordinary text/primary white-label background; use only nonessential decoration, including brand motif. Destructive buttons keep #B42318, confirm using text with context, and provide undo where the existing workflow supports it.
- Inputs use accessible #8E7F69 boundary where boundary itself establishes affordance. Thin #D9CDBB dividers are decorative and cannot by themselves identify an interactive field.
- Fields with units keep units in the same row or as accessible associated labels; no gold decorative substitution for longitude, UTC offset, days, or coordinates.

### Chart, dense tables, reference numbers
- Gold can mark selected UI mode, legend header or utility control, but NOT the 12 signs, 27 nakshatras, 108 padas, source planet/house colors, Rahu group divisions or original wheel geometry.
- Do not upscale source charts by transforming underlying vector positions. ChartContainer provides independent zoom/pan and ChartLegend provides textual data alternative.
- Numeric comparisons use aligned tabular figures; include degree symbol, units, UTC offset and relevant legend. Dates and unknown birth time must remain clearly distinguishable.
- Warnings about incomplete research or illustrative examples must be plainly readable and persist through filtering and mode changes.

### Validation and user test cases
For each family export a 1x/2x state sheet; verify text vertical centering, icon baseline, stroke fidelity, active marker placement, focus ring clipping, backgrounds and state contrast. Run long English, Burmese, 0–9 numerals, invalid date, DMS, missing profile, import conflict, long calculator label and 1024px icon rail fixture. Check source-page states and native variant links, not screenshots alone.

## Component acceptance evidence

For each family, record: source component ID and instance counts; referenced ui.* variables; imported Lucide IDs/hashes where applicable; visual state names and screenshots; enabled/disabled behavior specifications; text and nontext contrast; 1x/2x layout checks; prototype links; exceptions and resolution. Failing text overlap, missing icons, broken links or unintended changed chart geometry blocks promotion. Every new reusable component lives in the **existing single FigCraft master**, not a second design file.
