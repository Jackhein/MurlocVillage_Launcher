use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    // Initialize the application
    let mut app = QGuiApplication::new();

    // Initialize the QML engine
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML file into the engine
    if let Some(engine) = engine.as_mut() {
        // Ensure this path is where the QML file is located or packaged
        engine.load(&QUrl::from("qrc:/qml/main.qml"));
    }

    // Execute the application
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
