.PHONY: help dev preview smoke test check ralph icon bundle install open clean

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-10s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

dev: ## Launch MOTAA Desktop in development mode
	cargo run --locked

preview: ## Launch MOTAA Desktop with anonymous in-memory sample data
	cargo run --locked -- --preview

smoke: ## Run headless GPUI window smoke test
	cargo run --locked -- --smoke

test: ## Run all Rust workspace tests
	cargo test --locked --workspace

check: ## Run rustfmt check and strict clippy
	cargo fmt --all -- --check
	cargo clippy --locked --workspace --all-targets -- -D warnings

ralph: ## Run full verification and APK oracle parity loop
	bash docs/gpui/ralph.sh

icon: ## Regenerate VectorCraft icon and all macOS, web, and Windows app icon assets
	bash scripts/build-icon.sh

bundle: ## Build ad-hoc signed macOS MOTAA Desktop.app bundle
	bash scripts/build-macos.sh

install: ## Build and install MOTAA Desktop.app to /Applications
	@app=$$(bash scripts/build-macos.sh | tail -n 1); \
	printf 'Installing %s to /Applications/MOTAA Desktop.app\n' "$$app"; \
	rm -rf "/Applications/MOTAA Desktop.app"; \
	cp -R "$$app" "/Applications/MOTAA Desktop.app"; \
	printf 'Installed /Applications/MOTAA Desktop.app successfully.\n'

open: ## Build and open the ad-hoc signed macOS app bundle
	@app=$$(bash scripts/build-macos.sh | tail -n 1); \
	printf 'Opening %s\n' "$$app"; \
	open "$$app"

clean: ## Clean Cargo build artifacts
	cargo clean
