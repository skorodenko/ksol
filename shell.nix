{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    clang
    rust-analyzer
    qt6.full
    kdePackages.qtdeclarative
    llvmPackages.bintools
  ];
  shellHook = ''
    export QMAKE=qmake6
    export RUST_LOG=DEBUG
  '';
}
