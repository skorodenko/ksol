import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import "components" as CC
import controllers 1.0
import models 1.0

Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: mainPage

    Component.onCompleted: mpd_connector.connect()
    Component.onDestruction: mpd_connector.disconnect()

    function message(message, type, iconName = null) {
        infoMessage.visible = false;
        infoMessage.visible = true;
        infoMessage.text = message;
        infoMessage.type = type;
        infoMessage.icon.source = iconName;
    }

    QPlaylistsList {
        id: playlists_list
    }

    MPDConnector {
        id: mpd_connector
        onConnected: function (state) {
            //root.message("Connected to server", Kirigami.MessageType.Positive, "network-server");
            playlists_list.refresh(playlists_group.active);
        }
        onDbUpdated: function (state) {
            if (!!state) {
                root.message("DB Updated", Kirigami.MessageType.Positive, "dialog-information");
                playlists_list.refresh(playlists_group.active);
            } else {
                root.message("DB Updating", Kirigami.MessageType.Warning, "dialog-warning");
            }
        }
        onStatePlay: function (state) {
            switch (state) {
            case "stop":
                playback_play.icon.name = "media-playback-stop";
                playback_previous.enabled = false;
                playback_next.enabled = false;
                break;
            case "pause":
                playback_play.icon.name = "media-playback-start";
                playback_previous.enabled = true;
                playback_next.enabled = true;
                break;
            case "play":
                playback_play.icon.name = "media-playback-pause";
                playback_previous.enabled = true;
                playback_next.enabled = true;
                break;
            }
        }
    }

    QPlaylistsGroupModel {
        id: playlists_group
        onGroupChanged: function (group) {
            playlists_list.refresh(group);
        }
    }

    menuBar: QQC2.MenuBar {
        QQC2.Menu {
            title: qsTr("&Server")
            QQC2.Action {
                text: qsTr("&Refresh DB")
                onTriggered: function () {
                    mpd_connector.refresh_db();
                }
            }
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

                    MouseArea {
                        id: ma
                        anchors.fill: parent
                        acceptedButtons: Qt.LeftButton

                        onDoubleClicked: function (mouse) {
                            if (mouse.button == Qt.LeftButton) {
                                tiling_grid.model.addTile(name);
                            }
                        }
                    }

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
                        id: playback_previous
                        flat: true
                        enabled: false
                        icon.name: "media-skip-backward"
                        onClicked: function () {
                            mpd_connector.play_previous();
                        }
                    }
                    QQC2.Button {
                        id: playback_play
                        flat: true
                        icon.name: "media-playback-stop"
                        onClicked: function () {
                            mpd_connector.play_toggle();
                        }
                    }
                    QQC2.Button {
                        id: playback_next
                        flat: true
                        enabled: false
                        icon.name: "media-skip-forward"
                        onClicked: function () {
                            mpd_connector.play_next();
                        }
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

        footer: QQC2.ToolBar {
            implicitHeight: 18
            background: Rectangle {
                Kirigami.Theme.inherit: false
                Kirigami.Theme.colorSet: Kirigami.Theme.Header
                color: Kirigami.Theme.backgroundColor

                RowLayout {
                    QQC2.Label {
                        text: "Test"
                    }
                }
            }
        }

        ColumnLayout {
            id: tiles_root
            anchors.fill: parent

            Kirigami.InlineMessage {
                id: infoMessage
                visible: false
                onVisibleChanged: tmr.restart()

                Timer {
                    id: tmr
                    interval: Kirigami.Units.humanMoment / 2
                    onTriggered: infoMessage.visible = false
                }

                Layout.fillWidth: true
                Layout.alignment: Qt.AlignTop
            }

            CC.TilingGrid {
                id: tiling_grid

                stagePlaylist: (pl_uuid, sg_uuid) => mpd_connector.stagePlaylist(pl_uuid, sg_uuid)

                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }
}
