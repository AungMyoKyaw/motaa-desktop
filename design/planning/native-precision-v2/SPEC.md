# MOTAA Native Precision: Implementation-ready design specification

**Status:** APPROVED DESIGN DECISIONS · FIGCRAFT EXECUTION AUTHORIZED IN FOLLOW-UP AND COMPLETED · APP CODE STILL NOT AUTHORIZED  
**Date:** 2026-11-17 · **Version:** Design System v2.0 (planning)  
**Decision record:** Q001–Q042 = A, except **Q009 = B (light first, dark later)**; no further overrides.

**Authoritative native design source, once future execution is approved:** design/motaa-desktop.figcraft (one file only). This specification, TOKENS.json and REVIEW_GATES.md are proposed planning contracts, not a second design master or implemented application styles.

## 1. Product direction

**Native Precision** is a professional, chart-first macOS experience informed by Preview, System Settings and Numbers. Use restrained window chrome, a unified toolbar, five-workspace navigation, a collapsible inspector, context-sensitive controls, legible numeric tables and rigorous typographic hierarchy. Reject generic dashboards, oversized decorative cards, arbitrary celestial ornamentation, AI-styled icons and reconstruction/workbook banners in the product UI.

One exact design language: System Blue / Slate, semantic foundations → components → patterns → screens, actual editable Lucide SVG vectors at 1.75px stroke, 16/20/24px sizes, balanced density, accessible input states, 4px spacing grid and a single FigCraft source of truth.

### Approved decision reconciliation

1. **Q009=B overrides the original default:** Light UI ships first. Dark is deferred; do not create or claim an unfinished dark appearance. Prepare semantic names to allow a later dark mode, but assign no production dark values yet.
2. **Q012=A is different from Q009:** Preserve the original Light/Pink/Blue/Monochrome **astrological chart themes** as independent chart-render settings. They are not four macOS shell appearances.
3. **Q007=A with Q034=A:** Retain the recognizable brand blue #3378F6; it measures 4.07:1 against white and is not acceptable for normal 14px link/button text. Use readable action blue #1D61D6 (5.63:1 on white) for blue text and filled controls with white labels.
4. **Q003=A and Q014=A:** Native macOS interaction/layout conventions, but **Lucide source paths**, not SF Symbols, icon fonts, hand-traced symbols, emoji or AI-generated icons. System-native controls remain preferred for non-icon structure.
5. **Q011=A:** Original chart data colors classify planets, houses and traditional symbols; never globally swap their red/gold/blue shades for interface accent tokens. Use text legends and zoom to improve access without altering mathematical meaning.
6. **Q034=A:** WCAG 2.2 AA for shell/controls and documented alternate zoom/tabular access for source-accurate chart lettering that cannot be enlarged without harming geometry.
7. **Q037=A:** 1440×960, 1280×960 and 1024×900 are fixed FigCraft review canvases, not proof of real runtime window resizing.
8. **Q039=A:** All recovered modes, including Rasi aspects (original mode 11), are available to every user, without paid tier, subscription, device activation, password or membership. Advertised-only iZarTarPro algorithms are distinguishable from verified original MOTAA code.
9. **Q040=A:** Exactly one *.figcraft in the repo. New library pieces are native components/pages within that master; candidate and recovery copies stay outside the repository.
10. **Q042=A:** This approval is for the final planning specification only. No master edits, application code, commit, push, publishing or deployment are authorized.

## 2. Inspected baseline and evidence

The canonical FigCraft master currently contains **100 pages, 18,376 native nodes, six styles and two variables**, with five existing prototype app flows. Pages **01–90** are application and UI-state references; **91–100** are algorithm lessons. Most canvases are 1440×960; the two responsive references are 1280×960 and 1024×900. The current palette mixes earlier cream/gold and later blue/slate styling. Do not treat the earlier root DESIGN.md cream/gold frontmatter as the v2 palette.

Key sources:
- design/motaa-desktop.figcraft: canonical editable screen content.
- design/APP_ROUTES.json: exhaustive route and state ownership mapping.
- design/APP_CONSOLIDATION.md: five-workspace application architecture.
- design/SOURCE_TRACEABILITY.md: extracted Android behavior and original screenshot mapping.
- design/MACOS_DESIGN_REVIEW.md: previous visual QA and known static-design limitations.
- <source-research-repo>/MASTER_REVERSE_ENGINEERING.md, with decompiled original Android source at src/motaa/: baseline for original behavior. The iZarTarPro sibling is separate.

