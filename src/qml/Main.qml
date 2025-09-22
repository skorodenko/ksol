pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: mainPage

    Component.onCompleted: mpd_connector.connect()

    function message(message, type, iconName = null) {
        infoMessage.visible = false;
        infoMessage.visible = true;
        infoMessage.text = message;
        infoMessage.type = type;
        infoMessage.icon.source = iconName;
    }

    QMPDConnector {
        id: mpd_connector
        onConnectionUpdate: function (state) {
            switch (state) {
            case "connected":
                drun.playlists_list.update();
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
                drun.playlists_list.update();
            } else {
                root.message("DB Updating", Kirigami.MessageType.Warning, "dialog-warning");
            }
        }
        onGetPlaylistsResult: function (value) {
            drun.playlists_list.setQueue(value);
        }
        onStagePlaylistResult: function (value) {
            qplaylist.setQueue(value);
        }
        onPlayStateChanged: function (state) {
            switch (state) {
            case "":
            case "Stopped":
                playback_play.icon.name = "media-playback-stop";
                playback_previous.enabled = false;
                playback_next.enabled = false;
                break;
            case "Paused":
                playback_play.icon.name = "media-playback-start";
                playback_previous.enabled = true;
                playback_next.enabled = true;
                break;
            case "Playing":
                playback_play.icon.name = "media-playback-pause";
                playback_previous.enabled = true;
                playback_next.enabled = true;
                break;
            }
        }
        onTimelineUpdate: function (duration, elapsed) {
            media_seeker.to = duration;
            media_seeker.value = elapsed;
            var efm = Math.trunc(elapsed / 60).toString().padStart(2, '0');
            var efs = Math.floor(elapsed % 60).toString().padStart(2, '0');
            var dfm = Math.trunc(duration / 60).toString().padStart(2, '0');
            var dfs = Math.floor(duration % 60).toString().padStart(2, '0');
            media_duration.text = `${efm}:${efs} / ${dfm}:${dfs}`;
        }
        onActiveSongChanged: function (songPos) {
            qplaylist.activeSongPos = songPos;
        //qplaylist_view.positionViewAtRow(songPos, Qt.AlignVertical_Mask, 0.0, activeSongHighlight);
        }
        //        onSongChange: function (pl_uuid, sg_uuid) {
        //            var info = mpd_connector.getSongInfo(pl_uuid, sg_uuid);
        //            media_title.text = info.title + " | " + info.artist;
        //        }
    }

    QPlaylistModel {
        id: qplaylist
        onLayoutChanged: function () {
        //if (qplaylist.rowCount() > 0) {
        //qplaylist_view.currentIndex = 0;
        //qplaylist_view.forceActiveFocus();
        //}
        }
    }

    Connections {
        target: drun

        function onStagePlaylist(name, group) {
            mpd_connector.stagePlaylist(name, group);
        }
    }

    Connections {
        target: drun.playlists_group

        function onActiveGroupChanged(value) {
            mpd_connector.getPlaylists(value);
        }
    }

    Drun {
        id: drun
        implicitWidth: root.width * 0.8
        implicitHeight: root.height * 0.8
        anchors.centerIn: parent
    }

    Kirigami.Page {
        id: mainPage
        globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

        padding: Kirigami.Units.smallSpacing

        header: QQC2.ToolBar {
            implicitHeight: 48

            RowLayout {
                spacing: 8
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
                            text: qplaylist.activeSongTitle + " | " + qplaylist.activeSongArtist
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
                            from: 0
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
                                mpd_connector.updateDb();
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

        ColumnLayout {
            anchors.fill: parent

            Kirigami.InlineMessage {
                id: infoMessage
                visible: false
                onVisibleChanged: tmr.restart()

                Layout.fillWidth: true

                Timer {
                    id: tmr
                    interval: Kirigami.Units.humanMoment / 2
                    onTriggered: infoMessage.visible = false
                }
            }

            QQC2.HorizontalHeaderView {
                id: qplaylist_header

                z: 1
                Layout.fillWidth: true

                syncView: qplaylist_view
                textRole: "columnName"
            }

            QQC2.ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                TableView {
                    id: qplaylist_view

                    rowSpacing: Kirigami.Units.smallSpacing

                    model: qplaylist

                    //                                Connections {
                    //                                    target: control
                    //                                    function onSongChange(pl_uuid, sg_uuid) {
                    //                                        qplaylist.setActiveSong(sg_uuid);
                    //                                    }
                    //                                }

                    columnWidthProvider: function (column) {
                        return 1 / 14 * qplaylist_view.width;
                    }

                    delegate: QQC2.TableViewDelegate {
                        id: queue_delegate

                        required property string songDisplay
                        //required property real columnWidth

                        MouseArea {
                            anchors.fill: parent
                            onDoubleClicked: function () {
                                mpd_connector.playSong(parent.row);
                            }
                        }

                        Rectangle {
                            anchors.fill: parent
                            visible: qplaylist.activeSongPos === parent.row
                            color: Kirigami.Theme.neutralBackgroundColor
                        }

                        Text {
                            id: song_play_text
                            width: parent.width
                            horizontalAlignment: Qt.AlignLeft
                            color: Kirigami.Theme.textColor
                            //text: qplaylist.activeSongId === parent.songId ? "Test" : queue_delegate.songDisplay
                            text: queue_delegate.songDisplay
                            elide: Text.ElideRight
                        }
                    }
                }
            }
        }
    }
}
