# IC Nix environment version
IC_NIX_VERSION := 20250627

.PHONY: all build build-release test nix-shell-env clean help 
# Default target
all: build-release

# Build release
build-release:
	nix build

# Build the project with nix development environment
build-dev:
	nix run .#build-dev

# Setup nix environment and run tests
test:
	nix run .#test

# Enter nix-shell with POCKET_IC_BIN exported
nix-shell-env:
	nix-shell https://github.com/ninegua/ic-nix/releases/download/$(IC_NIX_VERSION)/dfx-env.tar.gz --run '\
		export POCKET_IC_BIN=$$(which pocket-ic) && \
		echo "Environment ready. POCKET_IC_BIN=$$POCKET_IC_BIN" && \
		exec $$SHELL'

# Clean build artifacts
clean:
	cargo clean

# Help target
help:
	@echo "Available targets:"
	@echo "  build-dev          - Build the project with nix development environment"
	@echo "  build-release   - Build the project for release"
	@echo "  test            - Run all tests in nix environment"
	@echo "  nix-shell-env   - Enter nix-shell with POCKET_IC_BIN exported"
	@echo "  clean           - Clean build artifacts"
	@echo "  all             - Build wasm and generate DID (default)"
	@echo ""
	@echo "Configuration:"
	@echo "  IC_NIX_VERSION  - IC Nix version (current: $(IC_NIX_VERSION))"
