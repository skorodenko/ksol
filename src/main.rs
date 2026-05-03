extern crate ksol;

use std::env;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tracing::instrument(level = "debug", name = "ksol")]
fn main() {
    use cxx_qt_lib::{
        QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl,
    };

    let filter = EnvFilter::from_default_env()
        .add_directive("mpd_protocol=error".parse().unwrap());
    tracing_subscriber::registry().with(fmt::layer()).with(filter).init();

    tracing::debug!("Starting application");

    let mut app = QGuiApplication::new();
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

    tracing::debug!("Application closing");
    ksol::utils::settings::Settings::dump();
}
