{
  description = "Rust devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              mpd
              clang
              qt6.full
              kdePackages.qtdeclarative
              llvmPackages.bintools
              rust-analyzer
              rust-bin.beta.latest.default
            ];
            shellHook = ''
              export QML_IMPORT_PATH=$NIXPKGS_QT6_QML_IMPORT_PATH
              export QMAKE=qmake6
              export RUST_LOG=DEBUG
            '';
          };
      }
    );
}
