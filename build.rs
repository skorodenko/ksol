use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        // Link Qt's Network library
        // - Qt Core is always linked
        // - Qt Gui is linked by enabling the qt_gui Cargo feature of cxx-qt-lib.
        // - Qt Qml is linked by enabling the qt_qml Cargo feature of cxx-qt-lib.
        // - Qt Qml requires linking Qt Network on macOS
        .qml_module(QmlModule {
            uri: "github.skorodenko.ksol",
            rust_files: &["src/mpd_connector.rs"],
            qml_files: &["src/qml/main.qml"],
            ..Default::default()
        })
        .build();
}
