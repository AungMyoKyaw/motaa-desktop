#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
android=false
production=false
background=false
while (($#)); do
  case $1 in
    --android) android=true ;;
    --production) production=true ;;
    --background) background=true ;;
    *) printf 'Usage: %s [--production] [--android] [--background]\n' "$0" >&2; exit 2 ;;
  esac
  shift
done
master=design/motaa-desktop.figcraft
diagram=docs/gpui/motaa-classes.puml
evidence=$(mktemp -d "${TMPDIR:-/tmp}/motaa-ralph.XXXXXX")
printf 'Evidence directory: %s\n' "$evidence"

shasum -a 256 "$master" > "$evidence/master.sha256"
figcraft-cli info "$master" --json > "$evidence/master.json"
jq -e --slurpfile design "$evidence/master.json" '
  ($design[0].pages | map(.name)) as $pages |
  (.routes | map(.figcraft_page_name)) as $entries |
  ($entries | length == 5 and (unique | length == 5)) and
  ($entries - $pages == []) and
  (.recovered_modes | length == 11) and
  (.educational_pages | length == 10) and
  ($design[0].missingFonts | length == 0)
' design/APP_ROUTES.json > "$evidence/routes.ok"
jq -r '.routes[].figcraft_page_name' design/APP_ROUTES.json > "$evidence/entries.txt"
while IFS= read -r page; do
  figcraft-cli render --page "$page" --scale 0.5 \
    --out "$evidence/${page%% *}.png" \
    --automation-read-root "$PWD" --automation-write-root "$evidence" "$master"
done < "$evidence/entries.txt"

plantuml --check-syntax "$diagram"
plantuml --output-dir "$evidence" --svg "$diagram"
python3 - "$evidence/motaa-classes.svg" <<'PY'
import sys
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
assert root.tag == "{http://www.w3.org/2000/svg}svg", root.tag
assert "WorkspaceSession" in "".join(root.itertext())
PY
shasum -a 256 -c "$evidence/master.sha256"

run_gate() {
  local gate=$1
  shift
  printf 'Running %s\n' "$gate"
  if "$@" 2>&1 | tee "$evidence/$gate.log"; then
    printf '%s passed\n' "$gate" | tee -a "$evidence/status.txt"
  else
    printf '%s failed. See %s/%s.log\n' "$gate" "$evidence" "$gate" | tee -a "$evidence/status.txt" >&2
    return 1
  fi
}

printf '%s\n' 'Design and class diagram passed.' | tee "$evidence/status.txt"
run_gate format cargo fmt --all -- --check
run_gate clippy cargo clippy --locked --workspace --all-targets -- -D warnings
run_gate tests cargo test --locked --workspace
run_gate build cargo build --locked --workspace
if $background; then
  printf '%s\n' 'Native smoke skipped in background mode.' | tee -a "$evidence/status.txt"
else
  run_gate native-smoke /usr/bin/perl -e 'alarm shift; exec @ARGV or die $!' 30 \
    "${CARGO_TARGET_DIR:-target}/debug/motaa-desktop" --smoke
  for workspace in Charts Profiles Calculations 'Time & Analysis' Advanced; do
    if ! rg -Fx "Rendered $workspace" "$evidence/native-smoke.log" > /dev/null; then
      printf 'Missing native route evidence for %s\n' "$workspace" | tee -a "$evidence/status.txt" >&2
      exit 1
    fi
  done
  if [[ $(rg -c '^Exercised chart mode ' "$evidence/native-smoke.log") != 12 ]]; then
    printf '%s\n' 'Missing chart mode smoke evidence.' | tee -a "$evidence/status.txt" >&2
    exit 1
  fi
  for tool in 'Ephemeris calendar' 'Heliocentric model' 'KP subdivisions' 'Sky map · 2D' 'Sky map · 3D' Ashtakavarga 'Divisional charts' 'Electional timing' 'Transit events' 'Daily Tatkalika' 'Nakshatra groups' Eclipses 'Strength timeline' 'Yoga / candidate times' Sarvatobhadra 'Satapati name wheel' Interpretations 'Ten pillars / Ayu Moon'; do
    rg -Fx "Exercised advanced tool $tool" "$evidence/native-smoke.log" > /dev/null
  done
  rg -Fx 'Verified forward and reverse keyboard focus' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified keyboard control activation' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified profile errors, unknown time, and undo' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified backup review and invalid-file handling' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified MUNP modes and chart palettes' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified workspace keyboard shortcuts' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified transit and strength workers' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified independent reference location' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified Rahu 28-division manual reference' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified circular chart detail navigation' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified compact chart resize' "$evidence/native-smoke.log" > /dev/null
  rg -Fx 'Verified leap-month ephemeris' "$evidence/native-smoke.log" > /dev/null
