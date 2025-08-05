{
  description = "ATP - Internet Computer canister development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Project metadata
        cargoToml = pkgs.lib.importTOML ./src/atp/Cargo.toml;
        workspaceToml = pkgs.lib.importTOML ./Cargo.toml;

        # Common configurations
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        commonBuildInputs = with pkgs; [
          openssl
          pocket-ic
          cacert
        ];

        commonNativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        # Helper functions
        mkPocketIC =
          version: sha256:
          pkgs.stdenv.mkDerivation rec {
            pname = "pocket-ic";
            inherit version;

            src = pkgs.fetchurl {
              url = "https://github.com/dfinity/pocketic/releases/download/${version}/pocket-ic-x86_64-linux.gz";
              inherit sha256;
            };

            nativeBuildInputs = with pkgs; [
              gzip
              patchelf
            ];

            unpackPhase = ''
              gzip -d < $src > pocket-ic
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp pocket-ic $out/bin/
              chmod +x $out/bin/pocket-ic

              # Patch the binary for NixOS
              patchelf --set-interpreter ${pkgs.glibc}/lib/ld-linux-x86-64.so.2 $out/bin/pocket-ic
              patchelf --set-rpath ${
                pkgs.lib.makeLibraryPath [
                  pkgs.glibc
                  pkgs.gcc-unwrapped.lib
                ]
              } $out/bin/pocket-ic
            '';

            buildInputs = with pkgs; [
              glibc
              gcc-unwrapped.lib
            ];

            meta = with pkgs.lib; {
              description = "PocketIC - Local Internet Computer replica for testing";
              homepage = "https://github.com/dfinity/pocketic";
              license = licenses.asl20;
              platforms = [ "x86_64-linux" ];
            };
          };

        mkCandidExtractor =
          version: sha256: cargoHash:
          pkgs.rustPlatform.buildRustPackage rec {
            pname = "candid-extractor";
            inherit version;
            src = pkgs.fetchCrate {
              inherit pname version sha256;
            };
            inherit cargoHash;
          };

        # Build helpers
        mkWasmBuild = feature: ''
          echo "Building ${feature} environment..."
          cargo build --package atp --target wasm32-unknown-unknown --release --no-default-features --features ${feature}
          candid-extractor target/wasm32-unknown-unknown/release/atp.wasm > target/wasm32-unknown-unknown/release/atp-${feature}.did
          cp target/wasm32-unknown-unknown/release/atp.wasm target/wasm32-unknown-unknown/release/atp-${feature}.wasm
        '';

        buildAllEnvironments = ''
          echo "Building all environments..."
          ${mkWasmBuild "local"}
          ${mkWasmBuild "test"}
          ${mkWasmBuild "production"}
        '';

        installAllArtifacts = ''
          mkdir -p $out/atp
          cp target/wasm32-unknown-unknown/release/atp-local.wasm $out/atp/
          cp target/wasm32-unknown-unknown/release/atp-local.did $out/atp/
          cp target/wasm32-unknown-unknown/release/atp-test.wasm $out/atp/
          cp target/wasm32-unknown-unknown/release/atp-test.did $out/atp/
          cp target/wasm32-unknown-unknown/release/atp-production.wasm $out/atp/
          cp target/wasm32-unknown-unknown/release/atp-production.did $out/atp/
        '';

        # Tool instances
        pocket-ic = mkPocketIC "9.0.3" "sha256-y/QII7qocs7KpD49mZDtItJuBpQuRtCfWQV+jhK1L44=";
        candid-extractor =
          mkCandidExtractor "0.1.6" "sha256-MTLhYGcrGaLc84YjX2QXMsY4+UrxDvWpFVBw5WZxnN8="
            "sha256-Mq2tO8gD7v5P7NGH+R4QkyA7jRXo4knIi+eoGT4JzuU=";
      in
      {
        # Development environment
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Build tools
            pkg-config
            openssl

            # Development tools
            cargo-watch
            cargo-edit
            cargo-outdated
            cargo-release

            # Internet Computer tools
            pocket-ic

            # System tools
            git
            gnumake
          ];

          shellHook = ''
            # Add cargo bin to PATH
            export PATH="$HOME/.cargo/bin:$PATH"

            # Set up SSL certificates
            export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
            export NIX_SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt

            # Set POCKET_IC_BIN environment variable
            export POCKET_IC_BIN=${pocket-ic}/bin/pocket-ic

            # Install candid-extractor if not present
            if ! command -v candid-extractor &> /dev/null; then
              echo "Installing candid-extractor..."
              cargo install candid-extractor
            fi

            echo "🦀 ATP Development Environment"
            echo "================================"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "pocket-ic version: $(pocket-ic --version)"
            echo ""
          '';

          # Environment variables
          RUST_BACKTRACE = "1";
          POCKET_IC_BIN = "${pocket-ic}/bin/pocket-ic";
        };

        # Build packages
        packages = {
          default = pkgs.rustPlatform.buildRustPackage rec {
            pname = cargoToml.package.name;
            version = workspaceToml.workspace.package.version;

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = commonNativeBuildInputs ++ [ candid-extractor ];
            buildInputs = commonBuildInputs;

            # Enable tests
            doCheck = true;

            # Override the check phase to only run lib tests and skip integration test directory
            checkPhase = ''
              runHook preCheck

              # Set up SSL certificates for PocketIC
              export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
              export NIX_SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt

              # Set up PocketIC environment
              export POCKET_IC_BIN=${pocket-ic}/bin/pocket-ic
              echo "Using PocketIC at: $POCKET_IC_BIN"

              cargo test --release

              runHook postCheck
            '';

            buildPhase = buildAllEnvironments;
            installPhase = installAllArtifacts;

            meta = with pkgs.lib; {
              description = cargoToml.package.description;
              homepage = cargoToml.package.repository;
              repository = cargoToml.package.repository;
              license = licenses.mit;
              maintainers = [ ];
              platforms = platforms.unix;
            };
          };
        };

        # Development apps
        apps = {
          build-dev = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "build-atp" ''
              cargo build --package atp --target wasm32-unknown-unknown --release
              candid-extractor target/wasm32-unknown-unknown/release/atp.wasm > target/wasm32-unknown-unknown/release/atp.did
            '';
          };

          test = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "test" ''
              # Set up SSL certificates
              export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
              export NIX_SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt

              # Build WASM for integration tests with test features
              echo "Building WASM for integration tests..."
              cargo build --package atp --target wasm32-unknown-unknown --release --no-default-features --features test

              # Set up environment for integration tests
              export POCKET_IC_BIN=${pocket-ic}/bin/pocket-ic
              export RUST_BACKTRACE=1

              # Run all tests including integration tests
              echo "Running all tests including integration tests..."
              cargo test --release
            '';
          };
        };
      }
    );
}
