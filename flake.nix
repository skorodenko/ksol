{
  description = "Rust devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
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
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        overlays = [
          (import rust-overlay)
        ];
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              mpd
              qt6.full
              qt6.wrapQtAppsHook
              kdePackages.extra-cmake-modules
              kdePackages.kirigami
              kdePackages.kirigami-addons
              kdePackages.qqc2-desktop-style
              clang
              cmake
              llvmPackages.bintools
              rust-analyzer
              rust-bin.beta.latest.default
            ];
            shellHook = ''
              export QML_IMPORT_PATH=$NIXPKGS_QT6_QML_IMPORT_PATH
              export QMAKE=qmake6
              export RUST_LOG=DEBUG
              alias pbuild="cmake --build build/ && wrapQtApp ./build/debug/ksol"
            '';
          };
      }
    );
}