fi
shasum -a 256 -c "$evidence/master.sha256"
if $android; then
  : "${MOTAA_ANDROID_SERIAL:?Set MOTAA_ANDROID_SERIAL to the isolated emulator serial}"
  : "${MOTAA_ANDROID_D8:?Set MOTAA_ANDROID_D8 to the Android build-tools d8 path}"
  run_gate android-capture python3 scripts/capture-apk-oracle.py \
    --serial "$MOTAA_ANDROID_SERIAL" --d8 "$MOTAA_ANDROID_D8" \
    --include-edge-cases --output "$evidence/android-runtime.json" \
    --profile-database "$evidence/android-profiles.sqlite" \
    --calendar "$evidence/android-calendar.json" --myanmar "$evidence/android-myanmar.json" \
    --rahu "$evidence/android-rahu.json"
  run_gate android-comparison env MOTAA_RUNTIME_ORACLE="$evidence/android-runtime.json" \
    cargo test --locked -p motaa-engine --test apk_runtime \
      compare_reference_to_shipped_motaa_runtime -- --nocapture
  run_gate android-calendar env MOTAA_ANDROID_CALENDAR="$evidence/android-calendar.json" \
    cargo test --locked -p motaa-engine --test apk_runtime calendar_matches_shipped_motaa_runtime
  run_gate android-myanmar env MOTAA_ANDROID_MYANMAR="$evidence/android-myanmar.json" \
    cargo test --locked -p motaa-engine --test apk_runtime \
      myanmar_calendar_and_observances_match_shipped_motaa_runtime
  run_gate android-rahu env MOTAA_ANDROID_RAHU="$evidence/android-rahu.json" \
    cargo test --locked --bin motaa-desktop rahu_bands_match_shipped_motaa_runtime
  run_gate android-migration env MOTAA_ANDROID_PROFILE_DATABASE="$evidence/android-profiles.sqlite" \
    cargo test --locked --bin motaa-desktop reads_android_sqlite_and_restores_operable_profiles
  if [[ -n "${MOTAA_ANDROID_ORIGINAL_SAVED_DATABASE:-}" ]]; then
    run_gate android-original-saved-profile cargo test --locked --bin motaa-desktop \
      reads_saved_profiles_written_by_v31_1 -- --ignored
  fi
fi
jq -r '.criteria[] | select(.state != "verified") | "\(.id): \(.next)"' \
  docs/gpui/production-criteria.json > "$evidence/remaining.txt"
if [[ -s "$evidence/remaining.txt" ]]; then
  printf 'Production criteria remain open. See %s/remaining.txt\n' "$evidence" | tee -a "$evidence/status.txt"
  if $production; then
    cat "$evidence/remaining.txt"
    exit 1
  fi
fi
if $background && $production; then
  printf '%s\n' 'Production verification requires a native smoke run.' | tee -a "$evidence/status.txt"
  exit 1
fi
printf '%s\n' 'Current implementation gates passed.' | tee -a "$evidence/status.txt"
