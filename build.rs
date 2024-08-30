use cxx_qt_build::{CxxQtBuilder, QmlModule};

/// https://doc.qt.io/qt-6/resources.html
fn main() {
    CxxQtBuilder::new()
        .qt_module("Network")
        .qml_module(QmlModule {
            uri: "WoW_Private_Server_Launcher",
            version_major: 1,
            version_minor: 0,
            rust_files: &["src/file_integrity.rs"],
            qml_files: &["qml/qml.qrc"],
            qrc_files: &[],
            ..Default::default()
        })
        .build();
}
