pragma ComponentBehavior: Bound

import QtQuick 6.9
import QtQuick.Layouts 6.9
import QtQuick.Controls 6.9 as QQC2
import org.kde.kirigami 2.20 as Kirigami
import src.qml 1.0
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

    MPDConnector {
        id: mpd_connector
        onConnected: function (state) {
            switch (state) {
            case "connected":
                drun.playlists_list.refresh(drun.playlists_group.active);
                connectionStateLabel.text = "Connected";
                connectionStateLabelBackground.color = Kirigami.Theme.positiveBackgroundColor;
                break;
            case "connecting":
                connectionStateLabel.text = "Connecting";
                connectionStateLabelBackground.color = Kirigami.Theme.neutralBackgroundColor;
                break;
            case "disconnected":
                connectionStateLabel.text = "Disconnected";
                connectionStateLabelBackground.color = Kirigami.Theme.negativeBackgroundColor;
                break;
            }
        }
        onDbUpdated: function (state) {
            if (!!state) {
                root.message("DB Updated", Kirigami.MessageType.Positive, "dialog-information");
                drun.playlists_list.refresh(drun.playlists_group.active);
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
        onSongChange: function (pl_uuid, sg_uuid) {
            var info = mpd_connector.getSongInfo(pl_uuid, sg_uuid);
            media_title.text = info.title + " | " + info.artist;
        }
    }

    QPlaylist {
        id: qplaylist
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
                            mpd_connector.playPrevious();
                        }
                    }
                    QQC2.Button {
                        id: playback_play
                        flat: true
                        icon.name: "media-playback-stop"
                        onClicked: function () {
                            mpd_connector.playToggle();
                        }
                    }
                    QQC2.Button {
                        id: playback_next
                        flat: true
                        enabled: false
                        icon.name: "media-skip-forward"
                        onClicked: function () {
                            mpd_connector.playNext();
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
                    visible: true

                    onClicked: {
                        globalMenu.popup();
                    }

                    QQC2.Menu {
                        id: globalMenu
                        QQC2.MenuItem {
                            text: qsTr("Refresh DB")
                            icon.name: "server-database"
                            onClicked: {
                                mpd_connector.refreshDb();
                            }
                        }
                    }
                }
            }
        }

        footer: QQC2.ToolBar {
            id: footer
            implicitHeight: 20
            background: Rectangle {
                height: parent.height
                Kirigami.Theme.inherit: false
                Kirigami.Theme.colorSet: Kirigami.Theme.Header
                color: Kirigami.Theme.backgroundColor
                RowLayout {
                    anchors.fill: parent
                    QQC2.Label {
                        id: connectionStateLabel
                        Layout.fillHeight: true
                        Layout.leftMargin: 2 * Kirigami.Units.largeSpacing
                        text: "Disconnected"
                        leftPadding: Kirigami.Units.smallSpacing
                        rightPadding: Kirigami.Units.smallSpacing
                        background: Rectangle {
                            id: connectionStateLabelBackground
                            Kirigami.Theme.inherit: false
                            Kirigami.Theme.colorSet: Kirigami.Theme.Window
                            color: Kirigami.Theme.negativeBackgroundColor
                        }
                    }
                }
            }
        }

        AppShortcuts {
            id: shortcuts
            Connections {
                target: shortcuts.drun_open
                function onActivated() {
                    drun.visible = true;
                }
            }

            Connections {
                target: shortcuts.drun_close
                function onActivated() {
                    drun.visible = false;
                    drun.group_filter = "";
                }
            }

            Connections {
                target: shortcuts.drun_group1
                function onActivated() {
                    drun.group_repeater.itemAt(0).click();
                }
            }

            Connections {
                target: shortcuts.drun_group2
                function onActivated() {
                    drun.group_repeater.itemAt(1).click();
                }
            }

            Connections {
                target: shortcuts.drun_group3
                function onActivated() {
                    drun.group_repeater.itemAt(2).click();
                }
            }

            Connections {
                target: shortcuts.drun_group4
                function onActivated() {
                    drun.group_repeater.itemAt(3).click();
                }
            }
        }

        Drun {
            id: drun
            implicitWidth: root.width * 0.8
            implicitHeight: root.height * 0.8
            anchors.centerIn: parent
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

            QQC2.Control {
                id: control

                property real cellWidth: control.width / 2
                property real cellHeight: control.height / 2

                Rectangle {
                    id: itemDelegate

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.largeSpacing

                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            QQC2.HorizontalHeaderView {
                                id: playlist_hheader
                                anchors.left: playlist_view.left
                                anchors.top: parent.top
                                syncView: playlist_view

                                delegate: Rectangle {
                                    color: Kirigami.Theme.backgroundColor
                                    implicitHeight: 20
                                    implicitWidth: TableView.view.width / qplaylist.columnCount()

                                    required property string display

                                    QQC2.Label {
                                        text: parent.display
                                        anchors.fill: parent
                                        horizontalAlignment: Text.AlignLeft
                                        clip: true
                                    }
                                }
                            }

                            TableView {
                                id: playlist_view
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.bottom: parent.bottom
                                anchors.top: playlist_hheader.bottom

                                model: qplaylist

                                //                                Connections {
                                //                                    target: control
                                //                                    function onSongChange(pl_uuid, sg_uuid) {
                                //                                        qplaylist.setActiveSong(sg_uuid);
                                //                                    }
                                //                                }

                                delegate: Item {
                                    id: pli_delegate
                                    implicitHeight: 20
                                    implicitWidth: TableView.view.width / qplaylist.columnCount()

                                    required property int column
                                    //required property bool activeSong
                                    //required property string display

                                    //                                    MouseArea {
                                    //                                        anchors.fill: parent
                                    //                                        onDoubleClicked: function () {
                                    //                                            control.stagePlaylist(itemDelegate.pl_uuid, pli_delegate.sgUuid);
                                    //                                        }
                                    //                                    }

                                    RowLayout {
                                        clip: true
                                        Kirigami.Icon {
                                            id: song_play_icon
                                            implicitHeight: song_play_text.contentHeight
                                            source: "media-playback-start"
                                            visible: pli_delegate.activeSong
                                        }
                                        QQC2.Label {
                                            id: song_play_text
                                            clip: true
                                            horizontalAlignment: Qt.AlignLeft
                                            text: pli_delegate.display
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
