use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qml_module(QmlModule {
            uri: "github.skorodenko.ksol",
            rust_files: &[
                "src/rust/qmpd_connector.rs",
                "src/rust/qplaylists_group_model.rs",
            ],
            qml_files: &[
                "src/qml/Main.qml",
                "src/qml/Drun.qml",
                "src/qml/Shortcuts.qml",
            ],
            ..Default::default()
        })
        .build();
}
