extern crate ksol;

use tokio;
use cxx_qt_lib::{QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use cxx_qt_lib_extras::QApplication;
use log::{LevelFilter, debug};
use std::env;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    debug!("Starting application");

    let mut app = QApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // To associate the executable to the installed desktop file
    //QGuiApplication::set_desktop_file_name(&QString::from("org.kde.kirigami_rust"));
    // To ensure the style is set correctly
    let style = env::var("QT_QUICK_CONTROLS_STYLE");
    if style.is_err() {
        QQuickStyle::set_style(&QString::from("org.kde.desktop"));
    }

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(
            "qrc:/qt/qml/github/skorodenko/ksol/src/qml/Main.qml",
        ));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }

    debug!("Application closed")
}
