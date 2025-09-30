use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qml_module(QmlModule {
            uri: "github.skorodenko.ksol",
            rust_files: &[
                "src/rust/qmpd_connector.rs",
                "src/rust/qplaylists_group_model.rs",
                "src/rust/qplaylists_list_model.rs",
                "src/rust/qplaylist_model.rs",
                "src/rust/qt.rs",
            ],
            qml_files: &[
                "src/qml/Main.qml",
                "src/qml/Drun.qml",
                "src/qml/Shortcuts.qml",
                "src/qml/PlaylistHeader.qml",
            ],
            ..Default::default()
        })
        .build();
}
