{
  pkgs,
  ...
}:
let
  qtdeps = pkgs.qt6.env "qtenv" [
    pkgs.qt6.qtbase
    pkgs.qt6.qtbase.dev
    pkgs.qt6.qtdeclarative
    pkgs.qt6.qtdeclarative.dev
    pkgs.qt6.qttools
    pkgs.qt6.qttools.dev
    pkgs.qt6.qtwayland
    pkgs.qt6.qtwayland.dev
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
    QMLLS_BUILD_DIRS = "$NIXPKGS_QT6_QML_IMPORT_PATH";
    CMAKE_PREFIX_PATH = "${qtdeps}";
    QT_HOST_PATH = "${qtdeps}";

    # Crucial for qt-build-utils / qmake to locate QuickControls2 and Qml .prl files:
    QMAKEPATH = "${qtdeps}/share/qt/mkspecs:${qtdeps}/lib/qt6/mkspecs";
    QT_PLUGIN_PATH = "${qtdeps}/${pkgs.qt6.qtbase.qtPluginPrefix}";

    RUST_LOG = "DEBUG";
  };
}
