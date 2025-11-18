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
        root.useNativeMpdServer = QSettingsModel.mpdSocket == root.nativeMpdSocket
        root.nativeMpdSocket = QSettingsModel.getNativeMpdSocket()
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

    QQC2.GroupBox {
        id: externalServerGroup

        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
        }

        label: QQC2.RadioButton {
            id: esgCheckBox
            checked: !root.useNativeMpdServer
            text: qsTr("External mpd server (mpd starting/stopping/configuring done outside this app)")
            onClicked: root.useNativeMpdServer = false
        }

        Kirigami.FormLayout {
            anchors.fill: parent
            enabled: esgCheckBox.checked

            RowLayout {
                spacing: Kirigami.Units.mediumSpacing
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
        }
    }

    QQC2.GroupBox {
        id: nativeServerGroup
        padding: Kirigami.Units.largeSpacing

        anchors {
            top: externalServerGroup.bottom
            left: parent.left
            right: parent.right
        }

        label: QQC2.RadioButton {
            id: nsgCheckBox
            checked: root.useNativeMpdServer
            text: qsTr("Native mpd server (mpd starting/stopping/configuring managed by this app)")
            onClicked: {
                root.useNativeMpdServer = true;
                QSettingsModel.mpdSocket = root.nativeMpdSocket;
            }
        }

        Kirigami.FormLayout {
            anchors.fill: parent
            enabled: nsgCheckBox.checked

            QQC2.ComboBox {
                Kirigami.FormData.label: "Output plugin:"
                model: ["pipewire", "pulse", "alsa"]
                currentIndex: model.indexOf(QSettingsModel.outputPluginType)
                onActivated: index => {
                    QSettingsModel.outputPluginType = model[index];
                }
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
