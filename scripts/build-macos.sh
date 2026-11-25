#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ $(uname -s) != Darwin ]]; then
  printf '%s\n' 'The macOS app bundle requires macOS.' >&2
  exit 1
fi

signing_identity=${MOTAA_DEVELOPER_ID:-}
dev_identity=${MOTAA_DEVELOPMENT_ID:-}
if [[ -n $signing_identity && -n $dev_identity ]]; then
  printf '%s\n' 'Set only one of MOTAA_DEVELOPER_ID or MOTAA_DEVELOPMENT_ID.' >&2
  exit 2
fi
if [[ -n $signing_identity ]]; then
  if [[ $signing_identity != 'Developer ID Application: '* ]]; then
    printf '%s\n' 'MOTAA_DEVELOPER_ID must be a Developer ID Application identity name.' >&2
    exit 2
  fi
  if ! security find-identity -v -p codesigning | grep -F -- "\"$signing_identity\"" >/dev/null; then
    printf '%s\n' 'The requested Developer ID Application identity is unavailable.' >&2
    exit 2
  fi
elif [[ -n $dev_identity ]]; then
  if [[ $dev_identity != 'Apple Development: '* ]]; then
    printf '%s\n' 'MOTAA_DEVELOPMENT_ID must be an Apple Development identity name.' >&2
    exit 2
  fi
  if ! security find-identity -v -p codesigning | grep -F -- "\"$dev_identity\"" >/dev/null; then
    printf '%s\n' 'The requested Apple Development identity is unavailable.' >&2
    exit 2
  fi
fi

target_dir=${CARGO_TARGET_DIR:-target}
case $target_dir in
  /*) ;;
  *) target_dir="$PWD/$target_dir" ;;
esac
version=$(cargo metadata --format-version 1 --no-deps --locked |
  jq -er '.packages[] | select(.name == "motaa-desktop") | .version')
cargo build --release --locked --bin motaa-desktop
mkdir -p "$target_dir/bundle"
stage=$(mktemp -d "$target_dir/bundle/motaa-stage.XXXXXX")
trap 'rm -rf "$stage"' EXIT
app="$stage/MOTAA Desktop.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
install -m 755 "$target_dir/release/motaa-desktop" "$app/Contents/MacOS/motaa-desktop"
if [[ -f assets/AppIcon.icns ]]; then
  install -m 644 assets/AppIcon.icns "$app/Contents/Resources/AppIcon.icns"
fi
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key><string>com.motaa.desktop</string>
  <key>CFBundleDisplayName</key><string>MOTAA Desktop</string>
  <key>CFBundleName</key><string>MOTAA Desktop</string>
  <key>CFBundleExecutable</key><string>motaa-desktop</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
if [[ -n $signing_identity ]]; then
  codesign --force --options runtime --timestamp --sign "$signing_identity" "$app"
  codesign --verify --strict -R='anchor apple generic and certificate leaf[field.1.2.840.113635.100.6.1.13] exists' "$app"
elif [[ -n $dev_identity ]]; then
  codesign --force --options runtime --timestamp --sign "$dev_identity" "$app"
  codesign --verify --strict -R='anchor apple generic and certificate leaf[field.1.2.840.113635.100.6.1.12] exists' "$app"
else
  codesign --force --sign - "$app"
fi
codesign --verify --deep --strict "$app"
perl -e 'alarm shift; exec @ARGV or die $!' 45 "$app/Contents/MacOS/motaa-desktop" --smoke
output=$(mktemp -d "$target_dir/bundle/motaa-$version.XXXXXX")
mv "$app" "$output/"
printf '%s\n' "$output/MOTAA Desktop.app"
