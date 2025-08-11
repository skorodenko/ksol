{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    clang
    qt6.full
    kdePackages.qtdeclarative
    llvmPackages.bintools
  ];
  shellHook = ''
    export QMAKE="qmake6"
  '';
}
