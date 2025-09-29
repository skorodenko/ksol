{
  description = "Rust devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=210c7e4d8d958a8f373a97aada2c40579cd2020d";
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
              qt6.full
              kdePackages.extra-cmake-modules
              kdePackages.kirigami
              kdePackages.kirigami-addons
              kdePackages.qqc2-desktop-style
              kdePackages.wrapQtAppsHook
              clang
              cmake
              llvmPackages.bintools
              rust-analyzer
              rust-bin.beta.latest.default
            ];
            shellHook = ''
              #export QML_IMPORT_PATH=$NIXPKGS_QT6_QML_IMPORT_PATH
              #export QML2_IMPORT_PATH=${qt6.qtdeclarative}/lib/qt-6/qml:${kdePackages.qtdeclarative}/lib/qt-6/qml
              export QMAKE=qmake6
              export RUST_LOG=DEBUG
            '';
          };
      }
    );
}
