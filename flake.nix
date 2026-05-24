{
  description = "Rust devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      nixpkgs-unstable,
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
        pkgs-unstable = import nixpkgs-unstable {
          inherit system;
        };
        overlays = [
          (import rust-overlay)
        ];
        qt6deps = pkgs.qt6.env "qt-full-${pkgs.qt6.qtbase.version}" (
          with pkgs.qt6;
          [
            qtdeclarative
            qtlanguageserver
            qttools
            qtwayland
          ]
        );
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            packages = [
              mpd
              qt6deps
              kdePackages.kirigami
              kdePackages.kirigami-addons
              clang
              cmake
              llvmPackages.bintools
              pkgs-unstable.rust-analyzer
              rust-bin.beta.latest.default
              pkgs-unstable.sccache
              nasm
            ];
            shellHook = ''
              # Qmlls fix
              export QMLLS_BUILD_DIRS=$NIXPKGS_QT6_QML_IMPORT_PATH
              # CxxQt build fix
              export QMAKE=qmake6
              export RUST_LOG=DEBUG
            '';
          };
      }
    );
}
