#!/usr/bin/env python3
"""Generate VectorCraft CLI headless batch commands for the MOTAA Desktop icon.

Design system: Gold Heritage Light / Observatory Manuscript (DESIGN.md)
Spec: Apple macOS HIG 1024x1024 canvas, 824x824 continuous-curve squircle (100px margin),
top-lit studio lighting, layered drop shadow, metallic gold astrolabe bezel, deep obsidian
lacquer dial, 12-house zodiac wheel with the 7 classical MOTAA planetary glyphs, and a
3D-faceted golden celestial compass emblem.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

CX = 512.0
CY = 512.0

LUMINOUS_GOLD = "#F4DC96"
DEEP_SHADOW = "#1A1208"


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
    """Build a closed SVG cubic Bezier path for an Apple continuous-curve squircle.

    Uses piecewise analytic parameterization (by x near horizontal caps, by y near
    vertical sides) across the 8 octants so derivatives stay bounded in [-1, 1].
    """
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

    # First-quadrant octant points (y-up mathematical coords):
    # Octant A: top center (0, a) -> 45 deg (x45, x45)
    # Octant B: 45 deg (x45, x45) -> right center (a, 0)
    # Store each segment as (p0, p1, p2, p3) in math coords (y-up).
    q1_segs: list[tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]] = []

    for i in range(half_quad_segs):
        x0 = x45 * (i / half_quad_segs)
        x1 = x45 * ((i + 1) / half_quad_segs)
        dx = x1 - x0
        y0, y1 = y_of_x(x0), y_of_x(x1)
        m0, m1 = dy_dx(x0), dy_dx(x1)
        p0 = (x0, y0)
        p1 = (x0 + dx / 3.0, y0 + m0 * dx / 3.0)
        p2 = (x1 - dx / 3.0, y1 - m1 * dx / 3.0)
        p3 = (x1, y1)
        q1_segs.append((p0, p1, p2, p3))

    for i in range(half_quad_segs):
        y0 = x45 * (1.0 - i / half_quad_segs)
        y1 = x45 * (1.0 - (i + 1) / half_quad_segs)
        dy = y1 - y0  # negative
        x0, x1 = y_of_x(y0), y_of_x(y1)
        m0, m1 = dy_dx(y0), dy_dx(y1)  # dx/dy
        p0 = (x0, y0)
        p1 = (x0 + m0 * dy / 3.0, y0 + dy / 3.0)
        p2 = (x1 - m1 * dy / 3.0, y1 - dy / 3.0)
        p3 = (x1, y1)
        q1_segs.append((p0, p1, p2, p3))

    # Map clockwise around all 4 quadrants in screen coordinates (y-down):
    # Q1: (+x, -y) top -> right
    # Q4: (+x, +y) right -> bottom (reversed segment order)
    # Q3: (-x, +y) bottom -> left
    # Q2: (-x, -y) left -> top (reversed segment order)
    all_segs: list[tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]] = []

    def to_screen(pt: tuple[float, float], sx: float, sy: float) -> tuple[float, float]:
        return (round(cx + sx * pt[0], 2), round(cy + sy * pt[1], 2))

    # Top -> Right
    for p0, p1, p2, p3 in q1_segs:
        all_segs.append((to_screen(p0, 1, -1), to_screen(p1, 1, -1), to_screen(p2, 1, -1), to_screen(p3, 1, -1)))
    # Right -> Bottom
    for p0, p1, p2, p3 in reversed(q1_segs):
        all_segs.append((to_screen(p3, 1, 1), to_screen(p2, 1, 1), to_screen(p1, 1, 1), to_screen(p0, 1, 1)))
    # Bottom -> Left
    for p0, p1, p2, p3 in q1_segs:
        all_segs.append((to_screen(p0, -1, 1), to_screen(p1, -1, 1), to_screen(p2, -1, 1), to_screen(p3, -1, 1)))
    # Left -> Top
    for p0, p1, p2, p3 in reversed(q1_segs):
        all_segs.append((to_screen(p3, -1, -1), to_screen(p2, -1, -1), to_screen(p1, -1, -1), to_screen(p0, -1, -1)))

    start = all_segs[0][0]
    parts = [f"M {start[0]:.2f} {start[1]:.2f}"]
    for _, p1, p2, p3 in all_segs:
        parts.append(f"C {p1[0]:.2f} {p1[1]:.2f}, {p2[0]:.2f} {p2[1]:.2f}, {p3[0]:.2f} {p3[1]:.2f}")
    parts.append("Z")
    return " ".join(parts)


def poly_path(points: list[tuple[float, float]]) -> str:
    parts = [f"M {points[0][0]:.2f} {points[0][1]:.2f}"]
    for x, y in points[1:]:
        parts.append(f"L {x:.2f} {y:.2f}")
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

    def add_glyph_path(name: str, d: str, weight: float = 10.5, fill: dict | None = None) -> str:
        pid = add("path.create", {"d": d})
        style_shape(
            pid,
            name=name,
            fill=fill if fill is not None else {"none": True},
            stroke={"color": LUMINOUS_GOLD},
            stroke_opts={"weight": weight, "cap": "round", "join": "round"},
        )
        return pid

    def add_glyph_circle(
        name: str,
        cx: float,
        cy: float,
        r: float,
        weight: float = 10.5,
        filled: bool = False,
    ) -> str:
        eid = add(
            "shape.ellipse",
            {
                "x": round(cx - r, 2),
                "y": round(cy - r, 2),
                "width": round(r * 2, 2),
                "height": round(r * 2, 2),
            },
        )
        if filled:
            style_shape(
                eid,
                name=name,
                fill={"color": LUMINOUS_GOLD},
                stroke={"none": True},
            )
        else:
            style_shape(
                eid,
                name=name,
                fill={"none": True},
                stroke={"color": LUMINOUS_GOLD},
                stroke_opts={"weight": weight, "cap": "round", "join": "round"},
            )
        return eid

    def add_glyph_line(
        name: str,
        x1: float,
        y1: float,
        x2: float,
        y2: float,
        weight: float = 10.5,
    ) -> str:
        lid = add(
            "shape.line",
            {
                "x1": round(x1, 2),
                "y1": round(y1, 2),
                "x2": round(x2, 2),
                "y2": round(y2, 2),
            },
        )
        style_shape(
            lid,
            name=name,
            fill={"none": True},
            stroke={"color": LUMINOUS_GOLD},
            stroke_opts={"weight": weight, "cap": "round", "join": "round"},
        )
        return lid

    # 1. Create 1024x1024 transparent RGB document with 144 ppi raster effects for crisp shadows
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

    # 2. Outer Apple HIG continuous-curve squircle (824x824 centered at 512,512; 100px gutter)
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
                    {"offset": 0.0, "color": "#FFFDF7"},
                    {"offset": 0.52, "color": "#F5EFE0"},
                    {"offset": 1.0, "color": "#E5D8BC"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, 100],
                "end": [512, 924],
                "stops": [
                    {"offset": 0.0, "color": "#F4E1AF"},
                    {"offset": 0.48, "color": "#C49A45"},
                    {"offset": 1.0, "color": "#7C5B21"},
                ],
            }
        },
        stroke_opts={"weight": 4.0, "align": "inside"},
    )
    # Grounded macOS HIG Dock shadow: soft ambient drop + crisp contact shadow
    add(
        "effect.apply",
        {
            "id": tile,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 32.0,
                "x": 0.0,
                "y": 12.0,
                "blur": 22.0,
                "shade": "color",
                "color": DEEP_SHADOW,
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
                "blur": 6.0,
                "shade": "color",
                "color": DEEP_SHADOW,
            },
        },
    )

    # 3. Top specular ceramic bevel rim inside the squircle edge
    spec_rim = add("path.create", {"d": squircle_path(CX, CY, 816.0)})
    style_shape(
        spec_rim,
        name="Squircle Specular Rim",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, 104],
                "end": [512, 920],
                "stops": [
                    {"offset": 0.0, "color": "#FFFFFF", "opacity": 0.90},
                    {"offset": 0.38, "color": "#FFFFFF", "opacity": 0.0},
                    {"offset": 1.0, "color": "#694A17", "opacity": 0.18},
                ],
            }
        },
        stroke_opts={"weight": 4.0, "align": "inside"},
    )

    # 4. Sculpted metallic gold astrolabe bezel (r = 342)
    r_bezel = 342.0
    bezel = add(
        "shape.ellipse",
        {"x": CX - r_bezel, "y": CY - r_bezel, "width": r_bezel * 2, "height": r_bezel * 2},
    )
    style_shape(
        bezel,
        name="Astrolabe Gold Bezel",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [512, CY - r_bezel],
                "end": [512, CY + r_bezel],
                "stops": [
                    {"offset": 0.0, "color": "#F7E3A8"},
                    {"offset": 0.28, "color": "#D6AE55"},
                    {"offset": 0.68, "color": "#9E7628"},
                    {"offset": 1.0, "color": "#654714"},
                ],
            }
        },
        stroke={"color": "#5B3F12"},
        stroke_opts={"weight": 2.0},
    )
    add(
        "effect.apply",
        {
            "id": bezel,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 34.0,
                "x": 0.0,
                "y": 10.0,
                "blur": 16.0,
                "shade": "color",
                "color": "#261806",
            },
        },
    )

    # 5. Bezel inner chamfer highlight ring (r = 330)
    r_chamfer = 330.0
    chamfer = add(
        "shape.ellipse",
        {"x": CX - r_chamfer, "y": CY - r_chamfer, "width": r_chamfer * 2, "height": r_chamfer * 2},
    )
    style_shape(
        chamfer,
        name="Astrolabe Bezel Highlight",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, CY - r_chamfer],
                "end": [512, CY + r_chamfer],
                "stops": [
                    {"offset": 0.0, "color": "#FFF5D6", "opacity": 0.78},
                    {"offset": 0.5, "color": "#8B6B2E", "opacity": 0.15},
                    {"offset": 1.0, "color": "#4A330D", "opacity": 0.50},
                ],
            }
        },
        stroke_opts={"weight": 3.5},
    )

    # 6. Deep obsidian-walnut enamel dial (r = 314)
    r_dial = 314.0
    dial = add(
        "shape.ellipse",
        {"x": CX - r_dial, "y": CY - r_dial, "width": r_dial * 2, "height": r_dial * 2},
    )
    style_shape(
        dial,
        name="Obsidian Observatory Dial",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 370],
                "end": [512, 820],
                "stops": [
                    {"offset": 0.0, "color": "#3A2E20"},
                    {"offset": 0.52, "color": "#221A12"},
                    {"offset": 1.0, "color": "#120D08"},
                ],
            }
        },
        stroke={"color": "#0D0905"},
        stroke_opts={"weight": 4.0, "align": "inside"},
    )
    add(
        "effect.apply",
        {
            "id": dial,
            "effect": "stylize.innerGlow",
            "params": {
                "mode": "multiply",
                "opacity": 45.0,
                "blur": 14.0,
                "color": "#090603",
                "source": "edge",
            },
        },
    )

    # 7. Precision graduation ticks & 12-house zodiac wheel
    r_outer = 304.0
    r_inner = 186.0
    r_mid = (r_outer + r_inner) / 2.0  # 245.0

    # Subtle degree ticks every 10 degrees (skipping the 12 main house spoke angles)
    spoke_angles = {round((-105.0 + i * 30.0) % 360.0, 1) for i in range(12)}
    for step in range(36):
        deg = -90.0 + step * 10.0
        if round(deg % 360.0, 1) in spoke_angles:
            continue
        x1, y1 = polar(294.0, deg)
        x2, y2 = polar(r_outer, deg)
        tick = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            tick,
            name=f"Dial Degree Tick {step + 1}",
            fill={"none": True},
            stroke={"color": "#D6B15E"},
            stroke_opts={"weight": 2.0, "cap": "butt"},
            opacity=36,
        )

    # 12 radial house spokes
    for i in range(12):
        deg = -105.0 + i * 30.0
        x1, y1 = polar(r_inner, deg)
        x2, y2 = polar(r_outer, deg)
        spoke = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            spoke,
            name=f"Zodiac House Spoke {i + 1}",
            fill={"none": True},
            stroke={"color": "#C89F48"},
            stroke_opts={"weight": 5.5, "cap": "butt"},
            opacity=82,
        )

    # Outer zodiac track boundary ring (r = 304)
    outer_ring = add(
        "shape.ellipse",
        {"x": CX - r_outer, "y": CY - r_outer, "width": r_outer * 2, "height": r_outer * 2},
    )
    style_shape(
        outer_ring,
        name="Zodiac Outer Track Ring",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, CY - r_outer],
                "end": [512, CY + r_outer],
                "stops": [
                    {"offset": 0.0, "color": "#F5DF9E"},
                    {"offset": 0.5, "color": "#C49940"},
                    {"offset": 1.0, "color": "#7D5B1C"},
                ],
            }
        },
        stroke_opts={"weight": 4.5},
    )

    # 8. Recessed inner sanctuary well (r = 186)
    sanctuary = add(
        "shape.ellipse",
        {"x": CX - r_inner, "y": CY - r_inner, "width": r_inner * 2, "height": r_inner * 2},
    )
    style_shape(
        sanctuary,
        name="Sanctuary Well",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 460],
                "end": [512, 698],
                "stops": [
                    {"offset": 0.0, "color": "#2C2216"},
                    {"offset": 0.72, "color": "#1B140C"},
                    {"offset": 1.0, "color": "#110C07"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "start": [512, CY - r_inner],
                "end": [512, CY + r_inner],
                "stops": [
                    {"offset": 0.0, "color": "#F5DF9E"},
                    {"offset": 0.5, "color": "#B88B36"},
                    {"offset": 1.0, "color": "#6B4C16"},
                ],
            }
        },
        stroke_opts={"weight": 6.5},
    )

    r_sanct_filigree = 166.0
    sanct_filigree = add(
        "shape.ellipse",
        {
            "x": CX - r_sanct_filigree,
            "y": CY - r_sanct_filigree,
            "width": r_sanct_filigree * 2,
            "height": r_sanct_filigree * 2,
        },
    )
    style_shape(
        sanct_filigree,
        name="Sanctuary Inner Ring",
        fill={"none": True},
        stroke={"color": "#D8B35F"},
        stroke_opts={"weight": 2.0},
        opacity=42,
    )

    # 9. Seven classical planetary glyphs in their exact MOTAA ruling signs (luminous gold on obsidian)
    # --- 12 o'clock (-90 deg): Mars ---
    mx, my = polar(r_mid, -90.0)
    add_glyph_circle("Glyph Mars Circle", mx - 9.0, my + 9.0, 20.0)
    add_glyph_line("Glyph Mars Shaft", mx + 5.5, my - 5.5, mx + 26.5, my - 26.5)
    add_glyph_path(
        "Glyph Mars Arrowhead",
        f"M {mx + 9.0:.2f} {my - 26.5:.2f} L {mx + 26.5:.2f} {my - 26.5:.2f} L {mx + 26.5:.2f} {my - 9.0:.2f}",
    )

    # --- 2 o'clock (-30 deg): Saturn ---
    sx, sy = polar(r_mid, -30.0)
    add_glyph_line("Glyph Saturn Stem", sx - 13, sy - 32, sx - 13, sy + 17)
    add_glyph_line("Glyph Saturn Crossbar", sx - 26, sy - 19, sx + 1, sy - 19)
    add_glyph_path(
        "Glyph Saturn Arch",
        (
            f"M {sx - 13:.2f} {sy - 1:.2f} "
            f"C {sx - 3:.2f} {sy - 15:.2f}, {sx + 17:.2f} {sy - 15:.2f}, {sx + 17:.2f} {sy + 2:.2f} "
            f"C {sx + 17:.2f} {sy + 15:.2f}, {sx + 10:.2f} {sy + 25:.2f}, {sx + 18:.2f} {sy + 30:.2f} "
            f"C {sx + 22:.2f} {sy + 32:.2f}, {sx + 27:.2f} {sy + 29:.2f}, {sx + 29:.2f} {sy + 24:.2f}"
        ),
    )

    # --- 4 o'clock (+30 deg): Jupiter ---
    jx, jy = polar(r_mid, 30.0)
    add_glyph_path(
        "Glyph Jupiter Hook",
        (
            f"M {jx - 23:.2f} {jy - 15:.2f} "
            f"C {jx - 22:.2f} {jy - 32:.2f}, {jx - 1:.2f} {jy - 34:.2f}, {jx - 1:.2f} {jy - 14:.2f} "
            f"C {jx - 1:.2f} {jy - 2:.2f}, {jx - 13:.2f} {jy + 7:.2f}, {jx - 25:.2f} {jy + 13:.2f} "
            f"L {jx + 25:.2f} {jy + 13:.2f}"
        ),
    )
    add_glyph_line("Glyph Jupiter Stem", jx + 12, jy - 19, jx + 12, jy + 32)

    # --- 6 o'clock (+90 deg): Venus ---
    vx, vy = polar(r_mid, 90.0)
    add_glyph_circle("Glyph Venus Circle", vx, vy - 13, 19.5)
    add_glyph_line("Glyph Venus Stem", vx, vy + 7, vx, vy + 35)
    add_glyph_line("Glyph Venus Crossbar", vx - 15, vy + 22, vx + 15, vy + 22)

    # --- 7 o'clock (+120 deg): Mercury ---
    ex, ey = polar(r_mid, 120.0)
    add_glyph_path(
        "Glyph Mercury Horns",
        f"M {ex - 16:.2f} {ey - 33:.2f} C {ex - 13:.2f} {ey - 19:.2f}, {ex + 13:.2f} {ey - 19:.2f}, {ex + 16:.2f} {ey - 33:.2f}",
    )
    add_glyph_circle("Glyph Mercury Circle", ex, ey - 3, 17.5)
    add_glyph_line("Glyph Mercury Stem", ex, ey + 15, ex, ey + 37)
    add_glyph_line("Glyph Mercury Crossbar", ex - 14, ey + 26, ex + 14, ey + 26)

    # --- 8 o'clock (+150 deg): Sun ---
    ux, uy = polar(r_mid, 150.0)
    add_glyph_circle("Glyph Sun Ring", ux, uy, 26.5)
    add_glyph_circle("Glyph Sun Core", ux, uy, 7.5, filled=True)

    # --- 9 o'clock (180 deg): Moon ---
    lx, ly = polar(r_mid, 180.0)
    add_glyph_path(
        "Glyph Moon Crescent",
        (
            f"M {lx - 14:.2f} {ly - 28:.2f} "
            f"C {lx + 11:.2f} {ly - 29:.2f}, {lx + 26:.2f} {ly - 13:.2f}, {lx + 26:.2f} {ly:.2f} "
            f"C {lx + 26:.2f} {ly + 13:.2f}, {lx + 11:.2f} {ly + 29:.2f}, {lx - 14:.2f} {ly + 28:.2f} "
            f"C {lx + 1:.2f} {ly + 20:.2f}, {lx + 8:.2f} {ly + 11:.2f}, {lx + 8:.2f} {ly:.2f} "
            f"C {lx + 8:.2f} {ly - 11:.2f}, {lx + 1:.2f} {ly - 20:.2f}, {lx - 14:.2f} {ly - 28:.2f} Z"
        ),
        weight=9.5,
    )

    # 10. Sculpted 3D-lit Golden Celestial Compass Star (replaces small wordmark for crispness at 16px+)
    r_cardinal = 148.0
    r_inter = 104.0
    r_waist_c = 38.0
    r_waist_i = 34.0

    # Underlying star silhouette casting soft shadow into the sanctuary well
    star_pts: list[tuple[float, float]] = []
    for k in range(8):
        angle = -90.0 + k * 45.0
        tip_r = r_cardinal if k % 2 == 0 else r_inter
        waist_r = r_waist_c if k % 2 == 0 else r_waist_i
        star_pts.append(polar(tip_r, angle))
        star_pts.append(polar(waist_r, angle + 22.5))

    star_shadow = add("path.create", {"d": poly_path(star_pts)})
    style_shape(
        star_shadow,
        name="Celestial Star Shadow Base",
        fill={"color": "#8B6B2E"},
        stroke={"none": True},
    )
    add(
        "effect.apply",
        {
            "id": star_shadow,
            "effect": "stylize.dropShadow",
            "params": {
                "mode": "multiply",
                "opacity": 55.0,
                "x": 0.0,
                "y": 8.0,
                "blur": 12.0,
                "shade": "color",
                "color": "#0B0703",
            },
        },
    )

    center_pt = (CX, CY)

    def add_ray_facets(
        name_prefix: str,
        angle: float,
        tip_r: float,
        waist_r: float,
        lit_side_ccw: bool,
    ) -> None:
        tip = polar(tip_r, angle)
        w_ccw = polar(waist_r, angle - 22.5)
        w_cw = polar(waist_r, angle + 22.5)
        lit_fill = {
            "gradient": {
                "kind": "linear",
                "start": [CX, CY - 140],
                "end": [CX, CY + 140],
                "stops": [
                    {"offset": 0.0, "color": "#FAEAAC"},
                    {"offset": 1.0, "color": "#DDB65B"},
                ],
            }
        }
        shade_fill = {
            "gradient": {
                "kind": "linear",
                "start": [CX, CY - 140],
                "end": [CX, CY + 140],
                "stops": [
                    {"offset": 0.0, "color": "#B68831"},
                    {"offset": 1.0, "color": "#745218"},
                ],
            }
        }
        f_ccw = add("path.create", {"d": poly_path([center_pt, w_ccw, tip])})
        style_shape(
            f_ccw,
            name=f"{name_prefix} Facet A",
            fill=lit_fill if lit_side_ccw else shade_fill,
            stroke={"none": True},
        )
        f_cw = add("path.create", {"d": poly_path([center_pt, tip, w_cw])})
        style_shape(
            f_cw,
            name=f"{name_prefix} Facet B",
            fill=shade_fill if lit_side_ccw else lit_fill,
            stroke={"none": True},
        )

    # Secondary intercardinal rays (45, 135, 225, 315)
    for idx, angle in enumerate([-45.0, 45.0, 135.0, -135.0]):
        add_ray_facets(f"Intercardinal Ray {idx + 1}", angle, r_inter, r_waist_i, lit_side_ccw=(angle in (-45.0, -135.0)))

    # Primary cardinal rays (-90 N, 0 E, 90 S, 180 W)
    for idx, angle in enumerate([-90.0, 0.0, 90.0, 180.0]):
        add_ray_facets(f"Cardinal Ray {idx + 1}", angle, r_cardinal, r_waist_c, lit_side_ccw=(angle in (-90.0, 180.0)))

    # Central raised solar boss
    r_boss_ring = 46.0
    boss_ring = add(
        "shape.ellipse",
        {"x": CX - r_boss_ring, "y": CY - r_boss_ring, "width": r_boss_ring * 2, "height": r_boss_ring * 2},
    )
    style_shape(
        boss_ring,
        name="Central Solar Bevel",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [CX, CY - r_boss_ring],
                "end": [CX, CY + r_boss_ring],
                "stops": [
                    {"offset": 0.0, "color": "#FCEBB6"},
                    {"offset": 0.5, "color": "#C99A3B"},
                    {"offset": 1.0, "color": "#6E4D14"},
                ],
            }
        },
        stroke={"color": "#4B330C"},
        stroke_opts={"weight": 2.0},
    )

    r_boss_core = 34.0
    boss_core = add(
        "shape.ellipse",
        {"x": CX - r_boss_core, "y": CY - r_boss_core, "width": r_boss_core * 2, "height": r_boss_core * 2},
    )
    style_shape(
        boss_core,
        name="Central Solar Cabochon",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [CX, CY - 12],
                "end": [CX, CY + 36],
                "stops": [
                    {"offset": 0.0, "color": "#FFF9E8"},
                    {"offset": 0.45, "color": "#E8C56E"},
                    {"offset": 1.0, "color": "#A87D2A"},
                ],
            }
        },
        stroke={"none": True},
    )

    # Subtle specular glint on upper half of the cabochon
    glint = add(
        "shape.ellipse",
        {"x": CX - 16, "y": CY - 24, "width": 32, "height": 16},
    )
    style_shape(
        glint,
        name="Solar Cabochon Glint",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [CX, CY - 24],
                "end": [CX, CY - 8],
                "stops": [
                    {"offset": 0.0, "color": "#FFFFFF", "opacity": 0.72},
                    {"offset": 1.0, "color": "#FFFFFF", "opacity": 0.0},
                ],
            }
        },
        stroke={"none": True},
    )

    # 11. Subtle upper observatory crystal sheen (keeps glyphs high-contrast)
    lens_sheen = add(
        "shape.ellipse",
        {"x": CX - 246, "y": 208, "width": 492, "height": 220},
    )
    style_shape(
        lens_sheen,
        name="Observatory Crystal Sheen",
        fill={
            "gradient": {
                "kind": "linear",
                "start": [CX, 208],
                "end": [CX, 428],
                "stops": [
                    {"offset": 0.0, "color": "#FFFFFF", "opacity": 0.11},
                    {"offset": 0.65, "color": "#FFFFFF", "opacity": 0.03},
                    {"offset": 1.0, "color": "#FFFFFF", "opacity": 0.0},
                ],
            }
        },
        stroke={"none": True},
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
