.PHONY: help install install-cargo install-binstall build test clean release tag-release patch

# Default target
help:
	@echo "Available targets:"
	@echo "  install       - Full installation (installs cargo, binstall, and ftmi)"
	@echo "  install-cargo - Install Rust and Cargo"
	@echo "  install-binstall - Install cargo-binstall"
	@echo "  build         - Build the project in debug mode"
	@echo "  test          - Run all tests"
	@echo "  clean         - Clean build artifacts"
	@echo "  release       - Build in release mode"
	@echo "  tag-release   - Create and push git tag (VERSION=x.y.z or auto-increment minor)"
	@echo "  patch         - Increment patch version and create tag"

# Full installation
install: install-cargo install-binstall
	@echo "Installing FTMI..."
	cargo binstall ftmi -y || cargo install --path .

# Install Rust and Cargo
install-cargo:
	@echo "Checking for Rust installation..."
	@if command -v cargo >/dev/null 2>&1; then \
		echo "Cargo is already installed: $$(cargo --version)"; \
	else \
		echo "Installing Rust and Cargo..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "Please run 'source ~/.cargo/env' or restart your terminal"; \
	fi

# Install cargo-binstall
install-binstall:
	@echo "Checking for cargo-binstall..."
	@if command -v cargo-binstall >/dev/null 2>&1; then \
		echo "cargo-binstall is already installed: $$(cargo-binstall --version)"; \
	else \
		echo "Installing cargo-binstall..."; \
		curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash; \
	fi

# Development targets
build:
	cargo build

test:
	cargo test

clean:
	cargo clean

release:
	cargo build --release --bins

# Release management
tag-release:
	@if [ -n "$(VERSION)" ]; then \
		echo "Creating release v$(VERSION)"; \
		git tag v$(VERSION); \
		git push origin v$(VERSION); \
	else \
		echo "Auto-incrementing minor version..."; \
		LAST_TAG=$$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0"); \
		echo "Last tag: $$LAST_TAG"; \
		LAST_VERSION=$$(echo $$LAST_TAG | sed 's/^v//'); \
		MAJOR=$$(echo $$LAST_VERSION | cut -d. -f1); \
		MINOR=$$(echo $$LAST_VERSION | cut -d. -f2); \
		NEW_MINOR=$$(($$MINOR + 1)); \
		NEW_VERSION="$$MAJOR.$$NEW_MINOR.0"; \
		echo "Creating release v$$NEW_VERSION"; \
		git tag v$$NEW_VERSION; \
		git push origin v$$NEW_VERSION; \
	fi

patch:
	@echo "Auto-incrementing patch version..."
	@LAST_TAG=$$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0"); \
	echo "Last tag: $$LAST_TAG"; \
	LAST_VERSION=$$(echo $$LAST_TAG | sed 's/^v//'); \
	MAJOR=$$(echo $$LAST_VERSION | cut -d. -f1); \
	MINOR=$$(echo $$LAST_VERSION | cut -d. -f2); \
	PATCH=$$(echo $$LAST_VERSION | cut -d. -f3); \
	NEW_PATCH=$$(($$PATCH + 1)); \
	NEW_VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
	echo "Creating release v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push origin v$$NEW_VERSION