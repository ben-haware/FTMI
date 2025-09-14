.PHONY: help install install-cargo install-binstall build test clean release

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