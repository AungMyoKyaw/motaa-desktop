# Native Precision v2: Visual review and acceptance gates

**Status:** Planned verification protocol, not executed design QA.  
**Decision scope:** Q001–Q042 approved, except Q009=B (light first; dark later).  
**Baseline:** 100 native FigCraft pages, 90 application/state refs, 10 lessons, five application workspaces, 1,297 native navigation actions, 35 native variable actions, current 15-test regression suite. Counts must be refreshed before a future redesign.  
**Reference:** SPEC.md and TOKENS.json in this folder; canonical design is design/motaa-desktop.figcraft.

## Gate policy

**Each gate is binary: PASS or FAIL, with evidence.** An unexecuted, incomplete, manually deferred or unverified check is **NOT RUN** and is not represented as a PASS. A mandatory FAIL blocks promotion of a staged FigCraft candidate. No test may be weakened, deleted or relabeled simply to make a candidate pass. All staging and backups stay outside the working repository.

### Preparation: mandatory before any future FigCraft execution

| Gate | Pass condition | Required evidence |
| --- | --- | --- |
| G00: Explicit authority | User separately authorizes editing FigCraft; this document approval alone is insufficient | User execution instruction |
| G01: Baseline integrity | Snapshot current SHA-256, Git status, page/node counts, action counts, exports and known issues; canonical file unchanged during inspection | Baseline manifest, preserved pre-edit master outside repo |
| G02: Source requirements | Review APP_ROUTES, APP_CONSOLIDATION, SOURCE_TRACEABILITY and Android reverse-engineered source; original behavior distinguished from advertised-only capabilities | Per-feature source-to-screen matrix, unresolved-proof register |
| G03: Single-master boundary | Exactly one *.figcraft under project; candidates/backups outside repository; no application code/commit/deploy | File search/count and repository status |

### Foundations and native vector library

| Gate | Pass condition | Evidence |
| --- | --- | --- |
| G04: Semantic palette | All application chrome references named semantic tokens, not scattered arbitrary colors; original chart data colors remain independently named | Extracted token inventory, sample swatch board and color usage report |
| G05: Contrast | Normal text **≥ 4.5:1**, large text **≥ 3:1**, meaningful nontext/control boundaries **≥ 3:1**. Brand #3378F6 not used for normal body text on white; readable action #1D61D6 used instead. No invisible controls disguised by #D9DEE6 alone | Machine-generated contrast table plus visual inspection of selected/hover/disabled/focus/status |
| G06: Lucide provenance | Every icon is the matching exact path from a real pinned Lucide release; correct name, version, 24×24 viewBox, 1.75px stroke, consistent optical alignment and license provenance. **100% icon coverage; 0 hand-drawn/raster/icon-font substitutions** | Icon-ID to source URL/version/hash map, vector node inspection and size comparison |
| G07: Icon sizes/behavior | 16 / 20 / 24px scale, correct semantic action and active/disabled states, icon-only label/tooltip and keyboard target; no decorative flood | 1×/2× icon atlas, accessibility label matrix, visual spot check of five-workspace navigation |
| G08: Typography | Latin system/Helvetica Neue and Myanmar Sangam MN rules observed; chart-specific Arial Narrow retained only where needed; **0 missing fonts, 0 visibly clipped/wrong Burmese glyphs, 0 truncated critical units** | Font inventory, multilingual text samples, full-resolution chart/form crops |
| G09: Component state coverage | All component families from TOKENS.json represented by native reusable FigCraft styles/components with correct default/hover/selected/focus/disabled/error/loading and relevant variants | Named component inventory and rendered state sheet |
| G10: Geometry consistency | 4px grid, consistent label baselines, 4/8/12 radii, stable 32/36/40 control heights, non-overlapping 2px focus ring, no misaligned Lucide glyphs | Geometry/spacing audit and annotated component snapshots |

### Pilot review: must pass before all 90 product states change

| Gate | Pass condition | Evidence |
| --- | --- | --- |
| G11: Pilot screens | Native candidate covers **01 Natal chart, 31 Profile library, 33 Calculation options, 69 Birth coordinates and 88 Desktop 1024** plus **11 All tools** and at least one real empty/error/validation state | Full-resolution before/after at each pilot, workflow notes |
| G12: Product feel | No design-spec headings, workbook banners, prototype gold badges, deceptive fake input controls or generic dashboard substitution in product screens. Chart context remains dominant | Human visual review against macOS Preview/System Settings conventions |
| G13: Explicit pilot approval | User reviews the selected light palette, Lucide set, main chart, dense settings, birth-location form and compact sidebar; **approves pilot in a subsequent message before propagation** | Written pilot approval plus approved screenshot IDs |
| G14: Light/dark scope | Light-only UI complete; dark not auto-invoked or represented as a finished product. Four independent chart themes preserved | Appearance-state inventory and chart-theme comparison sheet |

