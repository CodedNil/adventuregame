{
  description = "Development shell for The Unreliable Fellowship";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            dioxus-cli
            lld
            binaryen
            cargo-watch
          ];

          RUST_BACKTRACE = "1";
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    );
}
