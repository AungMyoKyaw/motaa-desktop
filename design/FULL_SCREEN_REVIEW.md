# MOTAA full-screen design review

21 November 2026 · FigCraft CLI 0.15.0

Reviewed every incoming screen (01–110) on 11 contact sheets. Cross-checked the native document for bounds, text overlap and obscuration, page order, component definitions, valid prototype destinations, reachability and hotspots with identical bounds but conflicting destinations. Regenerated all 111 screens at their native dimensions and compared the pixels against the previous renders. Inspected the 17 new or changed renders and checked the dense Mode 11 matrices and Western glyph key at larger scale. The other 94 renders are pixel-identical.

| Pages | Changes |
| --- | --- |
| 06, 09 | Replaced the stray TechnoHalo sample place with the existing Yangon transit fixture and clarified the action that opens the two location editors. |
| 64, 111 | Replaced the mode 11 placeholder with an editable Rasi chart and native 9×10 planet/Ascendant and 9×12 house matrices. The prototype switches between **Aspected only** and **All strengths** and keeps aspect markers visible. Each state links to All tools and Western aspects. |
| 65–68, 74–77, 83–85 | Removed secondary selection fills so the sidebar identifies a single workspace. |
| 84 | Uncovered the Western symbol key, corrected Jupiter/Saturn and expanded it to all 12 symbols used by the Western matrix. |
| 104, 107 | Clarified the layer labels and removed redundant abbreviated legend text. |

## Mode 11 evidence

The Android `S0()` branch exposes `g1`, its listener changes `f3275J`, and `Z()` dispatches to the `l0()` renderer. The fixture ran extracted `R()`, `T()`, `U0()` and `Q()` Java routines on fixed sample longitudes and Equal house cusps/midpoints. The harness restored loop exits lost in the decompilation and normalized one boolean-to-integer comparison. The numbers are reproducible fixture output. Original APK numerical parity and live calculations remain unverified. [Android renderer](../../motaa-app/src/motaa/motaa/myanmar/main.java) · [Toggle listener](../../motaa-app/src/motaa/L0/s0.java)

## Verification

- All 37 regression tests passed. All 111 pages are reachable in the native prototype graph, with 2,455 navigation actions and 71 variable actions.
- Original page order, token data and component definitions were verified against the recovery master. New chart and table layers are editable.
- The source-derived [walkthrough](index.html), [previews](previews/) and [manifest](source/screen-manifest.json) are current.
- Earlier feature sources remain documented in [the desktop workflow review](DESKTOP_WORKFLOW_REVIEW.md). Advanced advertised formulas and live date, storage and chart behaviors remain implementation work.

Master SHA-256 `17c89c77679a2daca17ed48951ffbbc4c47cad1b9938c90fb56cd82f09950113`

Recovery and visual evidence directory `~/.local/share/motaa-desktop-backups/all-screen-review-20261121.LiXqjQ`


## Navigation follow-up

The walkthrough now derives hotspots from visible native layers. Explicit transparent hit regions remain active. The scan excluded 80 actions attached to hidden layers, including hidden component descendants. First launch no longer has two leftover click areas from the empty-library screen.

Saved profiles links directly to Backup & restore. The 1280 and 1024 layouts have visible entry points on Design foundations. The saved-profile list, empty library and first-launch fixtures can also be opened from the walkthrough’s screen picker.

The local Android report and `UserDBActivity` confirm distinct chart inputs and local serialized profile storage. The desktop backup flow remains a local JSON design with import review. [Android reverse-engineering notes](../../motaa-app/REVERSE_ENGINEERING.md) · [Recovered profile activity](../../motaa-app/src/motaa/motaa/myanmar/UserDBActivity.java)
