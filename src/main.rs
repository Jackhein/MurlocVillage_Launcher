mod file_integrity;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from_local_file(&QString::from("qml/main.qml")));
    } else {
        eprintln!("Failed to load QQmlApplicationEngine.");
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    } else {
        eprintln!("Failed to execute QGuiApplication.");
    }
}
