import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Window 2.12

// This must match the qml_uri and qml_version
// specified with the #[cxx_qt::qobject] macro in Rust.
import MurlocVillage_Launcher 1.0

Window {
    title: qsTr("Hello App")
    visible: true
    height: 480
    width: 640
    color: "#e4af79"

    FileIntegrity  {
        id: fileintegrity
        file: ""
        result: ""
    }

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        /* space between widget */
        spacing: 10

        Button {
            text: "Verify"
            onClicked: fileintegrity.checkFile()
        }

        TextArea {
            placeholderText: qsTr("file to check")
            text: fileintegrity.file
            onTextChanged: fileintegrity.file = text

            background: Rectangle {
                implicitWidth: 400
                implicitHeight: 50
                radius: 3
                color:  "#e2e8f0"
                border.color:  "#21be2b"
            }
        }

        Label {
            text: fileintegrity.result
        }
    }
}