### Complete product coverage and source fidelity

| Gate | Pass condition | Evidence |
| --- | --- | --- |
| G15: Feature ownership | Every original feature/state reference remains mapped uniquely to one of the **five product workspaces**; all **90 application/state references** accounted for | APP_ROUTES mapping validation, route graph, actual FigCraft frame names |
| G16: Chart invariants | Original square/circular, Tricakka, Nakshatra, LatLon, Rahu, traditional/Western aspects, epoch combinations and Rasi mode 11 retain geometry/semantic labels; chart data palette not globally recolored | Rendered before/after overlays, source class/mode mapping, chart legend inspection |
| G17: Free-all policy | All eleven recovered modes directly navigable, Rasi aspects included; no functional premium, license, activation, password, member-gate or device-ID flow introduced | All-tools view and prototype graph/entitlement audit |
| G18: Advanced evidence honesty | Advertised-only iZarTarPro entries remain represented as available tool destinations or transparent pending-calculation states; no fabricated numerical results | Source-evidence register and UI string audit |
| G19: Birth/profile correctness | Separate birthplace/current location fields; DMS, decimal, hemispheres, UTC offset and historical/manual controls represented; unknown time, invalid date and unsaved-change states distinct | Form-state matrix, field screenshots at 1440/1280/1024 |
| G20: Time/calculation variants | Four natal/transit combinations, six time units, Dasha out-of-range recovery, nine ayanamshas, eight house systems and node choices visually reachable | FigCraft navigation/state coverage matrix |

### Visual, responsive and accessibility acceptance

| Gate | Pass condition | Evidence |
| --- | --- | --- |
| G21: Desktop canvases | 1440×960, 1280×960 and 1024×900 show no clipped primary action, overlapping label, uncontrolled scroll, truncated coordinate value or off-canvas control | Full-page renders and inspected crops for each targeted canvas |
| G22: Sidebar / toolbar / inspector | Five consistent workspaces, selected state, collapse/reveal, contextual inspector behavior; toolbar ≤3 principal groups. 1024 layout preserves discovery of all destinations without a truncated sixth row | Matched screenshot triptychs, route nav checks |
| G23: Component focus/accessibility | Design targets WCAG 2.2 AA; contrast per G05, focus visible and unobscured, control target size ≥24×24 where applicable, no color-only status, labels for icon-only actions | Focus/state screenshots, keyboard interaction design checklist, contrast report |
| G24: Responsive data clarity | Dense chart notation keeps readable zoom/detail alternative; numeric tables align columns and do not truncate degrees/DMS/UTC; Burmese remains correctly shaped | 1×/2× exports, enlarged source chart crops and table inspection |
| G25: Interaction prototype integrity | Every FigCraft navigation action points to existing target; no inert-looking UI action, dead-end mandatory flow or accidental jump into lessons/old source gates; variable actions intact unless deliberately redesigned | Full graph traversal, orphan-link report and interaction comparison |
| G26: Motion and reduced motion | Only 120–180ms subtle transitions planned; reduced-motion state has no essential information hidden in animation | Component spec and reduced-motion review frames |
| G27: Text and errors | Zero visible text-box collision, cropped line, truncated critical field, overlapping icon/control, missing error/helper or mislabeled action; recovery and destructive actions explicit | Automated geometry output plus page-by-page visual inspection; avoid relying solely on numeric bounds |
| G28: User-facing language | English UI labels, traditional Burmese chart notations and English digits where source-supported; no raw source-code paths in UI, no fake computed/verified claim | String scan + native render check |

### Full-master export and release-ready **design** handoff

