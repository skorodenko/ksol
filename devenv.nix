{
  pkgs,
  ...
}:
let
  qtdeps = pkgs.kdePackages.env "qtenv" [
    pkgs.kdePackages.qtbase
    pkgs.kdePackages.qtdeclarative
    pkgs.kdePackages.qttools
    pkgs.kdePackages.qtwayland
    pkgs.kdePackages.kirigami
    pkgs.kdePackages.kirigami-addons
  ];
in
{
  packages = with pkgs; [
    mpd
    mesa
    libglvnd
    qtdeps
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";

    clangLinker.enable = true;

    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "miri"
    ];
  };

  env = {
    QML_IMPORT_PATH = "${qtdeps}/lib/qt-6/qml";
    RUST_LOG = "DEBUG";
  };
}