These values are a **pre-implementation baseline**, not immutable future node counts. On execution, re-open the master and record SHA, Git status, page/node/action counts and any ongoing user modifications.

## 3. Five application workspaces and preserved behavior

| Real workspace | Current FigCraft entry | Immutable capability coverage |
| --- | --- | --- |
| Charts | 01 Natal chart | Square natal; circular; Tricakka; traditional/Western aspects; Nakshatra; LatLon; natal/transit; Rahu 27/28; original Rasi aspects mode 11; chart zoom and legend |
| Profiles | 31 Profile library | Create/open/edit/duplicate/search, recoverable delete, profile import/backup, invalid data, unknown birth time, independent birth/current geographical values |
| Calculation Setup | 33 Calculation options | Two zodiac systems, nine ayanamshas, eight house systems, True/Mean Rahu, stored-only settings as source-specified, Save-to-profile boundary |
| Time & Analysis | 34 Time navigator | Four independent birth/transit planetary-versus-Ascendant epoch combinations, six time units, period drilldown, range recovery, strengths |
| Advanced Analysis | 11 All tools | All original modes are free. Keep iZarTarPro-advertised entries visible but label any non-verified formula as unavailable/pending validation, never as calculated output |

All 90 application/UI-state pages remain mapped to their owning workspace. Screen-specific pickers, errors, modal sheets and alternative chart layouts are **states**, not 90 separate top-level destinations. The ten algorithm teaching pages remain in their own reference section.

Non-negotiable source distinctions: Burmese syllables and notational strings; independent planets/houses epochs; independent birthplace/current place; source custom Vehlow house mapping; 27 versus 28 Rahu grouping and the 27-name manual reference; separate traditional and Western aspect rules; unresolved MUNP meaning; correct historical time-zone inputs. Preserve all four chart display families. Do not claim numerical parity or functioning Swiss Ephemeris computations because FigCraft depicts a result.

## 4. Semantic foundations and token contract

**Proposed machine-readable values:** TOKENS.json. Tokens are planning inputs. When implementation is separately approved, create matching FigCraft native styles/variables and keep actual generated exports derived from that one master.

### 4.1 Color semantics: light UI

| Token | Color | Use |
| --- | --- | --- |
| brand.accent | #3378F6 | Visual identity and nontext decoration, not 14px body links |
| action.primary / link.default | #1D61D6 | Accessible blue action fill, link text, focus accent |
| surface.window | #F6F7F9 | App background |
| surface.sidebar | #EDEFF3 | Sidebar surface |
| surface.toolbar | #F9FAFC | Unified toolbar |
| surface.canvas | #FFFFFF | Main pane and form fields |
| surface.subtle | #F4F6F9 | Quiet grouped surfaces and alternating tables |
| surface.selected | #DCE8FA | Active navigation |
| text.primary | #202632 | Heading/normal text |
| text.secondary | #687483 | Supporting copy (4.76:1 on white) |
| text.sidebarSecondary | #566273 | Secondary text on sidebar (5.38:1) |
| text.selected | #173E78 | Foreground on selected surface (8.50:1) |
| border.subtle | #D9DEE6 | Decorative separators only |
| border.interactive | #8794A6 | Recognizable required control edge (3.08:1 on white) |
| status.success | #216E48 | Status foreground |
| status.warning | #8A4B0F | Status foreground |
| status.danger | #B42318 | Error/destructive foreground |

Do not use border.subtle as the sole means of identifying interactive controls. Use border.interactive plus label, surface and focus ring. Do not conflate semantic status with traditional chart colors. Keep chart classifications, legends and data palette in a separate namespace.

### 4.2 Typography

- Latin UI: platform system typography where available; **Helvetica Neue** as the supported FigCraft authoring/fallback family. No packaging or distribution of proprietary system font files.
- Burmese UI and chart labels: **Myanmar Sangam MN** with real-glyph shaping tests. Existing **Arial Narrow** only where source-dense chart numeral labels demand it. Never globally replace chart fonts without parity and legibility evidence.
- Design sizes: micro 11, caption/helper 12, body 14, section 16, subheading 20, page title 28, display 32 (design units approximating px/pt on the fixed canvas). Small labels cannot be the only place for critical information.
- Weights regular 400 / medium 500 / semibold 600 / bold 700. Body line height ~1.45, tight label ~1.25. Use tabular figures for numeric tables and legible degree/DMS/UTC notation.

