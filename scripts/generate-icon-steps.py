#!/usr/bin/env python3
"""Generate VectorCraft CLI headless batch commands for the MOTAA Desktop icon.

Brand Direction: "The Zata Matrix"
- Pure architectural symbol (zero text inside the app icon)
- Deep matte warm espresso-lacquer macOS squircle (#2B2621 -> #1E1B17 -> #14110E)
- Luminous satin champagne-gold (#F3D996 -> #D8B468 -> #B38B3E) traditional Burmese
  12-house Zata / Rasi chart geometry (outer square, inscribed diamond, four corner
  diagonals) cast as a single seamless relief with a central sculpted golden Lagna orb.
Canvas: 1024x1024 px (transparent background), 824x824 continuous-curve squircle (100px margin).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

CX = 512.0
CY = 512.0


def squircle_path(
    cx: float,
    cy: float,
    size: float,
    n: float = 4.9,
    half_quad_segs: int = 4,
) -> str:
    """Build a closed SVG cubic Bezier path for an Apple continuous-curve squircle."""
    a = size / 2.0
    x45 = a * (0.5 ** (1.0 / n))

    def y_of_x(x: float) -> float:
        u = (x / a) ** n
        return a * ((1.0 - u) ** (1.0 / n))

    def dy_dx(x: float) -> float:
        if x <= 1e-9:
            return 0.0
        u = (x / a) ** n
        return -((x / a) ** (n - 1.0)) * ((1.0 - u) ** (1.0 / n - 1.0))

    q1_segs: list[
        tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]
    ] = []

    for i in range(half_quad_segs):
        x0 = x45 * (i / half_quad_segs)
        x1 = x45 * ((i + 1) / half_quad_segs)
        dx = x1 - x0
        y0, y1 = y_of_x(x0), y_of_x(x1)
        m0, m1 = dy_dx(x0), dy_dx(x1)
        q1_segs.append(
            (
                (x0, y0),
                (x0 + dx / 3.0, y0 + m0 * dx / 3.0),
                (x1 - dx / 3.0, y1 - m1 * dx / 3.0),
                (x1, y1),
            )
        )

    for i in range(half_quad_segs):
        y0 = x45 * (1.0 - i / half_quad_segs)
        y1 = x45 * (1.0 - (i + 1) / half_quad_segs)
        dy = y1 - y0
        x0, x1 = y_of_x(y0), y_of_x(y1)
        m0, m1 = dy_dx(y0), dy_dx(y1)
        q1_segs.append(
            (
                (x0, y0),
                (x0 + m0 * dy / 3.0, y0 + dy / 3.0),
                (x1 - m1 * dy / 3.0, y1 - dy / 3.0),
                (x1, y1),
            )
        )

    def to_screen(pt: tuple[float, float], sx: float, sy: float) -> tuple[float, float]:
        return (round(cx + sx * pt[0], 2), round(cy + sy * pt[1], 2))

    all_segs = []
    for p0, p1, p2, p3 in q1_segs:
        all_segs.append((to_screen(p0, 1, -1), to_screen(p1, 1, -1), to_screen(p2, 1, -1), to_screen(p3, 1, -1)))
    for p0, p1, p2, p3 in reversed(q1_segs):
        all_segs.append((to_screen(p3, 1, 1), to_screen(p2, 1, 1), to_screen(p1, 1, 1), to_screen(p0, 1, 1)))
    for p0, p1, p2, p3 in q1_segs:
        all_segs.append((to_screen(p0, -1, 1), to_screen(p1, -1, 1), to_screen(p2, -1, 1), to_screen(p3, -1, 1)))
    for p0, p1, p2, p3 in reversed(q1_segs):
        all_segs.append((to_screen(p3, -1, -1), to_screen(p2, -1, -1), to_screen(p1, -1, -1), to_screen(p0, -1, -1)))

    start = all_segs[0][0]
    parts = [f"M {start[0]:.2f} {start[1]:.2f}"]
    for _, p1, p2, p3 in all_segs:
        parts.append(f"C {p1[0]:.2f} {p1[1]:.2f}, {p2[0]:.2f} {p2[1]:.2f}, {p3[0]:.2f} {p3[1]:.2f}")
    parts.append("Z")
    return " ".join(parts)


def build_steps() -> list[dict]:
    steps: list[dict] = []

    def add(command: str, params: dict) -> str:
        idx = len(steps)
        steps.append({"command": command, "params": params})
        return f"${idx}.id"

    def style_shape(
        obj_id: str,
        *,
        name: str,
        fill: dict | None = None,
        stroke: dict | None = None,
        stroke_opts: dict | None = None,
        opacity: float | None = None,
    ) -> None:
        if fill is not None:
            add("paint.setFill", {"id": obj_id, **fill})
        if stroke is not None:
            add("paint.setStroke", {"id": obj_id, **stroke})
        if stroke_opts is not None:
            add("stroke.set", {"id": obj_id, **stroke_opts})
        props: dict = {"id": obj_id, "name": name}
        if opacity is not None:
            props["opacity"] = opacity
        add("object.setProps", props)

    # 1. Create 1024x1024 transparent RGB document
    add(
        "file.new",
        {
            "name": "MOTAA Desktop Icon",
            "width": 1024,
            "height": 1024,
            "units": "Pixels",
            "backgroundContents": "transparent",
            "colorMode": "rgb",
            "rasterEffectsPpi": 144,
        },
    )

    # 2. Layer 1 — Deep Matte Espresso-Lacquer macOS Squircle (824x824, 100px margin)
    tile = add("path.create", {"d": squircle_path(CX, CY, 824.0)})
    style_shape(
        tile,
        name="MacOS Lacquer Squircle Base",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [512, 100],
                "end": [512, 924],
                "stops": [
                    {"offset": 0.0, "color": "#2B2621"},
                    {"offset": 0.48, "color": "#1E1B17"},
                    {"offset": 1.0, "color": "#14110E"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, 100],
                "end": [512, 924],
                "stops": [
                    {"offset": 0.0, "color": "#4E4438"},
                    {"offset": 0.5, "color": "#26211B"},
                    {"offset": 1.0, "color": "#120F0C"},
                ],
            }
        },
        stroke_opts={"weight": 2.5, "align": "inside"},
    )
    add(
        "effect.apply",
        {
            "id": tile,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 34.0,
                "x": 0.0,
                "y": 14.0,
                "blur": 24.0,
                "shade": "color",
                "color": "#0A0806",
            },
        },
    )
    add(
        "effect.apply",
        {
            "id": tile,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 18.0,
                "x": 0.0,
                "y": 3.0,
                "blur": 7.0,
                "shade": "color",
                "color": "#0A0806",
            },
        },
    )

    # Subtle inner specular rim bevel
    rim = add("path.create", {"d": squircle_path(CX, CY, 816.0)})
    style_shape(
        rim,
        name="Squircle Inner Rim Highlight",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, 104],
                "end": [512, 920],
                "stops": [
                    {"offset": 0.0, "color": "#6E5E49", "opacity": 0.42},
                    {"offset": 0.35, "color": "#383026", "opacity": 0.14},
                    {"offset": 1.0, "color": "#0E0C0A", "opacity": 0.35},
                ],
            }
        },
        stroke_opts={"weight": 2.0, "align": "inside"},
    )

    # Subtle warm ambient glow behind the gold Zata Matrix
    ambient = add(
        "shape.ellipse",
        {"x": 212.0, "y": 212.0, "width": 600.0, "height": 600.0},
    )
    style_shape(
        ambient,
        name="Warm Lacquer Ambient Glow",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 512],
                "end": [812, 512],
                "stops": [
                    {"offset": 0.0, "color": "#5C4728", "opacity": 0.22},
                    {"offset": 0.55, "color": "#30261A", "opacity": 0.08},
                    {"offset": 1.0, "color": "#1E1B17", "opacity": 0.0},
                ],
            }
        },
        stroke={"none": True},
    )

    # 3. Layer 2a — Unified Shadow Underlay Pass (eliminates internal shadow seams)
    shadow_params = {
        "mode": "multiply",
        "opacity": 44.0,
        "x": 0.0,
        "y": 7.0,
        "blur": 14.0,
        "shade": "color",
        "color": "#080604",
    }
    diagonals = [
        ("Top-Left", 266.0, 266.0, 390.0, 390.0),
        ("Top-Right", 758.0, 266.0, 634.0, 390.0),
        ("Bottom-Right", 758.0, 758.0, 634.0, 634.0),
        ("Bottom-Left", 266.0, 758.0, 390.0, 634.0),
    ]
    for label, x1, y1, x2, y2 in diagonals:
        s_seg = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            s_seg,
            name=f"Zata Shadow Diagonal {label}",
            fill={"none": True},
            stroke={"color": "#14110E"},
            stroke_opts={"weight": 24.0, "cap": "butt", "join": "miter"},
        )
        add("effect.apply", {"id": s_seg, "effect": "stylize.dropShadow", "params": shadow_params})

    s_diamond = add(
        "path.create",
        {"d": "M 512.00 268.00 L 756.00 512.00 L 512.00 756.00 L 268.00 512.00 Z"},
    )
    style_shape(
        s_diamond,
        name="Zata Shadow Diamond",
        fill={"none": True},
        stroke={"color": "#14110E"},
        stroke_opts={"weight": 24.0, "join": "bevel", "align": "center"},
    )
    add("effect.apply", {"id": s_diamond, "effect": "stylize.dropShadow", "params": shadow_params})

    s_outer = add(
        "shape.rectangle",
        {"x": 252.0, "y": 252.0, "width": 520.0, "height": 520.0},
    )
    style_shape(
        s_outer,
        name="Zata Shadow Outer Frame",
        fill={"none": True},
        stroke={"color": "#14110E"},
        stroke_opts={"weight": 26.0, "join": "miter", "miterLimit": 4.0, "align": "inside"},
    )
    add("effect.apply", {"id": s_outer, "effect": "stylize.dropShadow", "params": shadow_params})

    # 4. Layer 2b — Seamless Cast-Gold Zata Matrix (global gradient coordinates, zero seam shadows)
    gold_gradient = {
        "gradient": {
            "kind": "linear",
            "start": [252, 252],
            "end": [772, 772],
            "stops": [
                {"offset": 0.0, "color": "#F3D996"},
                {"offset": 0.46, "color": "#D8B468"},
                {"offset": 1.0, "color": "#B38B3E"},
            ],
        }
    }

    for label, x1, y1, x2, y2 in diagonals:
        seg = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            seg,
            name=f"Zata Gold Diagonal {label}",
            fill={"none": True},
            stroke=gold_gradient,
            stroke_opts={"weight": 24.0, "cap": "butt", "join": "miter"},
        )

    diamond = add(
        "path.create",
        {"d": "M 512.00 268.00 L 756.00 512.00 L 512.00 756.00 L 268.00 512.00 Z"},
    )
    style_shape(
        diamond,
        name="Zata Gold Inscribed Diamond",
        fill={"none": True},
        stroke=gold_gradient,
        stroke_opts={"weight": 24.0, "join": "bevel", "align": "center"},
    )

    outer_sq = add(
        "shape.rectangle",
        {"x": 252.0, "y": 252.0, "width": 520.0, "height": 520.0},
    )
    style_shape(
        outer_sq,
        name="Zata Gold Outer Frame",
        fill={"none": True},
        stroke=gold_gradient,
        stroke_opts={"weight": 26.0, "join": "miter", "miterLimit": 4.0, "align": "inside"},
    )

    # 5. Layer 3 — Central Celestial Lagna Orb (76x76 sculpted gold sphere)
    orb = add(
        "shape.ellipse",
        {"x": 474.0, "y": 474.0, "width": 76.0, "height": 76.0},
    )
    style_shape(
        orb,
        name="Lagna Celestial Orb",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [498, 496],
                "end": [552, 552],
                "stops": [
                    {"offset": 0.0, "color": "#FFF3C8"},
                    {"offset": 0.45, "color": "#E2BE72"},
                    {"offset": 1.0, "color": "#A67E33"},
                ],
            }
        },
        stroke={"none": True},
    )
    add(
        "effect.apply",
        {
            "id": orb,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 52.0,
                "x": 0.0,
                "y": 6.0,
                "blur": 14.0,
                "shade": "color",
                "color": "#080604",
            },
        },
    )

    return steps


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit(f"Usage: {sys.argv[0]} <output-steps.json>")
    out = Path(sys.argv[1])
    out.write_text(json.dumps(build_steps(), indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
