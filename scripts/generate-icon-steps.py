#!/usr/bin/env python3
"""Generate VectorCraft CLI headless batch commands for the MOTAA Desktop icon.

Design system: Gold Heritage (DESIGN.md)
Canvas: 1024x1024 px (transparent background), macOS HIG 824x824 squircle tile.
Motif: Concentric 12-house Burmese horoscopic wheel + inscribed square/diamond
Zata geometry + radiant 8-point astral core + heritage planetary jewels.
"""

import json
import math
import sys
from pathlib import Path

CX = 512.0
CY = 512.0


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

    # 2. Outer macOS squircle tile (824x824 centered at 512,512; radius 185)
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
                "start": [512, 410],
                "end": [512, 960],
                "stops": [
                    {"offset": 0.0, "color": "#332A1F"},
                    {"offset": 0.58, "color": "#211B14"},
                    {"offset": 1.0, "color": "#14100C"},
                ],
            }
        },
        stroke={
            "gradient": {
                "kind": "linear",
                "angle": 135,
                "stops": [
                    {"offset": 0.0, "color": "#E8CA7E"},
                    {"offset": 0.5, "color": "#9B7630"},
                    {"offset": 1.0, "color": "#5E4518"},
                ],
            }
        },
        stroke_opts={"weight": 8, "align": "inside"},
    )

    # 3. Inner gold filigree bevel frame
    bevel = add(
        "shape.rectangle",
        {"x": 126, "y": 126, "width": 772, "height": 772, "radius": 162},
    )
    style_shape(
        bevel,
        name="Inner Gold Bevel",
        fill={"none": True},
        stroke={"color": "#C9A250"},
        stroke_opts={"weight": 3, "align": "inside"},
        opacity=45,
    )

    # 4. Subtle warm aura behind the wheel
    aura = add(
        "shape.ellipse",
        {"x": 172, "y": 172, "width": 680, "height": 680},
    )
    style_shape(
        aura,
        name="Celestial Aura",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 512],
                "end": [852, 512],
                "stops": [
                    {"offset": 0.0, "color": "#7C5B21", "opacity": 0.34},
                    {"offset": 0.65, "color": "#7C5B21", "opacity": 0.12},
                    {"offset": 1.0, "color": "#14100C", "opacity": 0.0},
                ],
            }
        },
        stroke={"none": True},
    )

    # 5. Outer zodiac ring (r = 304) and inner zodiac ring (r = 246)
    r_outer = 304
    r_inner = 246
    outer_ring = add(
        "shape.ellipse",
        {
            "x": CX - r_outer,
            "y": CY - r_outer,
            "width": r_outer * 2,
            "height": r_outer * 2,
        },
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
                    {"offset": 0.0, "color": "#F5DF9F"},
                    {"offset": 0.45, "color": "#CFA54E"},
                    {"offset": 1.0, "color": "#8E6926"},
                ],
            }
        },
        stroke_opts={"weight": 10},
    )

    inner_ring = add(
        "shape.ellipse",
        {
            "x": CX - r_inner,
            "y": CY - r_inner,
            "width": r_inner * 2,
            "height": r_inner * 2,
        },
    )
    style_shape(
        inner_ring,
        name="Zodiac Inner Ring",
        fill={"color": "#1C1711"},
        stroke={"color": "#B8903F"},
        stroke_opts={"weight": 5},
    )

    # 6. 12 zodiac house spokes between inner_ring and outer_ring
    for i in range(12):
        deg = i * 30.0
        x1, y1 = polar(r_inner, deg)
        x2, y2 = polar(r_outer, deg)
        spoke = add("shape.line", {"x1": x1, "y1": y1, "x2": x2, "y2": y2})
        style_shape(
            spoke,
            name=f"Zodiac Spoke {i + 1}",
            fill={"none": True},
            stroke={"color": "#CFA650"},
            stroke_opts={"weight": 5 if i % 3 == 0 else 3.5, "cap": "round"},
        )

    # 7. Burmese Zata dual-square geometry inscribed inside r_inner
    # Rotated 45-deg diamond first
    sq_half = 164
    diamond = add(
        "shape.rectangle",
        {
            "x": CX - sq_half,
            "y": CY - sq_half,
            "width": sq_half * 2,
            "height": sq_half * 2,
            "radius": 6,
            "rotation": 45,
        },
    )
    style_shape(
        diamond,
        name="Zata Diamond Frame",
        fill={"color": "#261F16"},
        stroke={"color": "#BA9240"},
        stroke_opts={"weight": 6, "join": "round"},
    )

    # Upright traditional Burmese Zata square
    square = add(
        "shape.rectangle",
        {
            "x": CX - sq_half,
            "y": CY - sq_half,
            "width": sq_half * 2,
            "height": sq_half * 2,
            "radius": 6,
            "rotation": 0,
        },
    )
    style_shape(
        square,
        name="Zata Square Frame",
        fill={"color": "#2B2319"},
        stroke={
            "gradient": {
                "kind": "linear",
                "angle": 135,
                "stops": [
                    {"offset": 0.0, "color": "#F3DC9B"},
                    {"offset": 0.5, "color": "#C99F49"},
                    {"offset": 1.0, "color": "#8E6A26"},
                ],
            }
        },
        stroke_opts={"weight": 7, "join": "round"},
    )

    # 8. Traditional Burmese Zata corner diagonals (signature 4-corner house lines)
    diag_inner = 78
    diag_outer = 160
    for idx, (sx, sy) in enumerate([(-1, -1), (1, -1), (1, 1), (-1, 1)], start=1):
        dline = add(
            "shape.line",
            {
                "x1": CX + sx * diag_inner,
                "y1": CY + sy * diag_inner,
                "x2": CX + sx * diag_outer,
                "y2": CY + sy * diag_outer,
            },
        )
        style_shape(
            dline,
            name=f"Zata Corner Diagonal {idx}",
            fill={"none": True},
            stroke={"color": "#C9A14E"},
            stroke_opts={"weight": 4.5, "cap": "round"},
            opacity=85,
        )

    # 9. Inner sanctuary circle (center chamber)
    r_core = 104
    core_circle = add(
        "shape.ellipse",
        {
            "x": CX - r_core,
            "y": CY - r_core,
            "width": r_core * 2,
            "height": r_core * 2,
        },
    )
    style_shape(
        core_circle,
        name="Central Sanctuary Ring",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 512],
                "end": [616, 512],
                "stops": [
                    {"offset": 0.0, "color": "#382C1B"},
                    {"offset": 0.72, "color": "#1E1811"},
                    {"offset": 1.0, "color": "#15110C"},
                ],
            }
        },
        stroke={"color": "#E8CA7A"},
        stroke_opts={"weight": 6},
    )

    # 10. Radiant 8-point Burmese/Astral star at center
    star_outer = add(
        "shape.star",
        {"cx": CX, "cy": CY, "radius1": 74, "radius2": 26, "points": 8, "rotation": 0},
    )
    style_shape(
        star_outer,
        name="Astral Eight-Point Star",
        fill={
            "gradient": {
                "kind": "radial",
                "start": [512, 512],
                "end": [586, 512],
                "stops": [
                    {"offset": 0.0, "color": "#FFF6D6"},
                    {"offset": 0.45, "color": "#E6C16C"},
                    {"offset": 1.0, "color": "#A37A2D"},
                ],
            }
        },
        stroke={"color": "#F8E7B4"},
        stroke_opts={"weight": 2, "join": "round"},
    )

    # Center luminous sun jewel
    r_sun = 17
    sun_dot = add(
        "shape.ellipse",
        {
            "x": CX - r_sun,
            "y": CY - r_sun,
            "width": r_sun * 2,
            "height": r_sun * 2,
        },
    )
    style_shape(
        sun_dot,
        name="Solar Core Jewel",
        fill={"color": "#FFF9E8"},
        stroke={"color": "#B38834"},
        stroke_opts={"weight": 3},
    )

    # 11. Four cardinal gold diamonds on the outer zodiac ring (N, E, S, W)
    for name, deg in [("North", -90.0), ("East", 0.0), ("South", 90.0), ("West", 180.0)]:
        px, py = polar(r_outer, deg)
        marker = add(
            "shape.polygon",
            {"cx": px, "cy": py, "radius": 14, "sides": 4, "rotation": 0},
        )
        style_shape(
            marker,
            name=f"Cardinal Marker {name}",
            fill={"color": "#F5DF9F"},
            stroke={"color": "#211B14"},
            stroke_opts={"weight": 3},
        )

    # 12. Three heritage planetary orbs on the zodiac belt (mid-ring r = 275)
    # Placed at balanced astronomical angles matching DESIGN.md accents
    r_belt = 275.0
    planets = [
        ("Planet Crimson", -45.0, "#D94436", 18.0),
        ("Planet Jade", 75.0, "#3F8E65", 16.0),
        ("Planet Sapphire", 195.0, "#4A7EC2", 16.0),
    ]
    for pname, deg, color, pr in planets:
        px, py = polar(r_belt, deg)
        orb = add(
            "shape.ellipse",
            {"x": px - pr, "y": py - pr, "width": pr * 2, "height": pr * 2},
        )
        style_shape(
            orb,
            name=pname,
            fill={"color": color},
            stroke={"color": "#F6E2A6"},
            stroke_opts={"weight": 4},
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
