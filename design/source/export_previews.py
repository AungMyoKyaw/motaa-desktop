"""Export review PNGs from design/motaa-desktop.figcraft (never from staging)."""
from __future__ import annotations

import argparse
import json
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from figcraft_master import MASTER, ROOT, extract_screens


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--page", action="append", help="Exact page name; repeat to render several")
    parser.add_argument("--jobs", type=int, default=2, help="Concurrent FigCraft CLI renders")
    args = parser.parse_args()
    if not 1 <= args.jobs <= 4:
        parser.error("--jobs must be between 1 and 4")
    screens = extract_screens()
    if args.page:
        unknown = set(args.page) - {s["name"] for s in screens}
        if unknown:
            parser.error(f"Unknown FigCraft page(s): {sorted(unknown)}")
        screens = [s for s in screens if s["name"] in args.page]

    def render(screen: dict) -> dict:
        target = ROOT / "design" / screen["image"]
        target.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["figcraft-cli", "render", str(MASTER), "--page", screen["name"], "--out", str(target)],
            text=True, capture_output=True, check=True,
        )
        if not target.is_file() or not target.stat().st_size:
            raise RuntimeError(f"Missing or empty export: {target}")
        return {"page": screen["name"], "path": str(target.relative_to(ROOT)), "bytes": target.stat().st_size}

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = list(pool.map(render, screens))
    print(json.dumps({"master": str(MASTER.relative_to(ROOT)), "exported": len(results), "results": results}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
