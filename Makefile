.PHONY: help install install-cargo install-binstall build test clean release tag-release patch actions action status ensure-build

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
	@echo "  actions       - Open general GitHub Actions page"
	@echo "  action        - Open currently running GitHub Action for latest tag"
	@echo "  status        - Show success/failure status of latest tag release (CLI only)"
	@echo "  ensure-build  - Build project and stage Cargo.lock for commit"

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

# Ensure project builds and Cargo.lock is up to date
ensure-build:
	@echo "Building project to ensure it compiles..."
	cargo build
	@echo "Staging Cargo.lock if it was updated..."
	@if git diff --name-only Cargo.lock | grep -q Cargo.lock; then \
		echo "Cargo.lock was updated, staging for commit"; \
		git add Cargo.lock; \
	else \
		echo "Cargo.lock unchanged"; \
	fi

patch: ensure-build
	@echo "Auto-incrementing patch version..."
	@LAST_TAG=$$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0"); \
	echo "Last tag: $$LAST_TAG"; \
	LAST_VERSION=$$(echo $$LAST_TAG | sed 's/^v//'); \
	MAJOR=$$(echo $$LAST_VERSION | cut -d. -f1); \
	MINOR=$$(echo $$LAST_VERSION | cut -d. -f2); \
	PATCH=$$(echo $$LAST_VERSION | cut -d. -f3); \
	NEW_PATCH=$$(($$PATCH + 1)); \
	NEW_VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
	while git tag -l | grep -q "^v$$NEW_VERSION$$"; do \
		NEW_PATCH=$$(($$NEW_PATCH + 1)); \
		NEW_VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
	done; \
	echo "Updating Cargo.toml to version $$NEW_VERSION"; \
	sed -i '' "s/^version = .*/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	cargo build; \
	git add Cargo.toml Cargo.lock; \
	git commit -m "Bump version to $$NEW_VERSION"; \
	echo "Creating release v$$NEW_VERSION"; \
	git tag v$$NEW_VERSION; \
	git push origin main; \
	git push origin v$$NEW_VERSION

# Open general GitHub Actions page
actions:
	@echo "Opening GitHub Actions page..."
	@REPO_URL=$$(git remote get-url origin 2>/dev/null || echo ""); \
	if [ -z "$$REPO_URL" ]; then \
		echo "Error: No git remote origin found"; \
		exit 1; \
	fi; \
	GITHUB_URL=$$(echo $$REPO_URL | sed 's/\.git$$//' | sed 's/^git@github\.com:/https:\/\/github.com\//'); \
	ACTIONS_URL="$$GITHUB_URL/actions"; \
	echo "Opening: $$ACTIONS_URL"; \
	open "$$ACTIONS_URL"

# Open currently running GitHub Action for the latest tag
action:
	@echo "Finding currently running action for latest tag..."
	@REPO_URL=$$(git remote get-url origin 2>/dev/null || echo ""); \
	if [ -z "$$REPO_URL" ]; then \
		echo "Error: No git remote origin found"; \
		exit 1; \
	fi; \
	LATEST_TAG=$$(git describe --tags --abbrev=0 2>/dev/null || echo ""); \
	if [ -z "$$LATEST_TAG" ]; then \
		echo "Error: No tags found in repository"; \
		exit 1; \
	fi; \
	echo "Latest tag: $$LATEST_TAG"; \
	GITHUB_URL=$$(echo $$REPO_URL | sed 's/\.git$$//' | sed 's/^git@github\.com:/https:\/\/github.com\//'); \
	REPO_PATH=$$(echo $$GITHUB_URL | sed 's|https://github.com/||'); \
	echo "Checking for running actions on tag $$LATEST_TAG..."; \
	RUN_ID=$$(gh api repos/$$REPO_PATH/actions/runs --jq ".workflow_runs[] | select(.head_branch == \"$$LATEST_TAG\" and (.status == \"in_progress\" or .status == \"queued\")) | .id" | head -n1); \
	if [ -n "$$RUN_ID" ]; then \
		ACTIONS_URL="$$GITHUB_URL/actions/runs/$$RUN_ID"; \
		echo "Opening currently running action: $$ACTIONS_URL"; \
		open "$$ACTIONS_URL"; \
	else \
		echo "No currently running actions found for tag $$LATEST_TAG"; \
		ACTIONS_URL="$$GITHUB_URL/actions?query=event%3Apush+branch%3A$$LATEST_TAG"; \
		echo "Opening actions page filtered by tag: $$ACTIONS_URL"; \
		open "$$ACTIONS_URL"; \
	fi

# Show CLI-only status of latest tag release
status:
	@echo "Checking release status for latest tag..."
	@REPO_URL=$$(git remote get-url origin 2>/dev/null || echo ""); \
	if [ -z "$$REPO_URL" ]; then \
		echo "❌ Error: No git remote origin found"; \
		exit 1; \
	fi; \
	LATEST_TAG=$$(git describe --tags --abbrev=0 2>/dev/null || echo ""); \
	if [ -z "$$LATEST_TAG" ]; then \
		echo "❌ Error: No tags found in repository"; \
		exit 1; \
	fi; \
	echo "📋 Latest tag: $$LATEST_TAG"; \
	GITHUB_URL=$$(echo $$REPO_URL | sed 's/\.git$$//' | sed 's/^git@github\.com:/https:\/\/github.com\//'); \
	REPO_PATH=$$(echo $$GITHUB_URL | sed 's|https://github.com/||'); \
	echo "🔍 Fetching workflow status..."; \
	RUNS_JSON=$$(gh api repos/$$REPO_PATH/actions/runs --jq ".workflow_runs[] | select(.head_branch == \"$$LATEST_TAG\") | {id, status, conclusion, workflow_name: .name, html_url, created_at}" | jq -s 'sort_by(.created_at) | reverse'); \
	if [ "$$RUNS_JSON" = "[]" ] || [ -z "$$RUNS_JSON" ]; then \
		echo "⚠️  No workflow runs found for tag $$LATEST_TAG"; \
		exit 0; \
	fi; \
	echo "$$RUNS_JSON" | jq -r '.[] | "🔧 \(.workflow_name):\n   Status: \(.status)\n   Result: \(.conclusion // "pending")\n   URL: \(.html_url)\n"'; \
	OVERALL_STATUS=$$(echo "$$RUNS_JSON" | jq -r 'map(select(.status == "completed")) | if length == 0 then "running" elif all(.conclusion == "success") then "success" elif any(.conclusion == "failure") then "failure" else "mixed" end'); \
	case $$OVERALL_STATUS in \
		success) echo "✅ Overall Status: SUCCESS - All workflows completed successfully" ;; \
		failure) echo "❌ Overall Status: FAILURE - One or more workflows failed" ;; \
		mixed) echo "⚠️  Overall Status: MIXED - Some workflows succeeded, others failed" ;; \
		running) echo "🔄 Overall Status: RUNNING - Workflows still in progress" ;; \
		*) echo "❓ Overall Status: UNKNOWN" ;; \
	esac