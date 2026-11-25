"""Read screen geometry and native prototype navigation from the sole FigCraft master.

Do not build or overwrite the master from SVG/JSON/Python source.
All external previews and gallery metadata are derived from this document.
"""
from __future__ import annotations

import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MASTER = ROOT / "design" / "motaa-desktop.figcraft"


def load_document() -> dict:
    competing = sorted(
        candidate.relative_to(ROOT).as_posix()
        for candidate in ROOT.rglob("*.figcraft")
        if candidate != MASTER and ".git" not in candidate.parts
    )
    if competing:
        raise ValueError(
            "Multiple FigCraft documents detected; only design/motaa-desktop.figcraft "
            f"is permitted: {competing}"
        )
    with zipfile.ZipFile(MASTER) as archive:
        if archive.testzip() is not None:
            raise ValueError("Corrupt FigCraft archive")
        document = json.loads(archive.read("document.json"))
    if document.get("format") != "figcraft":
        raise ValueError(f"Unexpected FigCraft format: {document.get('format')}")
    return document


def screen_layers(nodes, frame, *, visible_only=False):
    """Resolve composite action hit regions and their labels into frame coordinates."""
    def nested_actions(node, dx=0, dy=0):
        if visible_only and not node.get("visible", True):
            return
        x, y = node.get("transform", [1, 0, 0, 1, 0, 0])[4:]
        ax, ay = dx + x, dy + y
        if node.get("interactions"):
            yield {**node, "transform": [1, 0, 0, 1, ax, ay]}
        for child_id in node.get("children", []):
            yield from nested_actions(nodes[str(child_id)], ax, ay)

    for child_id in frame.get("children", []):
        node = nodes[str(child_id)]
        if visible_only and not node.get("visible", True):
            continue
        main = nodes.get(str(node.get("component")), {})
        if node["type"] != "instance" or not main.get("name", "").startswith(
            ("MOTAA / Action button /", "MOTAA / Privacy /")
        ):
            yield node
            x, y = node.get("transform", [1, 0, 0, 1, 0, 0])[4:]
            for descendant_id in node.get("children", []):
                yield from nested_actions(nodes[str(descendant_id)], x, y)
            continue
        x, y = node.get("transform", [1, 0, 0, 1, 0, 0])[4:]
        for layer_id in node["children"]:
            layer = nodes[str(layer_id)]
            if visible_only and not layer.get("visible", True):
                continue
            matrix = list(layer.get("transform", [1, 0, 0, 1, 0, 0]))
            matrix[4] += x
            matrix[5] += y
            yield {**layer, "transform": matrix}


def extract_screens() -> list[dict]:
    document = load_document()
    nodes = document["nodes"]
    frame_names = {}
    for page_id in document["pages"]:
        page = nodes[str(page_id)]
        frames = page.get("children", [])
        if len(frames) != 1:
            raise ValueError(f"Expected one main frame on page {page['name']}")
        frame = nodes[str(frames[0])]
        if frame.get("type", "").lower() != "frame":
            raise ValueError(f"Expected a frame on page {page['name']}")
        frame_names[frame["id"]] = page["name"]

    screens = []
    seen_names = set()
    for page_id in document["pages"]:
        page = nodes[str(page_id)]
        frame = nodes[str(page["children"][0])]
        if page["name"] in seen_names:
            raise ValueError(f"Duplicate FigCraft page: {page['name']}")
        seen_names.add(page["name"])
        canvas = [int(frame["width"]), int(frame["height"])]
        expected = (
            [1280, 960] if page["name"] == "87 Desktop 1280" else
            [1024, 900] if page["name"] == "88 Desktop 1024" else
            [1280, 800] if page["name"] == "108 Compact chart - 1280x800" else
            [1440, 960]
        )
        if canvas != expected:
            raise ValueError(f"Unexpected canvas on {page['name']}: {canvas}; expected {expected}")
        links = []
        for node in screen_layers(nodes, frame, visible_only=True):
            for interaction in node.get("interactions", []):
                for action in interaction.get("actions", []):
                    if action.get("type") != "navigate":
                        continue
                    destination = frame_names.get(action["dest"])
                    if destination is None:
                        raise ValueError(f"Broken navigation on {page['name']}: {action}")
                    transform = node.get("transform", [1, 0, 0, 1, 0, 0])
                    if transform[:4] != [1.0, 0.0, 0.0, 1.0]:
                        raise ValueError(f"Unsupported rotated hotspot: {node['name']}")
                    links.append({
                        "x": transform[4],
                        "y": transform[5],
                        "width": node["width"],
                        "height": node["height"],
                        "label": node["name"].removeprefix("Navigate / "),
                        "destination": destination,
                    })
        screens.append({
            "name": page["name"],
            "frame": frame["name"],
            "canvas": canvas,
            "image": f"previews/{frame['name']}.png",
            "links": links,
        })

    return screens


def main() -> None:
    screens = extract_screens()
    print(json.dumps({
        "master": str(MASTER.relative_to(ROOT)),
        "screens": len(screens),
        "prototype_links": sum(len(screen["links"]) for screen in screens),
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()
