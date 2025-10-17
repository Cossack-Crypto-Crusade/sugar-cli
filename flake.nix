{
  description = "Nix flake for Sugar CLI (Cossack-Crypto-Crusade)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
          config.allowUnfree = true;
        };

        # 🔒 Pin Rust to match rust-toolchain.toml
        rust = pkgs.rust-bin.stable."1.68.0".default;

        src = pkgs.fetchFromGitHub {
          owner = "Cossack-Crypto-Crusade";
          repo = "sugar-cli";
          rev = "main"; # Pin to specific commit if needed
          # sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };

      in {
        # 🧱 Buildable package output
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "sugar-cli";
          version = "unstable";
          inherit src;

          cargoLock = {
            lockFile = src + "/Cargo.lock";
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            protobuf
          ];

          buildInputs = with pkgs; [
            openssl
            libgit2
            zlib
          ];

          doCheck = false;
        };

        # 🧰 Default development environment
        devShells.default = pkgs.mkShell {
          name = "sugar-cli-dev-env";

          buildInputs = with pkgs; [
            rust
            pkg-config
            openssl
            codeql
            libgit2
            cmake
            protobuf
            nodejs
            yarn
            solana-cli
          ];

          shellHook = ''
            echo "🧁 Entered Sugar CLI dev shell (Rust 1.68.0)"
            export RUST_BACKTRACE=1
            rustc --version
          '';
        };

        # 🛡️ CodeQL analysis shell
        devShells.codeql = pkgs.mkShell {
          name = "sugar-cli-codeql";

          buildInputs = with pkgs; [
            rust
            codeql
            cargo
          ];

          shellHook = ''
            echo "🛡️ Setting up CodeQL analysis..."

            if [ ! -d codeql-db ]; then
              codeql database create codeql-db --language=rust --command="cargo build"
            fi

            codeql database analyze codeql-db ~/codeql-repo/rust/ql/src/Security/*.ql \
              --format=sarif-latest \
              --output=results.sarif

            echo "✅ CodeQL analysis finished! See results.sarif"
          '';
        };
      });
}
