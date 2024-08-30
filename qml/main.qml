import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Window 2.12

// This must match the qml_uri and qml_version
// specified with the #[cxx_qt::qobject] macro in Rust.
import WoW_Private_Server_Launcher 1.0

Window {
    title: qsTr(fileintegrity.title)
    visible: true
    height: 480
    width: 640
    color: "#e4af79"

    FileIntegrity  {
        id: fileintegrity
        title: fileintegrity.title
        result: fileintegrity.result
        verify: fileintegrity.verify
        play: fileintegrity.play
        language: fileintegrity.language
        buttonColor: fileintegrity.buttonColor
    }

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        /* space between widget */
        spacing: 10

        Button {
            text: fileintegrity.verify
            onClicked: fileintegrity.checkFile()
            background: Rectangle {
                color: fileintegrity.buttonColor
            }
        }

        Button {
            text: fileintegrity.play
            onClicked: fileintegrity.result=fileintegrity.startGame()
            background: Rectangle {
                color: fileintegrity.buttonColor
            }
        }

        Button {
            text: fileintegrity.language
            background: Rectangle {
                color: fileintegrity.buttonColor
            }
        }

        TextArea {
            //placeholderText: qsTr("file to check")
            text: fileintegrity.result
            //onTextChanged: fileintegrity.result = text

            background: Rectangle {
                implicitWidth: 400
                implicitHeight: 50
                radius: 3
                color:  "#e2e8f0"
                border.color:  "#21be2b"
            }
        }
    }
}