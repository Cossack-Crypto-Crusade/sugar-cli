{
  description = "Nix flake for Sugar CLI (Cossack-Crypto-Crusade)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;   # <<< enable unfree packages
          };
        };

        rustPlatform = pkgs.rustPlatform;
        src = pkgs.fetchFromGitHub {
          owner = "Cossack-Crypto-Crusade";
          repo = "sugar-cli";
          rev = "main"; # pin commit hash if desired
          # sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };

      in {
        # 🧱 Buildable package output
        packages.default = rustPlatform.buildRustPackage {
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

          # optional: verify build
          doCheck = false;
        };

        # 🧰 Default development environment
        devShells.default = pkgs.mkShell {
          name = "sugar-cli-dev";

          buildInputs = with pkgs; [
            rustup
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
            echo "🧁 Entered Sugar CLI dev shell"
            export RUST_BACKTRACE=1
            export PATH="$HOME/.cargo/bin:$PATH"
            rustc --version
            cargo build
            cargo fmt
          '';
        };

        # 🛡️ CodeQL analysis shell
        devShells.codeql = pkgs.mkShell {
          name = "sugar-cli-codeql";

          buildInputs = with pkgs; [
            rustup
            codeql
            cargo
          ];

          shellHook = ''
            echo "🛡️ Setting up CodeQL analysis..."
            
            # Create CodeQL database if missing
            if [ ! -d codeql-db ]; then
              codeql database create codeql-db --language=rust --command="cargo build"
            fi

            # Run CodeQL security queries
            codeql database analyze codeql-db ~/codeql-repo/rust/ql/src/Security/*.ql \
              --format=sarif-latest \
              --output=results.sarif

            echo "✅ CodeQL analysis finished! See results.sarif"
          '';
        };
      });
}
