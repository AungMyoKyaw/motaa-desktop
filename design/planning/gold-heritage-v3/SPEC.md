# MOTAA Gold Heritage v3 — Implementation-Ready Design Specification

**Status:** G01–G42 approved and native FigCraft implementation applied on 2026-11-17; automated structural/contrast checks passed. User visual signoff pending.  
**Approval:** 2026-11-17; G01–G42 = A; zero overrides.  
**Visual theme:** Modern Heritage / Quiet Gold; restrained gold, ivory canvas, bronze controls, original chart color semantics.  
**Authoritative native file after a later execution authorization:** design/motaa-desktop.figcraft (sole editable FigCraft master).  
**Companion contracts:** DECISIONS.md; TOKENS.json; COMPONENT_CONTRACTS.md; EXECUTION_PLAN.md; REVIEW_GATES.md; READINESS.md.

## 1. Why this is a v3 design, not a new product

MOTAA Desktop recreates recovered MOTAA V31.1 workflows for working astrologers. Maintain the mature Native Precision v2 macOS shell, compact sidebar, unified toolbar, contextual inspector, reusable editable Lucide, precise tables, dense traditional charts and five workspace navigation. Replace only the v2 System Blue/Slate **shell visual identity** with Quiet Gold. Avoid generic dashboards, metallic gradients, ceremonial decoration and invented astrological marks. The desktop window must feel native, competent and calm rather than decorative.

The original project DESIGN.md already describes warm cream/gold. The presently implemented native master, design/tokens.css, design/index.html and NATIVE_PRECISION_V2_REVIEW.md reflect blue/slate v2. This is not a contradiction once status is explicit: **v3 is the approved future palette, v2 is the current factual baseline**. Preserve v2 audit evidence; after separate authorization and verified promotion, update canonical current-state documentation with new evidence.

## 2. Protected invariants and ownership

- **One master / 100 pages**: 90 product feature/state references and 10 astronomy or algorithm teaching pages. Preserve every page and existing application-state/education ownership. No new top-level product routes.
- **Five workspaces**: Charts and modes (01 Natal chart); Profiles and locations (31 Profile library); Calculation setup (33 Calculation options); Time and analysis (34 Time navigator); Advanced analysis (11 All tools). The source route registry is design/APP_ROUTES.json; design/APP_CONSOLIDATION.md is its rationale. Never use this specification to create 90 application routes.
- **Baseline native structure** (reported by the latest v2 audit): 22,812 native nodes, 44 variables in UI/chart collections, 23 styles, 35 real Lucide source-backed components, 450 linked icon instances, 19 component families with 76 state variants, 2,160 navigation interactions, 35 variable selection actions. Capture actual counts anew before editing; the numbers are baselines, not invented permanent ceilings.
- **Chart fidelity**: retain original Rasi/square, circular, Rahu, Nakshatra, Tricakka, Western, LatLon, periods, aspects, strength data symbols, original planetary color classifiers, original chart-coordinate geometry and 4 chart display modes (Light/Pink/Blue/Monochrome). Do not map shell accent gold onto source chart marks.
- **Feature availability**: all recovered modes, including original mode 11 Rasi aspects, remain accessible to everyone. No login, payment, membership, device activation or premium gating introduced. Advertised-only advanced numerical outputs require sourced mathematical implementation and may not be passed off as computed results.
- **Language**: English UI, English 0–9 digits, source-faithful Burmese glyphs/words. No automatic rewriting of original astrology terms.
- **Execution boundary at initial planning approval:** No edits permitted then. A subsequent separate explicit execution instruction authorized FigCraft changes and derived design artifacts only. Application code, Git history, hosting and deployment remain untouched.

## 3. Experience and visual hierarchy

