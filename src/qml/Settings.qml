pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Window {
    id: root
    title: "Settings"
    visible: false

    flags: Qt.Dialog
    modality: Qt.WindowModal

    property string customServerUrl: ""
    property string nativeMpdSocket: QSettingsModel.getNativeMpdSocket()

    onBeforeRendering: {
        root.customServerUrl = "";
    }

    Connections {
        target: QSettingsModel

        function onCheckServerConnectionResult(result) {
            infoMessage.visible = false;
            infoMessage.visible = true;
            infoMessage.text = result ? "Successfully connected to server" : "Failed to connect to server";
            infoMessage.type = result ? Kirigami.MessageType.Positive : Kirigami.MessageType.Error;
        }
    }

    Kirigami.InlineMessage {
        id: infoMessage

        visible: false
        anchors.bottom: footer.top
        anchors.left: parent.left
        anchors.right: parent.right
        onVisibleChanged: tmr.restart()

        Timer {
            id: tmr
            interval: Kirigami.Units.humanMoment
            onTriggered: infoMessage.visible = false
        }
    }

    QQC2.TabBar {
        id: tabBar

        QQC2.TabButton {
            text: qsTr("Appearance")
        }
        QQC2.TabButton {
            text: qsTr("Connection")
        }
        QQC2.TabButton {
            text: qsTr("Native server")
        }
    }

    StackLayout {
        id: view

        currentIndex: tabBar.currentIndex

        anchors {
            top: tabBar.bottom
            left: parent.left
            right: parent.right
            bottom: infoMessage.top
        }

        Item {
            id: appearencePage
        }

        Item {
            id: connectionPage
            Item {
                anchors.fill: parent
                anchors.margins: 18

                Kirigami.Heading {
                    id: tpHeading
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    wrapMode: Text.WordWrap
                    text: "Enter uri to connect to mpd server"
                }

                QQC2.TextField {
                    id: tpAdressField
                    anchors.top: tpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.left: parent.left
                    anchors.right: tpAddressCheck.left
                    anchors.rightMargin: Kirigami.Units.mediumSpacing
                    Binding {
                        target: root
                        property: "customServerUrl"
                        value: tpAdressField.text
                    }
                    placeholderText: QSettingsModel.mpdSocket
                }

                QQC2.Button {
                    id: tpAddressCheck
                    anchors.top: tpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.right: parent.right
                    text: "Check connection"
                    onClicked: QSettingsModel.checkServerConnection(root.customServerUrl)
                }
            }
        }

        Item {
            id: nativeMpdConfig
        }
    }

    QQC2.Control {
        id: footer
        implicitHeight: 48
        padding: 12

        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
        }

        QQC2.Button {
            id: cancelButton
            text: "Cancel"

            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 6

            onClicked: root.visible = false
        }

        QQC2.Button {
            id: finishButton
            text: "Apply"

            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 6

            onClicked:
            //                root.visible = false;
            //                QSettingsModel.initWizard = false;
            //                if (root.selectedOption == 1) {
            //                    QSettingsModel.mpdSocket = root.customServerUrl;
            //                }
            //                root.finished();
            {}
        }
    }

    Rectangle {
        z: -1
        anchors.fill: parent
        color: Kirigami.Theme.backgroundColor
    }
}
