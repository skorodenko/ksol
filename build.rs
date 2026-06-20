use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("github.skorodenko.ksol")
            .qml_files([
                "src/qml/Main.qml",
                "src/qml/Drun.qml",
                "src/qml/AppShortcuts.qml",
                "src/qml/BackgroundImage.qml",
                "src/qml/PlaylistHeader.qml",
                "src/qml/About.qml",
                "src/qml/InitWizard.qml",
                "src/qml/Settings.qml",
                "src/qml/SettingsMPDPage.qml",
                "src/qml/SettingsAppearancePage.qml",
                "src/qml/SidebarDelegate.qml",
            ])
            .depend("QtQuick"),
    )
    .files([
        "src/qt/mpd.rs",
        "src/qt/playlists.rs",
        "src/qt/playlist.rs",
        "src/qt/settings.rs",
        "src/qt/state.rs",
        "src/qt/qt.rs",
    ])
    .build();
}
