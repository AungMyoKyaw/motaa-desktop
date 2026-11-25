# FigCraft visual QA — 16 November 2026

## Scope and method

Reviewed the single authoritative FigCraft master using FigCraft CLI 0.15.0. Checked all 100 native pages for text collisions, out-of-frame elements, text occluded by later opaque rectangles, and valid prototype destinations. Rendered and compared all 39 affected screens against their previous previews, then regenerated all 100 PNG previews and the gallery.

## Confirmed defects and corrections

| Finding | Count | Correction |
| --- | ---: | --- |
| Birthplace labels obscured by a later opaque description background | 33 screens | Moved the labels into the right of the caption row and brought the native text layers to the front |
| Three theme captions covered by a later background | 3 screens | Raised text above the background and aligned both caption lines |
| Horizontal separator lines extending beyond the frame | 9 nodes | Clamped line endpoints to the existing frame edge |

The birthplace labels could not be repaired by moving them alone. Native FigCraft layer reordering was necessary and its visible pixel difference was verified.

## Final acceptance

- 100 pages, 18,033 nodes; 1,296 native navigations and 35 native variable selections preserved
- Exactly one FigCraft document in the repository
- No text bounding overlaps, out-of-frame elements, or entirely covered text (excluding intentional interaction hit areas)
- All 39 affected screens rendered, 33 recovered location labels visibly restored, and 3 theme captions visibly corrected
- All 100 preview PNGs regenerated from the canonical file
- All 12 design regression tests passed, including the new layout quality test

## Boundaries

This is a static/visual design QA, not a running calculation engine or verification of numerical parity with the original MOTAA APK. Keyboard and screen-reader behavior also remain outside this audit. Existing original chart geometry and astronomical illustrative values are preserved.

Recovery: ~/.local/share/motaa-desktop-backups/figcraft-qa-20261116-233654/

No commit, push, or application code implementation was performed.


## Open-access edition design audit — 17 November 2026

Re-read the original Android mode-list declarations in main.java (f3306o and f3307p), its T0 == 11 renderer, and the paidversion.java advertising component. The new desktop edition deliberately **does not reproduce the original membership gate**. Every original chart mode and every designed advanced feature entry is accessible for all users, with mode 11 Rasi aspects linked directly from All tools.

- Renamed/reworked five former member/activation frames (59–63) into included-feature, no-account, and offline-access states; page 64 remains original Rasi aspects.
- Updated 32 shared navigation rails from Membership to All included.
- Added a source-style native editable Rasi aspects catalog entry on page 11, including one direct prototype link to page 64.
- Checked all 100 frames: **0 overlapping text bounding boxes, 0 out-of-frame shapes, 0 text-wrap overflow in modified labels and 0 broken prototype targets**.
- Rendered and visually inspected 10 affected/reference pages and regenerated the full set of 100 source-generated PNGs.
- Preserved five primary product flows, 90 feature/state references, ten astronomy lessons and the original chart geometry.
- Confirmed **14 regression tests passed** and exactly one canonical FigCraft file.

The original membership evidence is preserved externally in the reverse-engineered repository and an external baseline FigCraft backup. The new release policy changes **UI access only**, not computation or numerical parity. The original astronomical/advanced algorithms remain subject to independent implementation and verification.