### 4.3 Spacing, shapes, controls

- 4px base grid with steps 4, 8, 12, 16, 20, 24, 32, 40 and 48.
- App content inset 24px on 1440/1280 and 16px on 1024. Avoid accidental one-pixel misalignment or drifting baselines.
- Sidebar item height 36–40px; standard control height 36px, compact 32px, primary 40px. Focusable target at least 24×24 CSS px where applicable; larger hit areas for touch without turning all visible controls into oversized cards.
- Radii 4 / 8 / 12px for compact buttons/fields, groups and panels; one-pixel borders and little or no shadow.
- Focus ring 2px with 2px offset in accessible blue. Hover, pressed, disabled, warning and selected must also have semantic/shape cues. No faux glassmorphism or decorative icons on every label.

### 4.4 Theme architecture

**Light production only.** Future dark appearance may override semantic token definitions later; placeholders are not authorized dark-mode deliverables. The four independent chart display styles remain available even while the UI stays light. Charts keep their own mode-specific colors and legends.

## 5. Lucide icon design contract

Real **Lucide** SVG paths (viewBox 0 0 24 24, nominal 1.75px stroke, fill none, round cap/join, semantically colored foreground). Present at 16px sidebar/inline, 20px toolbar/controls, 24px larger action/inspector. Reconcile optical alignment across those sizes in exported 1× and 2× renders; prevent flattening to PNG or substitute hand-drawn vectors.

Icons must be imported to FigCraft as independently editable native vector layers, named by their actual Lucide identifier. Preserve the ISC license notice when applicable in distribution. Pin a real Lucide release **when implementation begins** and validate every proposed icon against that release. Invalid/missing icon name = failed gate; no close-looking substitutions.

**Curated mapping:**

| Action/feature | Lucide identifier | Size |
| --- | --- | --- |
| Charts | orbit | 16 |
| Profiles | users-round | 16 |
| Calculation Setup | sliders-horizontal | 16 |
| Time & Analysis | calendar-clock | 16 |
| Advanced Analysis | telescope | 16 |
| Birth/current location | map-pin | 20 |
| Zodiac / celestial configuration | moon-star | 20 |
| Chart modes | layers | 16 |
| Save | save | 20 |
| Search | search | 16 |
| Toggle sidebar | panel-left | 20 |
| Toggle inspector | panel-right | 20 |
| Back/forward | chevron-left / chevron-right | 16 |
| Zoom | zoom-in / zoom-out | 20 |
| Export/import | download / upload | 20 |
| Duplicate | copy | 20 |
| Delete / Undo | trash-2 / rotate-ccw | 20 |
| Success / warning / error | circle-check / triangle-alert / circle-alert | 16 |
| Help / education | book-open | 20 |

The canonical machine-readable icon and semantic component mapping is in TOKENS.json. Use icons for navigation, commands and status, not as decorative prefixes to every text row. Icon-only actions require readable labels/tooltips, focus, hover and accessibility. Disable decorative SVG accessibility when it has no interactive meaning.

## 6. Components, states and design patterns

Build a native FigCraft foundation/component library **within the same one master** (after separate execution authorization). Component contracts:

| Component | Required states/behavior |
| --- | --- |
| WindowShell | Unified macOS-style titlebar, controls, focus/inactive window, resize/fullscreen references |
| SidebarNavigationItem | Lucide 16px + label, optional count, selected, hover, focus, disabled, collapsed/expanded |
| ToolbarAction | Icon+label/icon-only with tooltip, three principal toolbar groups max, standard macOS command conventions |
| InspectorPanel | Context help, collapsible 288px panel; never block essential chart information |
| Button | Primary, secondary, tertiary, destructive and icon-only; idle, hover, focus, pressed, disabled, busy |
| TextField | Label/value/units, helper, read-only, required, valid, invalid, focused, disabled |
| CoordinateInput | Decimal and DMS views, hemisphere, input validation, independent birth/current fields and historical offset |
| BirthDateTimeInput | Seconds, UTC offset, historical zone and manual override, unknown time and impossible-date behavior |
| ChoiceControl | Select, segmented/radio/checkbox; keyboard arrows, focus, selection and disabled states |
| SearchFilter | Query, filters, loading, clear state, no matching records |
| ProfileListRow | Select, open, edit, duplicate, import/export, safe deletion and undo |
| DataTable | Numeric/tabular alignment, sort, selection, empty/loading/error, visible horizontal overflow handling |
| ChartContainer | Exact source charts and classification; zoom/pan, legend, inspector, print/export |
| ChartModePicker | All 11 recovered modes available to everyone, including Rasi aspects, with current mode |
| TimeNavigator | Six time steps, four epoch combinations and Dasha range recovery states |
| Overlay | Menu, popover, sheet, dialog; Escape, focus trap/recovery, accessible labeling |
| StatusFeedback | Inline form validation, error/success/warning/undo; distinguish real result vs illustrative reference |
| EmptyLoadingErrorState | Empty profiles, invalid backups, missing data, unverified calculation, and recoverable faults |
| ChartLegend | Traditional symbols as text+shape alongside colors; avoid color-only meaning |

