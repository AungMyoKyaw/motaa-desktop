> **Historical review (pre–Native Precision v2).** For the latest validated design and counts, see [NATIVE_PRECISION_V2_REVIEW.md](NATIVE_PRECISION_V2_REVIEW.md). This document records the earlier baseline and is retained for comparison.

# MOTAA desktop — macOS visual-direction review
17 November 2026

## Design direction

MOTAA is a chart-first desktop application, not a reverse-engineering workbook. Its visual language now draws from macOS Preview and System Settings: calm window chrome, compact navigation, familiar window controls, legible form fields, and contextual detail panels.

**Single editable design source:** design/motaa-desktop.figcraft.

## Completed work

| Scope | Verified result |
| --- | --- |
| Original application pages 01–58 | Neutral macOS-style window/sidebar palette, traffic-light controls, condensed navigation and blue primary-action treatment. Original plot lines/vectors inside the chart content area were not modified. |
| Source-state pages 59–90 | Removed visible reconstruction badges; unified toolbar, compact five-workspace sidebar, selected row, contextual inspector, cleaner section hierarchy and native-style controls. |
| Birth/Current Location 69–70 | Real label/value visual layers for six fields; original example coordinates and UTC offset retained. |
| Responsive references 87–88 | Matched the same window treatment, constrained sidebar controls, and suppressed duplicate decorative/engineering badges. |
| Engineering copy | Moved source/prototype language away from the visible application UI; retained source reasoning in SOURCE_TRACEABILITY.md and the recovered source repository. |
| Lessons 91–100 | Preserved as editable educational references, separate from the shipped product UI. |

The five actual workspaces remain Charts, Profiles, Calculation Setup, Time & Analysis, and Advanced Analysis. All original chart modes, including free Rasi aspects, remain accessible.

## Verification

- FigCraft CLI 0.15.0; all design operations performed through native FigCraft edits on a project-external candidate.
- 100 pages rendered, zero rendering errors; 90 application-facing pages styled, ten algorithm lessons unchanged.
- 18,376 native nodes; exactly one FigCraft master.
- Five application prototype flows, 1,297 native navigation actions, 35 native variable actions preserved.
- Zero out-of-bounds elements, text-box overlaps, invalid navigation links, or changed-text fitting warnings (5px font metric tolerance for rounding).
- Zero missing system fonts; 15/15 design regression tests passed.
- Original line, vector and ellipse chart-plot geometry in the first 58 application pages (content x >= 208, y >= 180) unchanged.
- Original reverse-engineered application source and runtime code untouched.

**Recovery baseline:** ~/.local/share/motaa-desktop-backups/macos-rebuild-20261117-135910/baseline.figcraft

**Full-size visual audit:** ~/.local/share/motaa-desktop-backups/macos-rebuild-20261117-135910/all-100-renders/

## Boundaries

This is a **native-style FigCraft visual design**, not a running AppKit/Tauri application. Keyboard-focus traversal, screen-reader semantics, native platform menus, persistence, responsive behavior under real resizing, and actual Swiss Ephemeris / MOTAA numerical parity require independent implementation and testing.
