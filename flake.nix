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

        # Local path to your repo
        src = ./.;

        # Build sugar-cli package
        sugar-cli = pkgs.rustPlatform.buildRustPackage {
          pname = "sugar-cli";
          version = "unstable";
          inherit src;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [ pkg-config cmake protobuf ];
          buildInputs = with pkgs; [ openssl libgit2 zlib ];
          doCheck = false;
        };
      in
      {
        # 🧱 Default package output
        packages.default = sugar-cli;

        # 🧰 Default dev shell
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
            pnpm
            yarn
            solana-cli
            sugar-cli
          ];

          shellHook = ''
            echo "🧁 Entered Sugar CLI dev shell (Rust 1.68.0)"
            export RUST_BACKTRACE=1
            rustc --version
            sugar --version

            # Ensure ~/bin exists and is at the front of PATH
            mkdir -p $HOME/bin
            export PATH="$HOME/bin:$PATH"
            pnpm i

            # Helper to install sugar to ~/bin
            install-sugar() {
              if [ -f $HOME/bin/sugar ]; then
                echo "⚠️ $HOME/bin/sugar already exists. Overwrite? (y/N)"
                read ans
                [ "$ans" != "y" ] && return 0
              fi
              echo "📦 Installing sugar CLI to ~/bin..."
              install -Dm755 ${sugar-cli}/bin/sugar $HOME/bin/sugar
              echo "✅ sugar CLI installed to ~/bin!"
            }

            echo "💡 Run 'install-sugar' to install sugar CLI to ~/bin."
          '';
        };

        # 🛡️ CodeQL shell
        devShells.codeql = pkgs.mkShell {
          name = "sugar-cli-codeql";

          buildInputs = with pkgs; [ rust codeql cargo ];

          shellHook = ''
            echo "🛡️ Entered CodeQL analysis shell."
            echo "💡 Run 'codeql database create' and 'codeql database analyze' manually as needed."
          '';
        };

        # 🛠️ Install derivation
        packages.install = pkgs.stdenv.mkDerivation {
          name = "sugar-cli-install";
          buildInputs = [];
          unpackPhase = ":";
          installPhase = ''
            mkdir -p $HOME/bin
            echo "📦 Installing sugar CLI to ~/bin..."
            install -Dm755 ${sugar-cli}/bin/sugar $HOME/bin/sugar
            echo "✅ Installed sugar CLI to ~/bin."
          '';
        };
      });
}
