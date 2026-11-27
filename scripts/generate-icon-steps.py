#!/usr/bin/env python3
"""Generate VectorCraft CLI headless batch commands for the MOTAA Desktop icon.

Design system: Gold Heritage Light / Observatory Manuscript (DESIGN.md)
Direction: Engraved Burmese Zata (ဇာတာ) on Warm Ivory Parchment with a
Flush-Inset Burnished-Bronze Center Plaque & Precision Astronomical Origin Reticle (☉).
Canvas: 1024x1024 px (transparent background), 824x824 continuous-curve squircle (100px margin).
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

CX = 512.0
CY = 512.0


def polar(r: float, deg: float, cx: float = CX, cy: float = CY) -> tuple[float, float]:
    rad = math.radians(deg)
    return (round(cx + r * math.cos(rad), 2), round(cy + r * math.sin(rad), 2))


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

    q1_segs: list[tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]] = []

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

    # 2. Layer 1 — Apple macOS HIG Warm Ivory Parchment Squircle (824x824, 100px margin)
    tile = add("path.create", {"d": squircle_path(CX, CY, 824.0)})
    style_shape(
        tile,
        name="MacOS Squircle Base",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [512, 100],
                "end": [512, 924],
                "stops": [
                    {"offset": 0.0, "color": "#FFFEFA"},
                    {"offset": 0.52, "color": "#F7F2E6"},
                    {"offset": 1.0, "color": "#E8DEC6"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, 100],
                "end": [512, 924],
                "stops": [
                    {"offset": 0.0, "color": "#EFE4CA"},
                    {"offset": 1.0, "color": "#BFA574"},
                ],
            }
        },
        stroke_opts={"weight": 2.0, "align": "inside"},
    )
    add(
        "effect.apply",
        {
            "id": tile,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 26.0,
                "x": 0.0,
                "y": 12.0,
                "blur": 22.0,
                "shade": "color",
                "color": "#1C140A",
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
                "opacity": 14.0,
                "x": 0.0,
                "y": 3.0,
                "blur": 6.0,
                "shade": "color",
                "color": "#1C140A",
            },
        },
    )

    # Top specular ceramic bevel inside squircle edge
    spec_rim = add("path.create", {"d": squircle_path(CX, CY, 820.0)})
    style_shape(
        spec_rim,
        name="Squircle Specular Rim",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, 102],
                "end": [512, 922],
                "stops": [
                    {"offset": 0.0, "color": "#FFFFFF", "opacity": 0.92},
                    {"offset": 0.36, "color": "#FFFFFF", "opacity": 0.0},
                    {"offset": 1.0, "color": "#7C5B21", "opacity": 0.14},
                ],
            }
        },
        stroke_opts={"weight": 3.0, "align": "inside"},
    )

    # 3. Layer 2 — Engraved Burmese Zata Architectural Plate (576x576, exact 192px thirds)
    z0 = 224.0
    z_size = 576.0
    cell = z_size / 3.0  # 192.0
    gx1 = z0 + cell      # 416.0
    gx2 = z0 + 2 * cell  # 608.0
    z1 = z0 + z_size     # 800.0

    zata_plate = add(
        "shape.rectangle",
        {"x": z0, "y": z0, "width": z_size, "height": z_size, "radius": 4},
    )
    style_shape(
        zata_plate,
        name="Zata Rag-Paper Plate",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [512, z0],
                "end": [512, z1],
                "stops": [
                    {"offset": 0.0, "color": "#FCF9F1"},
                    {"offset": 1.0, "color": "#EFE6D2"},
                ],
            }
        },
        stroke={"none": True},
    )

    # Quiet tonal warmth on the 4 cardinal outer houses
    for name, rx, ry in [
        ("Zata North House Tint", gx1, z0),
        ("Zata South House Tint", gx1, gx2),
        ("Zata West House Tint", z0, gx1),
        ("Zata East House Tint", gx2, gx1),
    ]:
        card = add("shape.rectangle", {"x": rx, "y": ry, "width": cell, "height": cell})
        style_shape(card, name=name, fill={"color": "#DFCFAF"}, stroke={"none": True}, opacity=16)

    # Engraved espresso-bronze Zata lines
    line_stroke = {
        "gradient": {
            "kind": "linear",
            "start": [512, z0],
            "end": [512, z1],
            "stops": [
                {"offset": 0.0, "color": "#7B5A21"},
                {"offset": 0.5, "color": "#5E4315"},
                {"offset": 1.0, "color": "#3F2C0C"},
            ],
        }
    }

    # 4 corner diagonals (exact vertex-to-vertex)
    for name, x1, y1, x2, y2 in [
        ("Zata Diagonal NW", z0, z0, gx1, gx1),
        ("Zata Diagonal NE", z1, z0, gx2, gx1),
        ("Zata Diagonal SW", z0, z1, gx1, gx2),
        ("Zata Diagonal SE", z1, z1, gx2, gx2),
    ]:
        diag = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            diag,
            name=name,
            fill={"none": True},
            stroke=line_stroke,
            stroke_opts={"weight": 8.0, "cap": "butt"},
        )

    # 4 internal thirds lines
    for name, x1, y1, x2, y2 in [
        ("Zata Vertical Left", gx1, z0, gx1, z1),
        ("Zata Vertical Right", gx2, z0, gx2, z1),
        ("Zata Horizontal Top", z0, gx1, z1, gx1),
        ("Zata Horizontal Bottom", z0, gx2, z1, gx2),
    ]:
        line = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            line,
            name=name,
            fill={"none": True},
            stroke=line_stroke,
            stroke_opts={"weight": 8.5, "cap": "butt"},
        )

    # Outer Zata frame drawn above internal lines so corners and line ends lock cleanly
    zata_frame = add(
        "shape.rectangle",
        {"x": z0, "y": z0, "width": z_size, "height": z_size, "radius": 4},
    )
    style_shape(
        zata_frame,
        name="Zata Outer Frame",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, z0],
                "end": [512, z1],
                "stops": [
                    {"offset": 0.0, "color": "#70511C"},
                    {"offset": 0.5, "color": "#543B12"},
                    {"offset": 1.0, "color": "#38260A"},
                ],
            }
        },
        stroke_opts={"weight": 12.0, "align": "center"},
    )

    # 4. Layer 3 — Flush-Inset Burnished Antique-Bronze Center Square Plaque (192x192 at 416..608)
    plaque = add(
        "shape.rectangle",
        {"x": gx1, "y": gx1, "width": cell, "height": cell, "radius": 2},
    )
    style_shape(
        plaque,
        name="Center Bronze Plaque",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [512, gx1],
                "end": [512, gx2],
                "stops": [
                    {"offset": 0.0, "color": "#A47D32"},
                    {"offset": 0.45, "color": "#7C5B21"},
                    {"offset": 1.0, "color": "#523A11"},
                ],
            }
        },
        stroke={"color": "#432E0C"},
        stroke_opts={"weight": 2.5, "align": "inside"},
    )
    add(
        "effect.apply",
        {
            "id": plaque,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 24.0,
                "x": 0.0,
                "y": 6.0,
                "blur": 12.0,
                "shade": "color",
                "color": "#2A1B06",
            },
        },
    )

    # Machined inner chamfer highlight inside the bronze plaque
    chamfer = add(
        "shape.rectangle",
        {"x": gx1 + 5.0, "y": gx1 + 5.0, "width": cell - 10.0, "height": cell - 10.0, "radius": 1},
    )
    style_shape(
        chamfer,
        name="Plaque Chamfer Highlight",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, gx1 + 5.0],
                "end": [512, gx2 - 5.0],
                "stops": [
                    {"offset": 0.0, "color": "#F6E1A4", "opacity": 0.68},
                    {"offset": 0.42, "color": "#FFFFFF", "opacity": 0.0},
                    {"offset": 1.0, "color": "#2F1F07", "opacity": 0.30},
                ],
            }
        },
        stroke_opts={"weight": 1.5},
    )

    # 5. Layer 4 — Engraved Ivory Precision Astronomical Origin Mark (☉ Transit Reticle)
    reticle_ids: list[str] = []

    r_ring = 44.0
    ecliptic_ring = add(
        "shape.ellipse",
        {"x": CX - r_ring, "y": CY - r_ring, "width": r_ring * 2, "height": r_ring * 2},
    )
    style_shape(
        ecliptic_ring,
        name="Origin Ecliptic Ring",
        fill={"none": True},
        stroke={"color": "#FFFDF7"},
        stroke_opts={"weight": 9.0},
    )
    reticle_ids.append(ecliptic_ring)

    r_core = 12.0
    origin_core = add(
        "shape.ellipse",
        {"x": CX - r_core, "y": CY - r_core, "width": r_core * 2, "height": r_core * 2},
    )
    style_shape(
        origin_core,
        name="Origin Solar Point",
        fill={"color": "#FFFDF7"},
        stroke={"none": True},
    )
    reticle_ids.append(origin_core)

    for idx, deg in enumerate((-90.0, 0.0, 90.0, 180.0)):
        x1, y1 = polar(55.0, deg)
        x2, y2 = polar(68.0, deg)
        tick = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            tick,
            name=f"Origin Meridian Tick {idx + 1}",
            fill={"none": True},
            stroke={"color": "#F4E6C6"},
            stroke_opts={"weight": 4.5, "cap": "round"},
            opacity=85,
        )
        reticle_ids.append(tick)

    reticle_group = add("object.group", {"ids": reticle_ids})
    add("object.setProps", {"id": reticle_group, "name": "Astronomical Origin Reticle"})
    add(
        "effect.apply",
        {
            "id": reticle_group,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 24.0,
                "x": 0.0,
                "y": 2.0,
                "blur": 4.0,
                "shade": "color",
                "color": "#2D1E07",
            },
        },
    )

    return steps


def main() -> None:
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    steps = build_steps()
    payload = json.dumps(steps, indent=2) + "\n"
    if out_path:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(payload, encoding="utf-8")
    else:
        sys.stdout.write(payload)


if __name__ == "__main__":
    main()
