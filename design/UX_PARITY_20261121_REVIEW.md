# MOTAA UX parity and free advanced workspace review

Date: 21 November 2026  
Status: Promoted to the single canonical FigCraft master and verified  
Tool: FigCraft CLI 0.15.0  
Master: `design/motaa-desktop.figcraft`

## Product decisions

- Every advanced and research-derived workspace is visible, enabled and free.
- There is no research paywall, membership gate, hidden advanced destination or fake calculation path.
- Illustrative positions and proposed calculations remain honestly labeled. The design does not present them as verified runtime output.
- The five application workspaces, original chart geometry, original page sequence and ten educational lessons remain intact.
- Application source, the reverse-engineered source repository, Git history and deployment were not changed.

## Delivered UX improvements

| Area | Result |
| --- | --- |
| All tools | Searchable, categorized directory with explicit free access and direct chart-mode entries |
| Chart context | Natal and Transit context is visible; page 106 provides independent profile assignment |
| First launch | Page 105 offers an anonymous local sample, birth-profile creation and backup import without signup |
| Location editing | The 534-town offline directory, custom coordinates and coordinate controls have clear, non-overlapping labels |
| Sky workspaces | Pages 104 and 107 link 2D and 3D views with layer controls, inspection and time context |
| Compact layout | Page 108 provides a native 1280×800 chart with an 80% uniformly scaled source chart, clear mode controls, visible inspector notes and correctly mapped navigation hotspots |
| Recovery | Page 109 preserves inputs and drafts while offering Retry, Calculation options and All tools |
| Design-system QA | Page 110 exposes default, hover, pressed, focus, disabled, loading, error, success and selection contracts |

Pages 105–110 were appended after the preserved original 104 pages. Superseded duplicated content on pages 106, 109 and 110 is hidden only on those new compositions; the original source pages remain editable and unchanged in sequence.

## Design system

The native token architecture now runs from MOTAA primitives through Gold Heritage semantic aliases to component contracts. The master contains 133 variables, including 48 Gold Heritage UI colors, plus 31 shared styles. Light, Pink, Blue and Monochrome remain independent chart-display themes; the desktop chrome remains Gold Heritage light UI.

The component audit distinguishes source assets from duplicated native reference assets: 35 unique Lucide sources across 70 native components, and 19 unique families across 38 component sets. The five unique application flows have seven native start points because pages 106 and 108 retain additional entries copied with their source pages.

## Verification evidence

- 110 pages and 26,601 native nodes.
- 2,431 valid native navigation actions and 67 native variable selections.
- Every page is reachable through native prototype navigation.
- 110/110 PNG previews regenerated from the canonical master at native dimensions: 107 at 1440×960, one at 1280×960, one at 1024×900 and one at 1280×800.
- Manifest, `tokens.css` and HTML gallery regenerated from the same master.
- 28/28 external-candidate preflight tests passed before promotion.
- 35/35 complete regression tests passed after promotion and export.
- Exactly one `.figcraft` exists in the repository; no fonts are missing.
- Nested/effective-visibility bounds, nested text-collision and opaque-cover checks report zero findings across all 110 pages.
- All 110 pages were freshly rendered and visually reviewed in six final contact sheets; pages 02, 10, 44, 89, 97 and 108 also received full-size repair review.
- Review evidence: `design/reviews/layout-review-20261121/final-contact-01.jpg` through `final-contact-06.jpg`.
- Pages 47, 48, 52, 53, 87 and 88 subsequently received a separate full-size state/composition review after targeted edits.
- State/composition evidence: `design/reviews/state-layout-20261121/`; the deep-audit JSON contains no findings.
- The final component-state pass freshly rendered and reviewed all 110 pages in six new contact sheets. Pages 47, 48, 52, 53, 55, 71, 86, 90, 109 and 110 also received full-size related-state review.
- Component-state evidence: `design/reviews/component-states-20261121/`; before/after PNGs, all-page contact sheets, CLI results, preflight comparison and the zero-finding deep audit are retained.
- `npx @google/design.md lint DESIGN.md`: zero errors and 13 non-blocking orphaned-token warnings. Those warnings concern tokens not referenced by the four YAML component blocks; the native design and prose still use the tokens.

