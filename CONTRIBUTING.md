# Contributing to MOTAA Desktop

Thank you for contributing to MOTAA Desktop.

## Development Setup

Requirements:
- Stable Rust toolchain (`rustup`)
- **macOS**: Xcode with the Metal toolchain (`xcodebuild -downloadComponent MetalToolchain`)
- **Linux (Ubuntu 24.04)**: `libasound2-dev libfontconfig1-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev libxcb-shape0-dev libxcb-xfixes0-dev libvulkan-dev libx11-xcb-dev libudev-dev libclang-dev libssl-dev pkg-config`
- **Windows**: Visual Studio Build Tools and Windows SDK (`fxc.exe` shader compiler)

Run the app locally:
```sh
make preview   # Anonymous in-memory sample library
make dev       # Local SQLite profile library
```

## Verification Gates

Before opening a pull request, verify all checks pass cleanly:

```sh
make check     # cargo fmt --all -- --check && cargo clippy --locked --workspace --all-targets -- -D warnings
make test      # cargo test --locked --workspace
make smoke     # Headless GPUI window and interaction smoke test
```

If you touch `design/` assets or metadata, also run:
```sh
python3 -m unittest discover -s design/tests -v
```

## Guidelines

- Preserve traditional square and circular Myanmar chart geometry, English digits, and the 4 chart-only palettes.
- Keep `design/motaa-desktop.figcraft` as the single FigCraft master; never add duplicate `*.figcraft` files.
- Never commit personal birth data, local machine paths, or credentials.
- Keep commits focused and verify `make check` and `make test` after your final edit.
