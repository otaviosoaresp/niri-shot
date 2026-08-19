{
  description = "Screenshot tool for Niri Wayland compositor with annotation support";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in rec {
        packages = rec {
          default = niri-shot;
          niri-shot = pkgs.rustPlatform.buildRustPackage {
            pname = "niri-shot";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = with pkgs; [ pkg-config ];

            buildInputs = with pkgs; [
              gtk4
              cairo
              pango
              gdk-pixbuf
              graphene
              libadwaita
              openssl
            ];

            cargoHash = "sha256-ggYwVTP737hR0n2XP0w1lSGw/HMqTa0UxebWQ6bpbo0=";
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ packages.default ];

          buildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer
          ];

          shellHook = ''
            export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
          '';
        };

        formatter = pkgs.nixfmt;
      });
}