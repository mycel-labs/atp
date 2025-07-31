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

        # Rust toolchain with wasm32 target
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Download and extract pocket-ic
        pocket-ic = pkgs.stdenv.mkDerivation rec {
          pname = "pocket-ic";
          version = "9.0.3";

          src = pkgs.fetchurl {
            url = "https://github.com/dfinity/pocketic/releases/download/${version}/pocket-ic-x86_64-linux.gz";
            sha256 = "sha256-y/QII7qocs7KpD49mZDtItJuBpQuRtCfWQV+jhK1L44=";
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

          # Add runtime dependencies
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
      in
      {
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

                  # Set POCKET_IC_BIN environment variable
                  export POCKET_IC_BIN=${pocket-ic}/bin/pocket-ic

                  # Install candid-extractor if not present
                  if ! command -v candid-extractor &> /dev/null; then
                    echo "Installing candid-extractor..."
            self,
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
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "atp";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              rustToolchain
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            # Build the WASM target
            buildPhase = ''
              cargo build --package atp --target wasm32-unknown-unknown --release
            '';

            installPhase = ''
              mkdir -p $out/lib
              cp target/wasm32-unknown-unknown/release/atp.wasm $out/lib/
            '';

            meta = with pkgs.lib; {
              description = "ATP - Internet Computer canister";
              license = licenses.mit; # Adjust license as needed
            };
          };
        };

        # Development apps
        apps = {
          build = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "build" ''
              cargo build --release
            '';
          };

          build-atp = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "build-atp" ''
              cargo build --package atp --target wasm32-unknown-unknown --release
            '';
          };

          test = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "test" ''
              export POCKET_IC_BIN=${pocket-ic}/bin/pocket-ic
              cargo test
            '';
          };
        };
      }
    );
}
