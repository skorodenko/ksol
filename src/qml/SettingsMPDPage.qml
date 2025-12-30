pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.Page {
    id: root

    signal connectionCheckResult(result: bool)

    property string customServerUrl: ""
    property bool useNativeMpdServer: QSettingsModel.mpdSocket == root.nativeMpdSocket
    property string nativeMpdSocket: QSettingsModel.getNativeMpdSocket()

    function reset() {
        root.customServerUrl = "";
        root.useNativeMpdServer = QSettingsModel.mpdSocket == root.nativeMpdSocket;
        root.nativeMpdSocket = QSettingsModel.getNativeMpdSocket();
    }

    function handleConnectionCheck() {
        let result = QSettingsModel.checkServerConnection(root.customServerUrl);
        infoMessage.visible = false;
        infoMessage.visible = true;
        infoMessage.text = result ? "Successfully connected to server, applying new url" : "Failed to connect to server";
        infoMessage.type = result ? Kirigami.MessageType.Positive : Kirigami.MessageType.Error;
        if (result) {
            QSettingsModel.mpdSocket = root.customServerUrl;
        }
        root.connectionCheckResult(result);
    }

    Kirigami.FormLayout {
        anchors.fill: parent

        Item {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: "MPD server type"
        }

        ColumnLayout {
            Kirigami.FormData.label: "Choose MPD server type:"

            QQC2.RadioButton {
                checked: !root.useNativeMpdServer
                onClicked: root.useNativeMpdServer = false
                text: qsTr("External (managed outside this app)")
            }
            QQC2.RadioButton {
                checked: root.useNativeMpdServer
                onClicked: {
                    root.useNativeMpdServer = true;
                    QSettingsModel.mpdSocket = root.nativeMpdSocket;
                }
                text: qsTr("Native (managed by this app)")
            }
        }

        Item {
            enabled: !root.useNativeMpdServer
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: "External server settings"
        }

        RowLayout {
            spacing: Kirigami.Units.mediumSpacing
            enabled: !root.useNativeMpdServer
            Kirigami.FormData.label: "Server url:"

            QQC2.TextField {
                id: gpServerUrl
                Binding {
                    target: root
                    property: "customServerUrl"
                    value: gpServerUrl.text
                }
                placeholderText: QSettingsModel.mpdSocket == root.nativeMpdSocket ? "External server url (http://... or Unix socket)" : QSettingsModel.mpdSocket
            }

            QQC2.Button {
                id: gpAddressCheck
                text: "Check & apply"
                onClicked: root.handleConnectionCheck()
            }
        }

        Item {
            enabled: root.useNativeMpdServer
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: "Native server settings"
        }

        QQC2.ComboBox {
            enabled: root.useNativeMpdServer
            Kirigami.FormData.label: "Output plugin:"
            model: ["pipewire", "pulse", "alsa"]
            currentIndex: model.indexOf(QSettingsModel.outputPluginType)
            onActivated: index => {
                QSettingsModel.outputPluginType = model[index];
            }
        }
    }

    Kirigami.InlineMessage {
        id: infoMessage

        visible: false
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        onVisibleChanged: tmr.restart()

        Timer {
            id: tmr
            interval: Kirigami.Units.humanMoment
            onTriggered: infoMessage.visible = false
        }
    }
}
