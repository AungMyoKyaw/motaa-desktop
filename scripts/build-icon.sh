#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v vectorcraft-cli >/dev/null 2>&1; then
  printf '%s\n' 'vectorcraft-cli is required to generate the MOTAA Desktop icon.' >&2
  exit 1
fi

mkdir -p assets/icon site/assets packaging/windows/assets
steps_json=$(mktemp "${TMPDIR:-/tmp}/motaa-icon-steps.XXXXXX.json")
run_log=$(mktemp "${TMPDIR:-/tmp}/motaa-icon-run.XXXXXX.jsonl")
iconset_dir=$(mktemp -d "${TMPDIR:-/tmp}/motaa-iconset.XXXXXX")
trap 'rm -rf "$steps_json" "$run_log" "$iconset_dir"' EXIT

python3 scripts/generate-icon-steps.py "$steps_json"

vectorcraft-cli --safe-mode run \
  --commands "$steps_json" \
  --export assets/icon/motaa-icon.vectorcraft \
  --export assets/icon/motaa-icon.svg \
  --export assets/icon/motaa-icon-1024.png \
  --scale 1 > "$run_log"

python3 - "$run_log" <<'PY'
import json
import sys
from pathlib import Path

errors = []
for i, raw in enumerate(Path(sys.argv[1]).read_text(encoding="utf-8").splitlines(), start=1):
    if not raw.strip():
        continue
    rec = json.loads(raw)
    if rec.get("ok") is False or rec.get("error"):
        errors.append(f"line {i}: {rec}")
    for w in rec.get("warnings") or []:
        errors.append(f"line {i} warning: {w}")
if errors:
    raise SystemExit("VectorCraft run reported errors/warnings:\n" + "\n".join(errors))
PY

# Verify native document opens cleanly
vectorcraft-cli info assets/icon/motaa-icon.vectorcraft >/dev/null

# Generate all derived macOS, in-app, web, and Windows Store/MSIX raster icons
python3 - "$iconset_dir" <<'PY'
import sys
from pathlib import Path
from PIL import Image

iconset_dir = Path(sys.argv[1])
master = Image.open("assets/icon/motaa-icon-1024.png").convert("RGBA")
if master.size != (1024, 1024):
    raise SystemExit(f"Expected 1024x1024 master, got {master.size}")

# 1. In-app GPUI icon (256x256) and landing page favicon/icon
app_256 = master.resize((256, 256), Image.Resampling.LANCZOS)
app_256.save("assets/app-icon-256.png", format="PNG")
app_256.save("site/assets/app-icon-256.png", format="PNG")

# 2. macOS iconset PNGs (for iconutil)
appiconset = iconset_dir / "AppIcon.iconset"
appiconset.mkdir(parents=True, exist_ok=True)
for size in (16, 32, 128, 256, 512):
    master.resize((size, size), Image.Resampling.LANCZOS).save(
        appiconset / f"icon_{size}x{size}.png", format="PNG"
    )
    double = size * 2
    master.resize((double, double), Image.Resampling.LANCZOS).save(
        appiconset / f"icon_{size}x{size}@2x.png", format="PNG"
    )

# 3. Windows Store / MSIX assets
# Tight crop around the 824x824 squircle + shadow so Windows tiles/taskbar fill cleanly
win_tight = master.crop((76, 76, 948, 948))      # 872x872 for small taskbar/store logos
win_shadow = master.crop((64, 64, 960, 960))     # 896x896 preserving full outer shadow

win_tight.resize((44, 44), Image.Resampling.LANCZOS).save(
    "packaging/windows/assets/Square44x44Logo.png", format="PNG"
)
win_tight.resize((50, 50), Image.Resampling.LANCZOS).save(
    "packaging/windows/assets/StoreLogo.png", format="PNG"
)
win_tight.save(
    "assets/app-icon.ico",
    format="ICO",
    sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (256, 256)],
)

def paste_centered(canvas_size: tuple[int, int], icon_size: int, bg_rgba: tuple[int, int, int, int]) -> Image.Image:
    canvas = Image.new("RGBA", canvas_size, bg_rgba)
    resized = win_shadow.resize((icon_size, icon_size), Image.Resampling.LANCZOS)
    x = (canvas_size[0] - icon_size) // 2
    y = (canvas_size[1] - icon_size) // 2
    canvas.alpha_composite(resized, (x, y))
    return canvas

paste_centered((150, 150), 138, (0, 0, 0, 0)).save(
    "packaging/windows/assets/Square150x150Logo.png", format="PNG"
)
paste_centered((310, 310), 284, (0, 0, 0, 0)).save(
    "packaging/windows/assets/Square310x310Logo.png", format="PNG"
)
paste_centered((300, 300), 280, (0, 0, 0, 0)).save(
    "packaging/windows/assets/StoreIcon300.png", format="PNG"
)

# Wide tile & splash screen use AppxManifest.xml BackgroundColor #F8F5EE
bg_cream = (0xF8, 0xF5, 0xEE, 0xFF)
paste_centered((310, 150), 124, bg_cream).save(
    "packaging/windows/assets/Wide310x150Logo.png", format="PNG"
)
paste_centered((620, 300), 228, bg_cream).save(
    "packaging/windows/assets/SplashScreen.png", format="PNG"
)
PY

# Build native macOS AppIcon.icns when running on macOS
if [[ $(uname -s) == Darwin ]]; then
  if ! iconutil -c icns "$iconset_dir/AppIcon.iconset" -o assets/AppIcon.icns 2>/dev/null; then
    python3 -c '
from PIL import Image
img = Image.open("assets/icon/motaa-icon-1024.png").convert("RGBA")
img.save("assets/AppIcon.icns", format="ICNS", sizes=[(16,16),(32,32),(64,64),(128,128),(256,256),(512,512),(1024,1024)])
'
  fi
fi

printf '%s\n' \
  'assets/icon/motaa-icon.vectorcraft' \
  'assets/icon/motaa-icon.svg' \
  'assets/icon/motaa-icon-1024.png' \
  'assets/app-icon-256.png' \
  'site/assets/app-icon-256.png' \
  'assets/AppIcon.icns' \
  'packaging/windows/assets/Square44x44Logo.png' \
  'packaging/windows/assets/StoreLogo.png' \
  'packaging/windows/assets/Square150x150Logo.png' \
  'packaging/windows/assets/Square310x310Logo.png' \
  'packaging/windows/assets/StoreIcon300.png' \
  'packaging/windows/assets/Wide310x150Logo.png' \
  'packaging/windows/assets/SplashScreen.png'
