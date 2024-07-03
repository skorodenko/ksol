import QtQml
import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import controllers 1.0

Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: mainPage

    Component.onCompleted: mpd_connector.connect()
    Component.onDestruction: mpd_connector.disconnect()

    MPDConnector {
        id: mpd_connector
        onConnected: toggleMessage.visible = true
    }

    globalDrawer: Kirigami.GlobalDrawer {
        id: globalDrawer
        title: "Global menu"

        edge: Qt.RightEdge
        handleVisible: false

        header: ColumnLayout {
            Layout.fillWidth: true
            QQC2.ComboBox {
                visible: !globalDrawer.collapsed
                Layout.fillWidth: true
            }
        }
    }

    Kirigami.Page {
        id: mainPage
        globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

        header: QQC2.ToolBar {
            implicitHeight: 48

            RowLayout {
                spacing: 8
                anchors.rightMargin: 8
                anchors.fill: parent

                RowLayout {
                    id: media_playback_controls
                    spacing: 0

                    QQC2.Button {
                        id: playback_backward
                        icon.name: "media-skip-backward"
                        flat: true
                    }
                    QQC2.Button {
                        id: playback_play
                        icon.name: "media-playback-start"
                        flat: true
                    }
                    QQC2.Button {
                        id: playback_forward
                        icon.name: "media-skip-forward"
                        flat: true
                    }
                }

                ColumnLayout {
                    spacing: 0

                    RowLayout {
                        QQC2.Label {
                            id: media_title
                            text: "Title | Author"
                            Layout.fillWidth: true
                        }
                        QQC2.Label {
                            id: media_duration
                            text: "0:00 / 0:00"
                            Layout.rightMargin: Qt.Infinity
                        }
                    }

                    RowLayout {
                        QQC2.Slider {
                            id: media_seeker
                            Layout.fillWidth: true
                        }
                    }
                }

                QQC2.ToolButton {
                    icon.name: "application-menu"
                    visible: !globalDrawer.collapsible
                    onClicked: globalDrawer.open()
                }
            }
        }

        ColumnLayout {
            id: tiles_root
            anchors.fill: parent

            Kirigami.InlineMessage {
                id: toggleMessage
                icon.name: "network-server"

                onVisibleChanged: tmr.start()

                Timer {
                    id: tmr
                    interval: 2000
                    onTriggered: toggleMessage.visible = false
                }

                Layout.fillWidth: true
                Layout.alignment: Qt.AlignTop

                visible: false

                type: Kirigami.MessageType.Positive

                text: qsTr("Positive notification")
            }
        }
    }
}
