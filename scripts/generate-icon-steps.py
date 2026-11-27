#!/usr/bin/env python3
"""Generate VectorCraft CLI headless batch commands for the MOTAA Desktop icon.

Design system: Gold Heritage Light / Observatory Manuscript (DESIGN.md)
Inspiration: Classic MOTAA 12-house astrological wheel with the 7 classical planetary
glyphs in their ruling signs (Aries/Mars 12h, Aquarius/Saturn 2h, Sagittarius/Jupiter 4h,
Libra/Venus 6h, Virgo/Mercury 7h, Leo/Sun 8h, Cancer/Moon 9h) and centered clean geometric sans MOTAA text.
Canvas: 1024x1024 px (transparent background), macOS HIG 824x824 squircle tile.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

CX = 512.0
CY = 512.0

GLYPH_GOLD = "#6E4E17"


def polar(r: float, deg: float) -> tuple[float, float]:
    rad = math.radians(deg)
    return (round(CX + r * math.cos(rad), 2), round(CY + r * math.sin(rad), 2))


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

    def add_glyph_path(name: str, d: str, weight: float = 9.0, fill: dict | None = None) -> str:
        pid = add("path.create", {"d": d})
        style_shape(
            pid,
            name=name,
            fill=fill if fill is not None else {"none": True},
            stroke={"color": GLYPH_GOLD},
            stroke_opts={"weight": weight, "cap": "round", "join": "round"},
        )
        return pid

    def add_glyph_circle(
        name: str,
        cx: float,
        cy: float,
        r: float,
        weight: float = 9.0,
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
                fill={"color": GLYPH_GOLD},
                stroke={"none": True},
            )
        else:
            style_shape(
                eid,
                name=name,
                fill={"none": True},
                stroke={"color": GLYPH_GOLD},
                stroke_opts={"weight": weight, "cap": "round", "join": "round"},
            )
        return eid

    def add_glyph_line(
        name: str,
        x1: float,
        y1: float,
        x2: float,
        y2: float,
        weight: float = 9.0,
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
            stroke={"color": GLYPH_GOLD},
            stroke_opts={"weight": weight, "cap": "round", "join": "round"},
        )
        return lid

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
            "rasterEffectsPpi": 72,
        },
    )

    # 2. Outer macOS HIG squircle tile (824x824 centered at 512,512; radius 185)
    # Warm ivory parchment surface with burnished antique-gold metallic rim
    tile = add(
        "shape.rectangle",
        {"x": 100, "y": 100, "width": 824, "height": 824, "radius": 185},
    )
    style_shape(
        tile,
        name="MacOS Squircle Base",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 340],
                "end": [512, 960],
                "stops": [
                    {"offset": 0.0, "color": "#FFFEFC"},
                    {"offset": 0.56, "color": "#F7F2E6"},
                    {"offset": 1.0, "color": "#E6DBC2"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "angle": 135,
                "stops": [
                    {"offset": 0.0, "color": "#D9B462"},
                    {"offset": 0.48, "color": "#9E772B"},
                    {"offset": 1.0, "color": "#694A17"},
                ],
            }
        },
        stroke_opts={"weight": 10, "align": "inside"},
    )

    # 3. Inner gold filigree bevel frame
    bevel = add(
        "shape.rectangle",
        {"x": 124, "y": 124, "width": 776, "height": 776, "radius": 164},
    )
    style_shape(
        bevel,
        name="Inner Gold Bevel",
        fill={"none": True},
        stroke={"color": "#B8903D"},
        stroke_opts={"weight": 2.5, "align": "inside"},
        opacity=42,
    )

    # 4. Subtle warm sunlit gold aura behind the wheel
    aura = add(
        "shape.ellipse",
        {"x": CX - 368, "y": CY - 368, "width": 736, "height": 736},
    )
    style_shape(
        aura,
        name="Celestial Aura",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 512],
                "end": [880, 512],
                "stops": [
                    {"offset": 0.0, "color": "#C9A14A", "opacity": 0.22},
                    {"offset": 0.68, "color": "#C9A14A", "opacity": 0.07},
                    {"offset": 1.0, "color": "#F7F2E6", "opacity": 0.0},
                ],
            }
        },
        stroke={"none": True},
    )

    # 5. Subtle outer astrolabe accent hairline
    r_halo = 356.0
    halo_ring = add(
        "shape.ellipse",
        {"x": CX - r_halo, "y": CY - r_halo, "width": r_halo * 2, "height": r_halo * 2},
    )
    style_shape(
        halo_ring,
        name="Outer Astrolabe Hairline",
        fill={"none": True},
        stroke={"color": "#B38A38"},
        stroke_opts={"weight": 2.0},
        opacity=45,
    )

    # 6. Outer zodiac wheel fill (drawn BELOW the spokes so outer ring stroke masks spoke ends cleanly)
    r_outer = 340.0
    r_inner = 200.0
    r_mid = (r_outer + r_inner) / 2.0  # 270.0

    outer_fill = add(
        "shape.ellipse",
        {"x": CX - r_outer, "y": CY - r_outer, "width": r_outer * 2, "height": r_outer * 2},
    )
    style_shape(
        outer_fill,
        name="Zodiac Wheel Background",
        fill={"color": "#FCF9F2"},
        stroke={"none": True},
    )

    # 7. 12 offset radial house spokes (offset 15 deg so 12/3/6/9 o'clock sit centered in houses)
    for i in range(12):
        deg = -105.0 + i * 30.0
        x1, y1 = polar(r_inner, deg)
        x2, y2 = polar(r_outer, deg)
        spoke = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            spoke,
            name=f"Zodiac House Spoke {i + 1}",
            fill={"none": True},
            stroke={"color": "#9A742A"},
            stroke_opts={"weight": 8.0, "cap": "butt"},
        )

    # 7b. Outer zodiac ring stroke drawn ABOVE spokes so both spoke ends are cleanly masked
    outer_ring = add(
        "shape.ellipse",
        {"x": CX - r_outer, "y": CY - r_outer, "width": r_outer * 2, "height": r_outer * 2},
    )
    style_shape(
        outer_ring,
        name="Zodiac Outer Ring",
        fill={"none": True},
        stroke={
            "gradient": {
                "kind": "linear",
                "angle": 125,
                "stops": [
                    {"offset": 0.0, "color": "#C79E46"},
                    {"offset": 0.45, "color": "#8E6822"},
                    {"offset": 1.0, "color": "#5E4313"},
                ],
            }
        },
        stroke_opts={"weight": 12.5},
    )

    # 8. Inner sanctuary ring (r = 200)
    inner_ring = add(
        "shape.ellipse",
        {"x": CX - r_inner, "y": CY - r_inner, "width": r_inner * 2, "height": r_inner * 2},
    )
    style_shape(
        inner_ring,
        name="Central Sanctuary Ring",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 460],
                "end": [512, 712],
                "stops": [
                    {"offset": 0.0, "color": "#FFFEFA"},
                    {"offset": 0.65, "color": "#F5EDD8"},
                    {"offset": 1.0, "color": "#E5D5B0"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "angle": 135,
                "stops": [
                    {"offset": 0.0, "color": "#CAA149"},
                    {"offset": 0.5, "color": "#8E6822"},
                    {"offset": 1.0, "color": "#5C4112"},
                ],
            }
        },
        stroke_opts={"weight": 10.5},
    )

    # Subtle inner engraved ring inside sanctuary
    r_sanct_inner = 182.0
    sanct_bevel = add(
        "shape.ellipse",
        {
            "x": CX - r_sanct_inner,
            "y": CY - r_sanct_inner,
            "width": r_sanct_inner * 2,
            "height": r_sanct_inner * 2,
        },
    )
    style_shape(
        sanct_bevel,
        name="Sanctuary Inner Filigree",
        fill={"none": True},
        stroke={"color": "#B38A38"},
        stroke_opts={"weight": 2.0},
        opacity=45,
    )

    # 9. Seven classical planetary glyphs in their exact MOTAA ruling signs (upright, pure vector)
    # --- 12 o'clock (-90 deg): Mars (♂) ---
    mx, my = polar(r_mid, -90.0)
    add_glyph_circle("Glyph Mars Circle", mx - 9.5, my + 9.5, 20.5)
    add_glyph_line("Glyph Mars Shaft", mx + 5.5, my - 5.5, mx + 27.5, my - 27.5)
    add_glyph_path(
        "Glyph Mars Arrowhead",
        f"M {mx + 9.5:.2f} {my - 27.5:.2f} L {mx + 27.5:.2f} {my - 27.5:.2f} L {mx + 27.5:.2f} {my - 9.5:.2f}",
    )

    # --- 2 o'clock (-30 deg): Saturn (♄) ---
    sx, sy = polar(r_mid, -30.0)
    add_glyph_line("Glyph Saturn Stem", sx - 14, sy - 33, sx - 14, sy + 17)
    add_glyph_line("Glyph Saturn Crossbar", sx - 27, sy - 20, sx + 1, sy - 20)
    add_glyph_path(
        "Glyph Saturn Arch",
        (
            f"M {sx - 14:.2f} {sy - 1:.2f} "
            f"C {sx - 4:.2f} {sy - 16:.2f}, {sx + 17:.2f} {sy - 16:.2f}, {sx + 17:.2f} {sy + 2:.2f} "
            f"C {sx + 17:.2f} {sy + 15:.2f}, {sx + 10:.2f} {sy + 25:.2f}, {sx + 19:.2f} {sy + 31:.2f} "
            f"C {sx + 23:.2f} {sy + 33:.2f}, {sx + 28:.2f} {sy + 30:.2f}, {sx + 30:.2f} {sy + 25:.2f}"
        ),
    )

    # --- 4 o'clock (+30 deg): Jupiter (♃) ---
    jx, jy = polar(r_mid, 30.0)
    add_glyph_path(
        "Glyph Jupiter Hook",
        (
            f"M {jx - 24:.2f} {jy - 16:.2f} "
            f"C {jx - 23:.2f} {jy - 33:.2f}, {jx - 1:.2f} {jy - 35:.2f}, {jx - 1:.2f} {jy - 15:.2f} "
            f"C {jx - 1:.2f} {jy - 2:.2f}, {jx - 14:.2f} {jy + 7:.2f}, {jx - 26:.2f} {jy + 13:.2f} "
            f"L {jx + 26:.2f} {jy + 13:.2f}"
        ),
    )
    add_glyph_line("Glyph Jupiter Stem", jx + 12, jy - 20, jx + 12, jy + 33)

    # --- 6 o'clock (+90 deg): Venus (♀) ---
    vx, vy = polar(r_mid, 90.0)
    add_glyph_circle("Glyph Venus Circle", vx, vy - 14, 20.5)
    add_glyph_line("Glyph Venus Stem", vx, vy + 7, vx, vy + 36)
    add_glyph_line("Glyph Venus Crossbar", vx - 16, vy + 23, vx + 16, vy + 23)

    # --- 7 o'clock (+120 deg): Mercury (☿) ---
    ex, ey = polar(r_mid, 120.0)
    add_glyph_path(
        "Glyph Mercury Horns",
        f"M {ex - 17:.2f} {ey - 34:.2f} C {ex - 14:.2f} {ey - 20:.2f}, {ex + 14:.2f} {ey - 20:.2f}, {ex + 17:.2f} {ey - 34:.2f}",
    )
    add_glyph_circle("Glyph Mercury Circle", ex, ey - 4, 18.0)
    add_glyph_line("Glyph Mercury Stem", ex, ey + 14, ex, ey + 38)
    add_glyph_line("Glyph Mercury Crossbar", ex - 15, ey + 26, ex + 15, ey + 26)

    # --- 8 o'clock (+150 deg): Sun (☉) ---
    ux, uy = polar(r_mid, 150.0)
    add_glyph_circle("Glyph Sun Ring", ux, uy, 27.5)
    add_glyph_circle("Glyph Sun Core", ux, uy, 7.0, filled=True)

    # --- 9 o'clock (180 deg): Moon (☽) ---
    lx, ly = polar(r_mid, 180.0)
    add_glyph_path(
        "Glyph Moon Crescent",
        (
            f"M {lx - 15:.2f} {ly - 29:.2f} "
            f"C {lx + 11:.2f} {ly - 30:.2f}, {lx + 27:.2f} {ly - 14:.2f}, {lx + 27:.2f} {ly:.2f} "
            f"C {lx + 27:.2f} {ly + 14:.2f}, {lx + 11:.2f} {ly + 30:.2f}, {lx - 15:.2f} {ly + 29:.2f} "
            f"C {lx + 1:.2f} {ly + 21:.2f}, {lx + 8:.2f} {ly + 11:.2f}, {lx + 8:.2f} {ly:.2f} "
            f"C {lx + 8:.2f} {ly - 11:.2f}, {lx + 1:.2f} {ly - 21:.2f}, {lx - 15:.2f} {ly - 29:.2f} Z"
        ),
        weight=8.0,
    )

    # 10. Centered MOTAA text in clean geometric sans-serif (Avenir Next Bold)
    wordmark = add(
        "text.create",
        {
            "x": CX,
            "y": 539.0,
            "text": "MOTAA",
            "font": "Avenir Next",
            "style": "Bold",
            "size": 76.0,
            "color": "#543B10",
            "justify": "center",
        },
    )
    add(
        "text.setStyle",
        {
            "id": wordmark,
            "tracking": 55,
            "fill": "#543B10",
        },
    )
    add("object.setProps", {"id": wordmark, "name": "MOTAA Wordmark"})

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