Each component needs documented state variants, label baseline, icon box and spacing, 1×/2× render, focus semantics and one real feature example. Avoid separate "design system screens" becoming 6th or 7th product destinations.

## 7. Layout and responsive behavior

| FigCraft reference | Sidebar | Main content | Inspector |
| --- | --- | --- | --- |
| 1440×960 | 216px expanded | Chart-first / flexible forms | 288px when helpful |
| 1280×960 | 200px expanded | Maintain legible chart/form focus | Optional/collapsible |
| 1024×900 | 64px collapsed/revealable | Protect main work area | Hidden by default; on demand |

These widths are design targets, not existing implementation facts. Screen 88 needs full navigation discoverability via tooltip/keyboard/accessibility while icon-only, not simply clipped labels. If a source wheel cannot fit without distorting its divisions, use zoom/pan rather than modifying mathematical geometry. Form layouts reflow from two columns to one when needed. Toolbars group leading navigation/title, central context and trailing actions, at most three clusters. Critical actions must not rely solely on the bottom edge of a Mac window.

## 8. Design rollout after separate execution authorization

1. Hash and inspect the current sole FigCraft master; snapshot complete page/action/variable inventory; preserve a recovery copy outside the repository.
2. Create semantic style/variable foundations and Lucide native-vector component library **within the same FigCraft**. Validate icon provenance, stroke/size and light-theme contrast.
3. Pilot screens: **01 Natal chart**, **31 Profile library**, **33 Calculation options**, **69 Birth coordinates**, **88 Desktop 1024**. Also review **11 All tools** (direct free mode 11) and a form/error/empty state. Get visual sign-off before mass propagation.
4. Apply verified components to all **90 application/state references** without losing source capabilities. Leave ten teaching pages untouched unless a new educational-design authorization is granted.
5. Generate previews/tokens/gallery from the updated canonical master, never reverse-generate the master from old SVG/Python scripts. Update test baselines only for independently verified intentional changes.
6. Pass all gates in REVIEW_GATES.md; deliver the native file plus 100 rendered previews, annotated before/after samples, evidence matrix and defect report.
7. Stop at design delivery. No application code, Git commit/push, host/deploy or dark-mode implementation under this approval.

## 9. Known boundaries

Not resolved by a design-only approval: original APK numerical parity, Swiss Ephemeris runtime integration, complete MUNP algorithm, advertised-only advanced math correctness, live responsive behavior, actual macOS keyboard/VoiceOver APIs or future dark-mode colors. Document and test these during their appropriate implementation work; do not invent output or treat them as paid-only features.

**Readiness conclusion:** all decisions are internally consistent and the design system is ready for **a separate explicit FigCraft execution authorization**. This specification is itself the deliverable for the present request.

## 10. Full decision record: all 42 IDs

All recommendations are approved as option A **except Q009=B**. There is no unresolved user choice, and no later question is needed for this design-level specification.

