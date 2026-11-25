# MOTAA V31.1 · Learn the algorithms visually

This folder is a **source-informed educational walkthrough**, not an implemented calculation engine and not a numerical-parity certificate.

## Viewing

Open **[index.html](index.html)** in a browser. Every chapter includes the editable PlantUML source, a rendered SVG, and an exported PNG. The original FigCraft single source of truth remains at `../motaa-desktop.figcraft` and is deliberately unchanged by this lesson.

| Lesson | What it explains | Confidence |
| --- | --- | --- |
| [01. Two calculation paths](01-original-two-paths.svg) | Planetary astronomy and independent Myanmar calendar branches | Verified statically |
| [02. Astronomy sequence](02-astronomy-sequence.svg) | Input → Julian date → ephemeris → houses and rules | Source-mapped conceptual ordering |
| [03. Myanmar calendar](03-myanmar-calendar.svg) | JDN → Myanmar year → watat → month → traditional flags | Verified statically; numerical parity pending |
| [04. Dasha](04-dasha-walkthrough.svg) | Moon longitude, Nakshatra lookup, 120-year cycle and source date constants | Recovered formulas; dynamic parity pending |
| [05. Future Rust engine](05-proposed-rust-engine.svg) | Example modular architecture and CLI connection to a future desktop | **Proposal only** |
| [06. Verification gates](06-verification-gates.svg) | How to compare a new implementation with reference APK output | **Planned test strategy** |

## Source of facts

Recovered research is in:
`<source-research-repo>/MASTER_REVERSE_ENGINEERING.md`

Relevant existing files:
- `src/motaa/motaa/myanmar/main.java`
- `src/motaa/L0/AbstractC0055z.java`
- `src/motaa/L0/C0052w.java`
- `src/motaa/L0/AbstractC0029c.java`
- `src/motaa/L0/u0.java`

The source was decompiled using JADX, so diagram call ordering is intended to teach the model, not assert every individual runtime invocation. Names like "Rust engine" are **not** recovered facts.

## Regenerate

From this directory:

    plantuml --check-syntax *.puml
    plantuml --svg *.puml
    plantuml --png *.puml

This folder is intentionally separate from the canonical FigCraft master and from production code.
