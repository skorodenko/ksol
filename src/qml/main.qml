import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import org.kde.breeze as Breeeze
import controllers 1.0
import models 1.0

Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: mainPage

    Component.onCompleted: mpd_connector.connect()
    Component.onDestruction: mpd_connector.disconnect()

    MPDConnector {
        id: mpd_connector
        onConnected: function (state) {
            toggleMessage.visible = true;
            playlists_list.refresh(playlists_group.active);
        }
    }

    QPlaylistsList {
        id: playlists_list
    }

    QPlaylistsGroupModel {
        id: playlists_group
        onGroupChanged: function (group) {
            playlists_list.refresh(group);
        }
    }

    globalDrawer: Kirigami.GlobalDrawer {
        id: globalDrawer
        title: "Global menu"

        edge: Qt.RightEdge
        handleVisible: false

        contentItem: Kirigami.HeaderFooterLayout {
            id: mainLayout

            anchors {
                fill: parent
                topMargin: globalDrawer.collapsed && !showHeaderWhenCollapsed ? -contentItem.y : 0
            }

            Behavior on anchors.topMargin {
                NumberAnimation {
                    duration: Kirigami.Units.longDuration
                    easing.type: Easing.InOutQuad
                }
            }

            header: QQC2.ComboBox {
                textRole: "name"
                valueRole: "value"
                Layout.fillWidth: true
                model: playlists_group
                onActivated: playlists_group.setActive(currentValue)
                Component.onCompleted: currentIndex = playlists_group.active
            }

            contentItem: ListView {
                model: playlists_list
                Layout.fillWidth: true
                Layout.fillHeight: true

                implicitWidth: Math.min(Kirigami.Units.gridUnit * 20, globalDrawer.parent.width * 0.8)

                QQC2.ScrollBar.vertical: QQC2.ScrollBar {
                    policy: QQC2.ScrollBar.AlwaysOn
                }

                delegate: Item {
                    height: 30
                    width: ListView.view.width

                    required property string name

                    QQC2.Label {
                        text: parent.name
                        font.pixelSize: 14
                        elide: Text.ElideRight
                        anchors.leftMargin: 20
                        anchors.rightMargin: 20
                        anchors.left: parent.left
                        anchors.right: parent.right
                    }
                }
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
