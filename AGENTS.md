# MOTAA Desktop — Local Agent Instructions

## Authoritative design

- The **only** FigCraft master is `design/motaa-desktop.figcraft`. Preserve its editable native pages, variables, images, interactions, and canvas.
- Do not introduce another `*.figcraft` anywhere in this repository, including under `design/staging/`. The validation tool rejects duplicates.
- Existing SVG artwork in `design/source` is historical output, not an authoring input. Never run or reconstruct the retired Python/SVG-to-FigCraft generation pipeline.
- For edits, make candidate files outside this repository, keep a recovery backup, use the verified FigCraft CLI, visually/structurally inspect, and promote the result to the canonical path only after verification.
- The current master has 111 pages, 2,456 native navigation actions and 71 native variable selections, 31 shared styles and 134 native variables; the original 100 pages remain preserved. Native Precision v2 adds 35 genuine Lucide components and 19 native component families. If intentionally changing these totals, update the validation tests and verification record based on actual evidence.
- Regenerate PNGs, CSS and the HTML walkthrough **from the master**, using the commands in `design/README.md`.
- Do not edit implementation or product specifications just to accommodate an export. The master controls design decisions.

## Gold Heritage v3 / Native Precision v2 / Lucide

- The current approved shell is **Modern Heritage Gold, light UI only**, with bronze action #7C5B21, antique-gold brand #8B6B2E, warm ivory #F8F5EE, and a separate four-mode original chart palette. Native Precision v2 remains historical provenance; do not silently implement a dark theme.
- Use actual Lucide v0.468.0 source geometry as editable native components (not decorative unicode glyphs or icon-font approximations). Preserve the original 450 linked icon instances plus 20 in newly duplicated native shells, and their prototype destinations.
- 19 native component sets with four states each live in the same single master on Design Foundations; the component definitions may be hidden from the visible canvas but must remain editable in Assets.
- The current numerical and visual audit is documented in design/NATIVE_PRECISION_V2_REVIEW.md; maintain the actual 100-page render and source-chart preservation gates.

## macOS visual language

- The canonical FigCraft design now uses cohesive macOS-style chrome, a compact five-workspace sidebar, clear active state, contextual inspector, and native-looking input hierarchy. Do not restore engineering banners or workbook language to visible product screens.
- Read design/NATIVE_PRECISION_V2_REVIEW.md (current) and design/MACOS_DESIGN_REVIEW.md (historical), and inspect the native master before changing UI. Source chart geometry, calculations, and all existing features remain the baseline.
- Pages 91–100 are educational references, not app destinations; keep those separate.
- The new design is a static native-style specification, not proof of executable AppKit controls or numerical parity.

## Application-level consolidation

- The native design has **five real product entry flows**, not 100 separate application routes. Their existing start frames are 01 Natal chart, 31 Profile library, 33 Calculation options, 34 Time navigator and 11 All tools.
- Other screens remain source-state, reference-output and algorithm teaching fixtures. Keep all features and original chart geometry; never swap in a generic dashboard, an unsourced result, or a flattened fake image.
- Read design/APP_CONSOLIDATION.md and design/APP_ROUTES.json before adding a product page. All 90 feature/state references belong to exactly one route; 10 lessons are education-only.
- All original chart modes are available to everyone in this desktop edition, including original mode 11 (Rasi aspects). The original Android membership branch is historical evidence only. No account, password, device ID, payment or activation in the new design; advertised ZarTarPro formulas still require verification.
- Do not silently delete or merge original FigCraft frames. A state is merged into a route's architecture, but its independent regression reference remains available.

## Verification

```sh
python3 design/source/figcraft_master.py
python3 design/source/export_previews.py
python3 design/source/export_tokens.py
python3 design/source/build_gallery.py
python3 -m unittest discover -s design/tests -v
```

The previous variants and generators are preserved as external recovery archives in `~/.local/share/motaa-desktop-backups/`. Do not restore them into the working tree as new masters.

## Source parity 2026-11-20
- Current: 110 pages, 26,601 nodes, 2,431 navigations, 67 variable selections, 31 styles and 133 variables.
- The 2026-11-20 correction pass preserves the 104-page structure while correcting Rahu radii, source palettes, Moon fixture signs/nakshatra, original Astro font mappings, epoch-state links, storage provenance and desktop-extension labels. See design/SOURCE_PARITY_20261120_REVIEW.md.
- The complete 534-town Android location directory is in design/source/locations.v31.1.json; its original Astro font is packaged in design/fonts/Astro.ttf.
- Native pages 101–110 stay within the five existing routes: Time (101/103), Calculations (102/110), Advanced (104/107/109), Profiles (105/106), and Charts (108). The ten astronomy lessons remain pages 91–100.
- All eleven original modes and all advanced/sibling-inspired workflows are enabled free for every user. Pages 104 and 107 provide 2D/3D illustrative sky fixtures; no live ephemeris or numerical/runtime parity is implied.
- See design/SOURCE_PARITY_20261117_REVIEW.md and design/previews for audit evidence.

## Native option alignment regression guard

One-line native option labels must be vertically centered within 2 px; full-width access-navigation labels are also horizontally centered. Preserve the two-line form fields on pages 69/70 and multi-text design specimens on page 86. Run test_native_option_rows_centered_without_flattening_fields on every change and verify the page 61 before/after review asset. Do not repeat developer-only membership/prototype copy or competing exit controls.

## UX parity and free advanced workflows · 2026-11-21
- Pages 1–110 retain their order. Page 111 adds the all-strengths state of the recovered Rasi aspects mode and links back to the filtered state on page 64.
- Advanced and research-derived entries are never hidden or paywalled. Unverified results stay honestly labeled; no fake calculations or parity claims.
- `MOTAA / Primitives` feeds preserved Gold Heritage semantic aliases and component-level tokens. Chart-only palettes remain separate. See design/UX_PARITY_20261121_REVIEW.md for final evidence.