| ID | Approved | Binding design decision |
| --- | --- | --- |
| Q001 | A | Native Precision visual direction |
| Q002 | A | Professional astrology instrument |
| Q003 | A | Prioritize macOS conventions |
| Q004 | A | Precise, calm and enduring personality |
| Q005 | A | Preview + System Settings + Numbers references |
| Q006 | A | Let chart geometry and typography establish identity |
| Q007 | A | System Blue / Slate primary palette |
| Q008 | A | Neutral, subtly layered surfaces |
| Q009 | **B** | **Light first, dark later** |
| Q010 | A | Semantic status colors (success, warning, error, info) |
| Q011 | A | Preserve existing chart colors as data meaning |
| Q012 | A | Retain all four chart display themes |
| Q013 | A | Lucide outline at 1.75px nominal stroke |
| Q014 | A | Real editable Lucide SVG paths |
| Q015 | A | One consistent icon stroke family |
| Q016 | A | 16 / 20 / 24px icon scale |
| Q017 | A | Functional icon placement, no decoration overload |
| Q018 | A | Curated naming and verified icon alignment |
| Q019 | A | Semantic foundations → components → patterns → screens |
| Q020 | A | Platform typography, Helvetica Neue fallback |
| Q021 | A | Myanmar Sangam MN; validate Burmese glyph rendering |
| Q022 | A | Balanced professional interface density |
| Q023 | A | 4px base spacing grid |
| Q024 | A | Subtle borders and 8–12px panel radii |
| Q025 | A | Five real workspaces with feature/state drilldown |
| Q026 | A | Collapsible icon + label sidebar |
| Q027 | A | Unified macOS-style toolbar |
| Q028 | A | Contextual collapsible inspector |
| Q029 | A | Chart-first arrangement, exact source semantics |
| Q030 | A | Native-style grouped forms and explicit validation |
| Q031 | A | Default/hover/focus/pressed/disabled/loading/error states |
| Q032 | A | Dense aligned numeric data with stable units/format |
| Q033 | A | Keyboard-first Mac navigation and focus conventions |
| Q034 | A | WCAG 2.2 AA targets and documented chart exceptions |
| Q035 | A | Inline validation, confirmation and recoverable errors |
| Q036 | A | 120–180ms subtle transitions and reduced-motion support |
| Q037 | A | 1440 / 1280 / 1024 review canvases, min-size rules |
| Q038 | A | Foundations → pilots → verified 90-screen rollout |
| Q039 | A | No original feature lost, premium features included |
| Q040 | A | Exactly one native FigCraft source file |
| Q041 | A | 100-page visual regression, icon, contrast and navigation QA |
| Q042 | A | FigCraft + previews + docs only; no app code/commit/deploy |

## 11. Reference standards and attribution

Planning references (current as of 2026-11-17):
- Apple Human Interface Guidelines: Designing for macOS: https://developer.apple.com/design/human-interface-guidelines/designing-for-macos
- Apple Human Interface Guidelines: Sidebars: https://developer.apple.com/design/human-interface-guidelines/sidebars
- Apple Human Interface Guidelines: Toolbars: https://developer.apple.com/design/human-interface-guidelines/toolbars
- Lucide upstream icon list: https://lucide.dev/icons
- Lucide SVG stroke width guidance: https://lucide.dev/guide/lucide/basics/stroke-width
- Lucide ISC license: https://lucide.dev/license
- W3C WCAG 2.2 normative criteria: https://www.w3.org/TR/WCAG22/

The planned 1.75 stroke is an intentional override of Lucide's documented default 2px; all imported components must follow one chosen rendering rule. Apple/HIG and Lucide conventions guide the new UI; they are not evidence that the static FigCraft prototype has working native controls.

## 12. Final architecture-readiness checklist

- [x] All Q001–Q042 answers explicitly recorded; Q009=B takes precedence.
- [x] Full light-first semantic palette with an accessible distinction between brand blue and readable action blue.
- [x] Chart data palette explicitly isolated from interface branding.
- [x] Real Lucide SVG import strategy, icon names, sizes, stroke and licensing defined.
- [x] Typography and Burmese font rules documented.
- [x] Five workspace boundaries and 90+10 reference ownership preserved.
- [x] Component inventories, states, layout metrics, responsive breakpoints and interaction contracts specified.
- [x] Pilot-stage approval and complete 100-page review gates specified.
- [x] All features available for all users without premium gate.
- [x] One-master and no-implementation boundaries preserved.
- [ ] Future explicit authorization to edit FigCraft: **NOT GIVEN**.
- [ ] Future approval of light-theme pilot renders: **NOT GIVEN**.
- [ ] Actual app implementation or dark-mode authorization: **NOT GIVEN**.


## Execution authorization and outcome (2026-11-17)

The user explicitly authorized FigCraft execution in a subsequent turn. Native Precision v2 was applied to the sole master, with the 100-page render and 17/17 automated regression checks passing. The source-backed Lucide icon library, 19 native reusable component families, light UI tokens, 90 application screens and compact reference were implemented. See design/NATIVE_PRECISION_V2_REVIEW.md for exact source/hash/QA evidence and review boundaries. The original planning-era statements requiring future FigCraft authorization are historical and superseded by this explicit follow-up. **User visual signoff is pending; application code, commit, push, deployment and dark mode remain unauthorized.**
