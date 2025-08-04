# IC Nix environment version
IC_NIX_VERSION := 20250627

.PHONY: all build generate-did test test-package nix-shell-env clean install-tools help 
# Default target
all: build-atp generate-did-atp

# Build the project for native target
build:
	cargo build --package atp --target wasm32-unknown-unknown --release

# Generate Candid interface definition
generate-did: build\
  candid-extractor target/wasm32-unknown-unknown/release/atp.wasm > target/wasm32-unknown-unknown/release/atp.did

# Setup nix environment and run tests
test:
	nix-shell https://github.com/ninegua/ic-nix/releases/download/$(IC_NIX_VERSION)/dfx-env.tar.gz --run '\
		export POCKET_IC_BIN=$$(which pocket-ic) && \
		cargo test'

# Run tests for specific package
test-package:
	nix-shell https://github.com/ninegua/ic-nix/releases/download/$(IC_NIX_VERSION)/dfx-env.tar.gz --run '\
		export POCKET_IC_BIN=$$(which pocket-ic) && \
		cargo test --package ic-nosql-tests'

# Enter nix-shell with POCKET_IC_BIN exported
nix-shell-env:
	nix-shell https://github.com/ninegua/ic-nix/releases/download/$(IC_NIX_VERSION)/dfx-env.tar.gz --run '\
		export POCKET_IC_BIN=$$(which pocket-ic) && \
		echo "Environment ready. POCKET_IC_BIN=$$POCKET_IC_BIN" && \
		exec $$SHELL'

# Clean build artifacts
clean:
	cargo clean

# Install required tools
install-tools:
	rustup target add wasm32-unknown-unknown
	cargo install candid-extractor

# Help target
help:
	@echo "Available targets:"
	@echo "  build           - Build the project for native target"
	@echo "  generate-did - Generate Candid interface definition for ATP"
	@echo "  test            - Run all tests in nix environment"
	@echo "  test-package    - Run ic-nosql-tests package tests in nix environment"
	@echo "  nix-shell-env   - Enter nix-shell with POCKET_IC_BIN exported"
	@echo "  clean           - Clean build artifacts"
	@echo "  install-tools   - Install required tools (wasm32 target and candid-extractor)"
	@echo "  all             - Build wasm and generate DID (default)"
	@echo ""
	@echo "Configuration:"
	@echo "  IC_NIX_VERSION  - IC Nix version (current: $(IC_NIX_VERSION))"
