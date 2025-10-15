{
  description = "Nix flake for Sugar CLI (Cossack-Crypto-Crusade)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        rustPlatform = pkgs.rustPlatform;
        src = pkgs.fetchFromGitHub {
          owner = "Cossack-Crypto-Crusade";
          repo = "sugar-cli";
          rev = "main"; # you can pin a commit hash here
          # sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
          # ^ fill in once pinned; leave commented for local development
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

        # 🧰 Development environment
        devShells.default = pkgs.mkShell {
          name = "sugar-cli-dev";

          buildInputs = with pkgs; [
            rustup
            pkg-config
            openssl
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
          '';
        };
      });
}
