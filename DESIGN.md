---
name: MOTAA Desktop
description: A cream and gold desktop chart workbook preserving Burmese astrological notation.
colors:
  primary: "#7C5B21"
  background: "#F8F5EE"
  paper: "#FFFEFC"
  rail: "#F1ECDF"
  ink: "#28251F"
  muted: "#686053"
  line: "#D9CDBB"
  selected: "#EDE0BF"
  soft: "#F6F1E8"
  error: "#B42318"
  data-green: "#326C4E"
  data-blue: "#315F9D"
  success: "#216E48"
  warning: "#8A4B0F"
  action-hover: "#694A17"
  action-pressed: "#523A13"
  focus: "#7C5B21"
  disabled: "#EAE5DC"
  disabled-ink: "#6E655B"
typography:
  body:
    fontFamily: "Helvetica Neue, Myanmar Sangam MN, sans-serif"
    fontSize: "14px"
    fontWeight: 400
  title:
    fontFamily: "Helvetica Neue, Myanmar Sangam MN, sans-serif"
    fontSize: "28px"
    fontWeight: 500
  helper:
    fontFamily: "Helvetica Neue, sans-serif"
    fontSize: "12px"
    fontWeight: 400
  wordmark:
    fontFamily: "Baskerville, serif"
    fontSize: "28px"
    fontWeight: 650
rounded:
  control: "6px"
  field: "5px"
  navigation: "7px"
  panel: "12px"
spacing:
  unit: "8px"
  content-inset: "32px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.paper}"
    rounded: "{rounded.control}"
    height: "38px"
  button-secondary:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "38px"
  field:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.field}"
    height: "44px"
  navigation-selected:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.ink}"
    rounded: "{rounded.navigation}"
---

## Overview

**Creative North Star: "The desktop chart workbook"**

MOTAA is a cream and gold working chart book translated into a desktop workspace for astrologers using an English interface and traditional Myanmar chart notation. Traditional square and circular chart geometry occupies the main working surface, while a persistent rail, active birth profile and calculation setup give the dense notation context. This document records the first FigCraft design; it does not describe an implemented Tauri application.

**Key Characteristics:**

- English interface labels and English digits, with source syllables retained as linguistic chart data.
- Flat paper surfaces, quiet rules and restrained gold selection.
- Drawn chart geometry and readable data symbols.

## Colors

Primary gold identifies the main action and selected controls. Selected gold is the quiet fill for active navigation and selected periods; soft cream groups calculation context. Background, paper and rail distinguish the workspace without shadows. Ink carries chart geometry and primary text; muted ink supports secondary labels; line separates rows and fields.

Error red marks invalid input and also appears in source-derived planet notation. Data green and data blue distinguish chart symbols; retain textual notation because these colors alone do not communicate meaning. These data colors are not additional action accents. The circular diagram additionally preserves the original house-marker palette (yellow, cyan, pale blue, green, pale gold, red, gold and lavender) and the original planet-color classifier; those colors belong only to chart data. Exported CSS names remain in [design/tokens.css](design/tokens.css): `gold` corresponds to primary, `bg` to background, `red` to error, `green` to data-green and `blue` to data-blue.

## Typography

Body and screen titles use English. The artwork explicitly chooses Myanmar Sangam MN for strings containing Burmese and Helvetica Neue for Latin strings; the family stacks above express the intended frontend fallback relationship. Helper labels supply context below English headings. Baskerville appears only in the MOTAA wordmark. Circular ring names use Arial Narrow to improve fit without shrinking the numeral rings. In the 760 px detail view, quarter codes and inner indices are approximately 12.7 px, ruler numbers 15 px, planet numbers 14.3 px, and the full name key 16 px. Marker text uses black on red/green data fills and white on black/blue fills for stronger contrast.

**The Notation Rule.** Use English digits (0–9) for every numeric label, planet code, ruler, quarter and index. Mode 9 retains its 108 source syllables as linguistic data notation. Nakshatra names follow the exact spellings in the user-provided contents image; narrow ring labels may abbreviate long names, while the detail view presents all 27 names in full at 16 px. Preserve degree signs, minute marks and retrograde indicators as text. Myanmar line height and fallback fonts require validation in the future frontend; the static design does not establish a CSS line-height token.

## Layout

The verified canvas is 1440 × 960, with a 208 px rail and 60 px header. Content begins 32 px beyond the rail. The main chart sits beside a planet table; periods use three adjacent hierarchy columns. Circular mode has a linked detail screen with a 760 px wheel and a complete English nakshatra name key. The profile editor pairs a broad form with a narrower last-valid chart preview. Chart exploration screens retain a 98 px time footer.

The spacing unit is 8 px, with local optical adjustments for Burmese and data tables. A minimum 1180 × 800 window is proposed, but has not been verified; no responsive breakpoints are established. Detailed screen dimensions and navigation scope live in [design/SPEC.md](design/SPEC.md).

## Elevation & Depth

The design is flat. Paper and rail tones, one-pixel rules and selection fills separate regions. No shadow vocabulary or motion behavior is established in the prototype.

