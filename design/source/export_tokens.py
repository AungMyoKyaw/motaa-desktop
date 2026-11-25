"""Export semantic CSS tokens and chart display themes from the sole FigCraft master.

This file is a design artifact exporter, not an application build system.
Native variable collections are preserved:
- "MOTAA / Primitives" contains raw foundation values.
- "Gold Heritage / UI" resolves semantic and component aliases in light appearance only.
- "MOTAA" contains four astrology chart-display themes. These do not change UI appearance.
- "Birth details privacy" remains native interaction state, not a CSS palette.
"""

from __future__ import annotations

from figcraft_master import ROOT, load_document


def _css_value(value: dict, *, unit: str = "px") -> str:
    if value["type"] == "color":
        color = value["value"]
        channels = "rgb" if color.get("a", 1) == 1 else "rgba"
        return "#" + "".join(f"{round(color[c] * 255):02X}" for c in channels)
    if value["type"] == "number":
        return f'{value["value"]:g}{unit}'
    raise ValueError(f'Unsupported CSS value type: {value["type"]}')


def _resolve_value(data: dict, variable: dict, mode_id: str) -> dict:
    """Follow native aliases across collections, including single-mode primitives."""
    variables = {entry["id"]: entry for entry in data["variables"]}
    collections = {entry["id"]: entry for entry in data["collections"]}
    seen = set()
    current = variable
    current_mode = mode_id
    while True:
        if current["id"] in seen:
            raise ValueError(f'Circular variable alias: {variable["name"]}')
        seen.add(current["id"])
        value = current["values"][current_mode]
        if value["type"] != "alias":
            return value
        current = variables[value["value"]]
        target_modes = collections[current["collection"]]["modes"]
        if current_mode not in current["values"]:
            if len(target_modes) != 1:
                raise ValueError(f'Cannot resolve cross-mode alias for {current["name"]}')
            current_mode = target_modes[0]["id"]


def _tokens_for_collection(data: dict, collection: dict, mode: dict) -> list[str]:
    result: list[str] = []
    for variable in data["variables"]:
        if variable["collection"] != collection["id"]:
            continue
        if variable["resolvedType"] not in ("color", "number"):
            continue
        values = variable["values"]
        if mode["id"] not in values:
            raise ValueError(
                f'Missing {mode["name"]} value for {variable["name"]}'
            )
        token = "--" + variable["name"].replace("/", "-")
        value = _resolve_value(data, variable, mode["id"])
        unit = "ms" if variable["name"].startswith("duration/") else "px"
        result.append(f'  {token}: {_css_value(value, unit=unit)};')
    return result


def generate() -> str:
    data = load_document()["variables"]
    collections = {entry["name"]: entry for entry in data["collections"]}
    expected_names = {"MOTAA", "Gold Heritage / UI", "MOTAA / Primitives"}
    missing = expected_names - collections.keys()
    if missing:
        raise ValueError(f"Missing native palette collections: {sorted(missing)}")

    chart = collections["MOTAA"]
    chart_modes = chart["modes"]
    if [m["name"] for m in chart_modes] != [
        "Light", "Pink", "Blue", "Monochrome"
    ]:
        raise ValueError("The four source chart themes must stay intact")

    ui = collections["Gold Heritage / UI"]
    if [m["name"] for m in ui["modes"]] != ["Light"]:
        raise ValueError(
            "Gold Heritage v3 is light-first; dark mode is not authorized"
        )

    primitives = collections["MOTAA / Primitives"]
    if [m["name"] for m in primitives["modes"]] != ["Light"]:
        raise ValueError("MOTAA primitives must retain a single Light mode")

    lines = [
        "/* GENERATED from design/motaa-desktop.figcraft. Do not edit directly. */",
        "/* Gold Heritage v3 desktop appearance is light-only in this version. */",
        "/* Four original MOTAA themes affect chart data, not desktop chrome. */",
        ":root {",
    ]
    lines.extend(_tokens_for_collection(data, primitives, primitives["modes"][0]))
    lines.extend(_tokens_for_collection(data, ui, ui["modes"][0]))
    lines.extend(_tokens_for_collection(data, chart, chart_modes[0]))
    lines.extend(["}", ""])

    for mode in chart_modes[1:]:
        name = mode["name"].lower()
        # Preserve older gallery selectors while exposing a clearer chart-only name.
        lines.append(
            f':root[data-motaa-chart-theme="{name}"], '
            f':root[data-motaa-theme="{name}"] {{'
        )
        lines.extend(_tokens_for_collection(data, chart, mode))
        lines.extend(["}", ""])
    return "\n".join(lines)


if __name__ == "__main__":
    target = ROOT / "design" / "tokens.css"
    target.write_text(generate())
    print(
        f"Exported {target.relative_to(ROOT)} from canonical light UI "
        "and four independent chart-display themes"
    )
