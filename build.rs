use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("github.skorodenko.ksol").qml_files([
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
    ]))
    .files([
        "src/rust/qmpd_connector.rs",
        "src/rust/qplaylists_group_model.rs",
        "src/rust/qplaylists_list_model.rs",
        "src/rust/qplaylist_model.rs",
        "src/rust/qsettings_model.rs",
        "src/rust/qt.rs",
    ])
    .build();
}