## Shapes

Controls have gently rounded corners; fields are slightly tighter, and selected rail items slightly softer. Chart borders, sectors and table grids stay geometric. The traditional square chart retains its outer square, internal thirds and diagonal corner divisions. The mode-9 circular chart preserves source radii 240/220/145/125/95/78/60, 27 nakshatra sectors and 108 subdivisions. Its colored zigzag connects house midpoints to adjacent cusps; it is not an aspect network. Within the numbered ring (radii 125–145), light 0.4-unit radial separators (#B7B39F) divide each of the 108 cells; the existing darker boundaries remain every fourth cell for the 27 nakshatras. Apply the same separators to the outer syllable ring (radii 220–240), keeping all 108 cells aligned radially. Icons use small line drawings rather than decorative illustrations.

## Components

Primary buttons use `{colors.primary}` with `{colors.paper}` text, `{rounded.control}` corners and the established control height. Secondary buttons use paper, ink and a line border. Disabled actions use line fill and muted text. Fields use paper and line borders; the invalid-date field switches its border to error red and adds a textual explanation.

Selected navigation uses `{colors.selected}` and `{rounded.navigation}`, pairing an icon with an English label and a short English description. Calculation settings sit in a soft horizontal strip. Data rows use thin separators rather than individual cards. The aspect matrix combines geometry, symbols and a written legend; period rows pair dates, planetary numerals and a direction arrow.

**The Reference Rule.** Visible chart values, period dates and aspect marks remain identified as reference or illustrative fixtures until calculation parity is independently established. A drawn Save control or prototype link is not evidence of persistence or computation.

Hover, keyboard focus, active press and live form states are not implemented by the static artifact. Their visual treatment and accessibility require frontend validation.

## Do's and Don'ts

### Do:

- Do use English for every interface label, action and validation message.
- Do reserve Baskerville for the MOTAA wordmark.
- Do preserve traditional chart geometry and numeric values while displaying English digits.
- Do keep birth details and calculation setup visible while exploring charts.
- Do pair data color with symbols, labels or a legend.
- Do distinguish reference fixtures from verified calculation output.

### Don't:

- Don't replace charts with screenshot backgrounds.
- Don't use data red, green or blue as competing primary actions.
- Don't add shadows or gradients to the recorded paper surfaces.
- Don't turn dense result rows into a stack of decorative cards.
- Don't claim smaller-window behavior from the fixed desktop canvas.
- Don't treat navigation hotspots as working forms, saving or calculations.


## Native source-informed extension · November 2026

The sole FigCraft master contains 90 screens: the original 58 and 32 additional source states, with four native theme-variable modes, six styles, two shared components and 35 instances. Source-faithful time scrubbing anchors to current system time. A restricted member-only mode and simulated activation flow are represented. Existing original chart geometry remains unchanged. Smaller 1280 and 1024 layouts are reference frames, not a responsive application runtime. See design/SOURCE_TRACEABILITY.md for the evidence map.


## Circular algorithm teaching pages, November 2026

The authoritative FigCraft master now includes six additional native editable pages 91–96, explaining circular chart Mode 9. These use recovered ring radii, sign / Nakshatra / Pada subdivisions, planetary placement with collisions, twelve house cusp-to-midpoint connectors and final layering. The original 90 pages and their source geometry are preserved. The new longitudes and Equal-house connector positions are illustrations, not a live ephemeris or numerically verified APK results.

## Swiss Ephemeris learning screen

Page 97 traces the actual Moshier/Lahiri lunar-position call at UT Julian day 2451545.0, and reproduces sidereal longitude 195.24696056 degrees from tropical 219.03172767 degrees and Lahiri adjustment 23.78476711 degrees. Existing page 92 now links to this explainer. No personal birth date is stored in the tracked FigCraft master and no new calculation engine exists.

## Geographic astronomy lesson extension

Pages 98–100 use source-mapped FigCraft native shapes to explain geographic dependency of the sidereal Ascendant and houses. A reproducible independent Swiss Ephemeris/Lahiri/Equal fixture feeds the twelve-house table, then the source-defined black-cusp and colored-midpoint connector algorithm. The 97 prior pages remain intact, and the native master now contains 100 pages. Neither a live ephemeris nor original APK numerical parity is implemented.

## Native FigCraft quality audit · November 2026

A 100-page FigCraft CLI audit fixed header layer ordering on 33 screens, three covered theme captions, and nine off-canvas horizontal separators. All 39 affected screens were rendered and compared, and 100 page exports regenerated. See design/QA_REVIEW.md and design/verification.json. The master is still a static prototype, not an implemented calculation engine.


## Consolidated application structure · 17 November 2026

The 100 native FigCraft pages are now organized into five prototype application flows using existing source-faithful screens instead of generating redundant dashboards: Chart / Natal (01), Profiles (31), Calculation Setup (33), Time & Analysis (34), and Extensions & Access (11). The complete 90-state ownership matrix and 10-lesson annex are documented in design/APP_ROUTES.json and design/APP_CONSOLIDATION.md. Screen contents, original chart geometry, 1,296 navigation destinations and 35 variable actions are preserved. Native prototype flow changes affect metadata only; all five source screens were verified pixel-identical to the previous output.

## Everyone gets all modes — 2026-11-17

The desktop edition has no premium, subscription, license, membership, device activation or password requirement. Unlike the original Android access control, Rasi aspects (mode 11) opens directly from All tools. The other ten source modes and every designed advanced entry remain available. Native pages 59–63 now describe included features rather than authentication results; page 64 preserves the source-mode output geometry. Original access evidence stays in the reverse-engineered repo, and numerical parity remains unverified.

## macOS-style visual review · 17 November 2026

The canonical FigCraft master was restyled across 90 product/state pages to resemble a cohesive macOS desktop tool: restrained system chrome, compact navigation, blue native-style actions, grouped white controls, clear forms and contextual inspector panels. Ten algorithm lessons remain untouched, and original chart-plot geometry was preserved. Verification: design/MACOS_DESIGN_REVIEW.md and design/verification.json. This design does not yet implement an actual macOS application.


## Current visual authority: Gold Heritage v3 (2026-11-17)

The canonical FigCraft master now uses **Modern Heritage Gold**: warm-ivory window #F8F5EE, antique-gold identity #8B6B2E, and deep-bronze primary actions #7C5B21. Native Precision v2's macOS layout, 19 component families, 450 Lucide placements, 100 pages, 2,195 prototype actions and four original chart-display themes remain. See design/GOLD_HERITAGE_V3_REVIEW.md and design/planning/gold-heritage-v3/.

## Historical visual authority: Native Precision v2 (2026-11-17)

At the Native Precision v2 milestone, the legacy cream/gold frontmatter was historical; the then-current FigCraft used **System Blue / Slate, light UI only**, with semantic UI variables separate from the four original chart themes. Primary actions use accessible #1D61D6 and brand accent uses #3378F6. Native assets contain source-backed Lucide 0.468.0 icons and reusable component-family states. See design/NATIVE_PRECISION_V2_REVIEW.md and the approved design/planning/native-precision-v2/SPEC.md for the implemented direction. Keep source-chart geometry/notation intact and do not fabricate numerical outputs.

## Source parity extension (November 2026)
The sole native FigCraft master now includes four additional Gold Heritage feature-state pages, retaining the original 100, the five Mac workspaces, all source chart geometry and universal free access. Earlier figures above are historical milestones; the current design has 104 pages.

## Design-system architecture and component states · 21 November 2026

The native master separates **MOTAA / Primitives** from **Gold Heritage / UI** semantics and component contracts. Raw ivory, gold, bronze, ink, feedback, spacing, control-size, radius, focus, and motion primitives feed preserved semantic variable IDs. Component color, control, selection, feedback, focus, and motion aliases consume those semantics. The four MOTAA chart-display themes remain independent and never restyle desktop chrome.

Page 110 makes default, hover, pressed, keyboard-focus, disabled, loading, error, success, and selection states visible. Controls use the 38px height, 6px radius, 8px spacing rhythm, and a 2px visible focus ring. Page 108 applies the same system at a real 1280×800 desktop size without flattening the chart.

## Observatory Manuscript UI/UX Overhaul · 2026-11-25

The GPUI desktop runtime (`src/`) was overhauled under the **Observatory Manuscript** direction while preserving the Gold Heritage light-only palette, traditional square/circular chart geometry, English digits, and the four independent chart palettes:

- **Token & primitive layer (`src/theme.rs`)**: Centralizes light-only parchment/ivory/bronze tokens, semantic badges (`Tone::Neutral`, `Gold`, `Error`), tabular table headers/rows (`table_header_row`, `tabular_row`), and shared `empty_state` / `error_banner` primitives.
- **Three-column macOS workbench (`src/main.rs`, `src/inspector.rs`)**: Pairs a collapsible left workspace rail (`208px` expanded / `64px` icon-only, `⌘B`), a `52px` live context header (active workspace, natal profile, reference timestamp, calculation summary, and `Quick jump ⌘K`), and a collapsible `284px` right contextual inspector (`⌘I`) for birth profile, calculation toggles, epoch overlays, and chart palette swatches.
- **Quick-Jump Command Palette (`src/palette.rs`)**: `⌘K` opens a keyboard-first fuzzy launcher across all 12 chart modes, all 16+2 advanced tools, saved profiles, workspaces, and workbench actions.
- **Streamlined Charts & Time (`src/chart_view.rs`, `src/time_view.rs`)**: Replaces the multi-row toolbar with primary mode pills + `More modes ▾` and tightens the time footer to `64px` with an inline step-unit cycler, `⌘[` / `⌘]` / `⌘⇧N` shortcuts, and a center-marked scrubber.
- **Profiles & Advanced Directory (`src/profiles.rs`, `src/datetime_picker.rs`, `src/advanced.rs`)**: Adds match counts and empty states to Profiles, deduplicates stepper/month controls in `datetime_picker.rs`, and groups the Advanced directory by astrological domain with unified `tool_header` breadcrumbs.