Final SHA-256: `7cc8edff0ab7e23dc7a50bbf4d75b2a1ff99fdef3618a17582e75b1bfb8ab15c`  
Baseline SHA-256: `1bb0d3ba3274b6f5b00c5a0db9eb2b4952d33640aaf11e5ea1e8d5de85331b99`  
Layout-review baseline SHA-256: `047e33998d8dc1d17cf464c41c16e755bc2f627dad76ba34af5dc96f38c3c34b`  
State-layout baseline SHA-256: `d3966a34e91e3e74adacaea339a37b52d1262adaa4c79f6f1ffc1bb53f61b708`  
Component-state baseline SHA-256: `f05fbfa277ce8dac7032f08b30305b323061b6b1267af9670a3994e616551b7c`  
Recovery master: `~/.local/share/motaa-desktop-backups/ux-parity-20261120/pre-ux-parity.figcraft`  
Layout-review recovery master: `~/.local/share/motaa-desktop-backups/layout-review-20261121/pre-layout-review.figcraft`  
State-layout recovery master: `~/.local/share/motaa-desktop-backups/state-layout-20261121/pre-state-layout.figcraft`  
Component-state recovery master: `~/.local/share/motaa-desktop-backups/component-states-20261121/pre-component-states.figcraft`  

## Full layout collision review

- Moved the education links on pages 02, 10 and 89 into free sidebar space so they no longer cover Settings or the local-product footer.
- Moved the page 44 Moon callout away from the Burmese wheel glyph it covered.
- Separated the page 97 equals operator from the result card.
- Corrected page 108's inherited 960 px divider to 800 px, moved the chart below the mode buttons, uniformly scaled the duplicated chart from 84% to 80%, and moved all three chart notes into the visible inspector area.
- Changed exactly 47 existing native nodes through FigCraft CLI. No pages, nodes, navigation actions, variables, fonts or component definitions were added or removed.
- The original chart pages and original 104-page sequence remain intact; only the duplicated compact chart was rescaled.

## State and composition review

- Page 48 now uses post-delete copy, marks Mira as recently deleted, and places Undo deletion and Return to profiles in one padded action row with aligned native hotspots.
- Pages 87 and 88 now have labeled three-region window previews, balanced 2×2 behavior grids, 14 px option-body text and concrete breakpoint contracts. Decorative markers and the orphaned fifth cards were removed from the visible compositions.
- Breakpoint copy explicitly identifies a static specification rather than claiming verified responsive runtime behavior.
- Pages 47, 52 and 53 no longer expose prototype/export annotations as orphaned product UI. Page 47 also clarifies the local-only deletion scope.
- Changed exactly 96 existing native nodes through 118 successful FigCraft CLI steps. No pages, nodes, navigation actions, variables or component definitions were added or removed.
- Two semantic regression tests now pin deletion-state behavior, action/hotspot alignment, hidden annotations, balanced breakpoint grids and honest static-reference copy. The established option-row fixture count is 136 because each breakpoint page intentionally has four behavior cards plus a preview.

## Component-state grid and final all-screen sweep

- Page 110 now uses one two-column grid for all four panels: 571 px columns, a 24 px gutter and matching 24 px inner padding. The mixed 552/582 px top columns and 716/418 px bottom columns are gone.
- All six primary-button specimens use equal widths and consistent gaps. Validation fields share the same geometry, including a complete Success label, input, vertically centered value and aligned helper message.
- Feedback rows share one marker/title/copy grid. Default, Hover, Selected and Disabled navigation states now use the same visible token-based row background, border, radius and label alignment.
- Changed 57 existing native node records through 63 successful FigCraft CLI steps. Three reused Success-field nodes were moved above the opaque panel stack; no pages, nodes, variables, styles, navigation actions or component definitions were added or removed.
- The complete 110-page nested bounds/text-collision/opaque-cover audit reports zero findings. All 110 previews, the screen manifest, tokens and HTML gallery were regenerated from the promoted master.
- A new semantic regression pins the shared panel grid, equal button widths, complete Success field, feedback geometry and visible selection-row contracts; 35/35 tests pass after final export.

## Honest limitations

This is an editable native design and prototype specification, not an implemented desktop runtime. It does not prove live ephemeris calculations, numerical parity with the Android application, persistence, keyboard/VoiceOver behavior, reduced motion or real macOS resizing. The 2D/3D sky fixtures and other illustrative values must not be represented as verified astronomical output. Those implementation tasks remain separate; no fake result was introduced to conceal them.