| Gate | Pass condition | Evidence |
| --- | --- | --- |
| G29: Complete render | **100/100 original page references render successfully**, plus any intentionally added foundation/component pages; 0 failed render, corrupted page or missing generated preview | Export manifest with page/image names, sizes and checksums |
| G30: Visual regression | Inspect each of the 90 app/state screens and ten reference lessons; 0 unresolved known critical/major visual defects. Original astronomy teaching diagrams intact without separate approval | 100-page contact sheet, representative high-res crops, side-by-side diffs, defect ledger |
| G31: Automated regressions | Existing full design regression suite passes; new icon, color, state-ownership and responsive checks pass. Intentional page/node/action changes documented and tested, not waved through | Test output, diff summary, baseline migration notes |
| G32: Native artifact validity | Canonical FigCraft reopens in FigCraft CLI, native nodes/images/variables/actions remain editable; no font missing and no second FigCraft master | CLI validation, archive integrity check, file count and test read |
| G33: Generator consistency | Previews, gallery, variables and exported tokens derived from canonical master; all 100 page image names consistent with manifest | Regenerated export hashes, index walkthrough check |
| G34: Design-only delivery | Deliver native FigCraft, generated previews, component/token inventory, contrast/icon audits, and change log; **no product app code, commit, push or deployment** | Final file inventory and Git status |
| G35: Final review | User can compare accepted pilot and full rollout; any residual limitations explicitly listed; do not claim perfect macOS native behavior from static prototypes | Final review summary and user decision |

## Minimum pilot sign-off sheet

Every pilot screenshot must be reviewed at native size and have all fields below:

- [ ] Screen ID, FigCraft page name, exported PNG, candidate SHA and baseline reference.
- [ ] Correct approved light palette, readable blue actions and complete contrast sample.
- [ ] Actual Lucide icons, 1.75px stroke, consistent optical alignment at icon scale.
- [ ] Native-looking but **meaningful** controls: a value must look editable only when the future component is truly editable.
- [ ] Titlebar/sidebar/inspector hierarchy and consistent baseline grid.
- [ ] No chart palette damage, computation semantics loss or fabricated data.
- [ ] Valid target/response for all navigational and form-related prototype interactions.
- [ ] All hover/selection/focus/disabled/error variants actually specified, not guessed.
- [ ] No text clipping or symbol corruption (including Burmese, seconds, DMS, degrees, UTC offsets).
- [ ] Owner approval recorded before further propagation.

## Accessibility and control-specific notes

- WCAG 2.2 AA targets apply to the future app implementation; static FigCraft has **design evidence only**, not proven VoiceOver support or keyboard focus.
- Focus indicators must remain visible against all approved surfaces. Status conveys meaning with text/icon/shape as well as color.
- Original source plot marks that cannot satisfy UI-sized letter requirements require zoom, legends or tabular equivalents. Do not change their astrology geometry merely to satisfy a blanket font rule.
- Visual status text must describe whether a displayed value is **source-derived**, **illustrative**, or **not currently calculated**; do not substitute "premium-locked" for unfinished math.
- Document contrast exceptions with location/screenshot/justification and accessible alternative; critical unresolved exception blocks approval.
- Minimize disruptive modal dialogs; support Escape, standard keyboard conventions, focus recovery and undo for destructive actions in the actual runtime.

## Review sequencing and stop rules

**Stage A: Baseline and foundations:** G00–G10. No mass edits before they pass.  
**Stage B: Pilot:** G11–G14, plus pilot-appropriate G21–G28. Stop for explicit pilot approval.  
**Stage C: 90-screen design rollout:** G15–G28, only after approved pilot.  
**Stage D: Final FigCraft QA:** G29–G35. Stop at design delivery.

At **any FAIL**: keep the existing canonical master; diagnose in a project-external candidate, correct, re-render, re-test. At **NOT RUN**: do not state the gate passed. On source conflict: use the original Android evidence and user-approved behavior, not a convenient design assumption. On unverified advanced algorithms: label honestly, retain the feature entry, and do not invent numerical results.

### Current verification status (2026-11-17)

- Design decisions: **APPROVED**, Q001–Q042 except Q009=B.
- Implementation-ready specification: **WRITTEN**.
- TOKENS.json manifest: **PROPOSED**, not in FigCraft.
- All execution-stage gates G00–G35: **NOT RUN** under this approval; G00 specifically awaiting separate authorization.
- No design implementation, application code, commit, push or deployment authorized.



## Implementation progress update (2026-11-17)

The user separately authorized FigCraft execution after this initial gate plan. The one native master was updated, and automated source/vector integrity, 100/100 renders, chart preservation, navigation validity, layout geometry and 17/17 regression tests passed. See design/NATIVE_PRECISION_V2_REVIEW.md for current evidence. **G13 pilot human approval and G35 final user visual signoff remain pending**, as do live runtime accessibility, real resize/motion and numerical parity checks. The older Current verification status subsection is a historical pre-execution snapshot, not a claim that execution has not occurred.