A. **Window**: warm ivory window (#F8F5EE), slightly deeper warm sidebar (#F1ECDF), quiet near-ivory toolbar (#F9F7F2); nearly white calculation/chart canvas (#FFFEFC). Use flat native surfaces, thin understated borders and 4/8/12px radii. No texture, gradient, gold shadow flood, black/gold luxury theme or bright metallic text.

B. **Gold role**: antique gold (#8B6B2E) for identity and select accents, deep bronze (#7C5B21) for primary action labels, white-on-bronze buttons, focus outlines, active markers and semantic links. Champagne (#B68A3C) is decorative only, never ordinary small text on white. Limit gold to orientation and action; user attention belongs on the chart.

C. **Selected state**: pale gold background (#EDE0BF), selected text (#614719), plus a 3px deep-bronze left or leading indicator. Selected state must remain distinguishable from pale-hover (#F3EADB) by color, indicator and accessibility labels. Avoid color-only communication.

D. **Interaction**: primary bronze default, darker bronze hover (#694A17), darkest pressed (#523A13), explicit focus 2px bronze outline with 2px offset. Secondary/tertiary controls remain neutral with readable ink and visibly bounded hit areas. Destructive actions remain danger red, not bronze. Loading states must not imply a calculation is complete.

E. **Info/status**: success green (#216E48), warning amber (#8A4B0F), error red (#B42318), informational blue (#315F9D), each with text and Lucide icon; these are semantic status colors rather than brand accents. Do not reuse chart-data colors as UI status colors or vice versa.

F. **Typography**: macOS system UI on runtime / Helvetica Neue in FigCraft, Myanmar Sangam MN for Myanmar data, Baskerville for MOTAA wordmark only. Arial Narrow retained only where required for source chart fit. Scale 11, 12, 14, 16, 20, 28 and 32px; line-height 1.25 labels / 1.45 body; numbers use tabular figures and align right in numeric columns. Avoid unlicensed bundling of system fonts.

G. **Shape/density**: 4px spacing grid, page inset 24px (16px compact), row heights 32px compact / 36px regular / 40px prominent, sidebar rows 36–40px. Min CSS target 24px with spacing/target exceptions audited; dense charts retain contextual zoom rather than arbitrary larger source geometry. Suppress floating ornamentation.

## 4. Screen-level composition

### Charts and modes (01 Natal, 02 Circular, 12 Tricakka and family)
Retain original chart grid/wheel geometry, order, axis and data notations. Give chart content the bright flat canvas; keep border and labels in the preserved chart-data namespace. Gold highlights **UI selection, toolbars and chart-frame chrome only**, never reclassifies planets, houses or wheel segments. Mode picker remains reachable with search/help and genuine mode states. Do not flatten charts to screenshots.

### Profiles and locations (31 library, 32 editor and states)
Show named profiles, selected row, create/edit/duplicate, undo deletion, search/empty/import/conflict states with warm neutrals and gold selection. Separate save/cancel and destructive controls. Keep location coordinates, timezone, known/unknown birth-time and validity rules unchanged. Treat date and time values as reference or user input, not invented calculations.

### Calculation setup (33 options and pickers)
Stable column rhythm and label/value hierarchy. Bronze action on Save to profile, neutral on Cancel or Reset, gold selected segmented values, clear transient workspace override versus persisted profile choice. All nine ayanamsha, eight house, two zodiac and two lunar-node selections stay available.

### Time and analysis (34 navigator and analysis)
Preserve six time units, independent epochs, periods/subperiods, calculated-range recovery and Go to start/end semantics. Gold indicates UI-selected unit or active period; it does not recolor period algorithm data. Prevent stale child values when out of range. Show source-derived vs illustrative state.

### Advanced analysis (11 All tools, 21–30 etc.)
Keep the complete tool directory and existing user access. Gold anchors active workspace, headers and primary actions; detailed matrices, ephemeris, calendars, graphs and legends retain independent semantics. Advertised-only algorithms keep explicit unverified/source-needed status; never fabricate numerical results.

### General states and compact layouts
Include empty profile library, invalid birth date, chart zoom, unavailable calculations, import review, disabled/loading/error, destructive undo and unsaved changes. Avoid blue v2 leftovers in any generated app chrome. Ensure gold state is visible at 1024 width without truncating Burmese labels; compact icon-only rail retains tooltips and accessible names.

## 5. Theme and ownership architecture

Two distinct token namespaces with distinct intent:

1. **ui.***: all desktop shell/button/selection/typography/surface/status tokens defined in TOKENS.json, Light only. Materialize these as native semantic FigCraft variables/styles only in a later authorized design execution.
2. **chart.***: existing source-derived Light/Pink/Blue/Monochrome chart theme variables and their geometric/data semantics. Preserve exact values, ownership and behavior. All four chart themes remain independent of the gold UI shell. Blue in chart themes MUST NOT be treated as an unconverted v2 UI accent.

Theme switching changes chart appearance, not the shell or gold navigation. No dark-shell values are approved. Future dark mode would require a separate palette/contrast and review decision.

## 6. Desktop layout rules

| Fixed FigCraft review canvas | Sidebar | Inspector | Requirements |
|---|---:|---|---|
| 1440×960 | 216px expanded | 288px contextual | main chart and inspector remain legible, no arbitrary crop |
| 1280×960 | 200px expanded | 288px collapsible | preserve table/form readability |
| 1024×900 | 64px icon-only | 288px hidden by default | tooltip, name, focus and explicit inspector toggle |

Toolbar: at most three principal groups (navigation/title, optional context, trailing actions/inspector). Regular forms: two columns when space permits; compact forms: one. Fixed FigCraft references are NOT a validation of fluid Tauri window resizing; that is a future application acceptance criterion.

## 7. Components and state contracts

COMPONENT_CONTRACTS.md is normative for all 19 Native Precision families. Derive UI appearance from semantic tokens and retain real native editable component/link identity. At least Default/Hover/Focus/Disabled are kept for each of the existing 19 families; add documented selected, error, pressed, loading, empty or read-only states only as needed by that component. Document actual differences between definitions and placed instances; do not claim extra variants without native evidence. Icons remain pinned Lucide v0.468.0 genuine geometry, 1.75px stroke, 16/20/24px; if an upstream version changes, compare hash/path before migration.

## 8. Accessibility acceptance

The planned token pairs in TOKENS.json meet WCAG 2.2 AA visual target thresholds for their stated roles: 4.5:1 normal text, 3:1 large text and meaningful nontext indicators. The contrast manifest is a mathematical palette check only. Actual composed contrast, opacity overlays, antialiasing, inactive/hover states, chart accents and screenshots require independent checking. For original chart glyphs constrained by source geometry, provide magnification, legend and tabular alternatives rather than silently rescaling core charts.

FigCraft can specify meaningful focus, logical traversal, tooltip, semantic names, Escape/Enter, undo and keyboard equivalents; **actual** VoiceOver, input behavior and resize responsiveness remain unverified until app implementation.

## 9. Delivery and verification boundaries

After separate explicit FigCraft authorization, prepare a recoverable external candidate, snapshot hashes, design a representative pilot, obtain a pilot signoff, then apply semantic restyling to all 90 app/state pages. Preserve the 10 education pages, 100-page total, existing functional routes/actions, editable chart vectors, Lucide assets and tested source fidelity. Render and inspect all 100 pages, regenerate CSS and HTML **from master**, evaluate required contrast and page geometry, run the existing design regression suite and provide before/after evidence. Never weaken tests or silently edit records to make failures disappear. Only promote the candidate when mandatory gates pass and the user accepts visual quality. There is no authority to perform any of these implementation steps from this planning approval.

## 10. Decision precedence and known non-goals

For v3 shell palette/direction: G01–G42 and this spec supersede conflicting blue-shell styling in v2. For all unrelated structure, behavior, source-chart geometry, language and accessibility contracts: v2 remains authoritative. Existing SOURCE_TRACEABILITY, route ownership and verified audits remain evidence. Any contradiction in future execution must be resolved and documented prior to promotion. Do not delete features or repurpose the 10 educational lessons. No application release, numeric parity claim, dark mode, runtime behavior verification, paid gate, infrastructure change, chart algorithm or native AppKit conversion is approved here.
