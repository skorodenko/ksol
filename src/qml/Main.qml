pragma ComponentBehavior: Bound

import QtQuick 6.9
import QtQuick.Layouts 6.9
import QtQuick.Controls 6.9 as QQC2
import org.kde.kirigami 2.20 as Kirigami
import github.skorodenko.ksol 1.0


Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: mainPage

    Component.onCompleted: mpd_connector.connect()
//    Component.onDestruction: {
//        mpd_connector.disconnect();
//        qqueue.free();
//    }

    function message(message, type, iconName = null) {
        infoMessage.visible = false;
        infoMessage.visible = true;
        infoMessage.text = message;
        infoMessage.type = type;
        infoMessage.icon.source = iconName;
    }

    QMPDConnector {
        id: mpd_connector
//        onConnected: function (state) {
//            switch (state) {
//            case "connected":
//                drun.playlists_list.refresh(drun.playlists_group.active);
//                connectionStateLabel.text = "Connected";
//                connectionStateLabelBackground.color = Kirigami.Theme.positiveBackgroundColor;
//                break;
//            case "connecting":
//                connectionStateLabel.text = "Connecting";
//                connectionStateLabelBackground.color = Kirigami.Theme.neutralBackgroundColor;
//                break;
//            case "disconnected":
//                connectionStateLabel.text = "Disconnected";
//                connectionStateLabelBackground.color = Kirigami.Theme.negativeBackgroundColor;
//                break;
//            }
//        }
//        onDbUpdated: function (state) {
//            if (!!state) {
//                root.message("DB Updated", Kirigami.MessageType.Positive, "dialog-information");
//                drun.playlists_list.refresh(drun.playlists_group.active);
//            } else {
//                root.message("DB Updating", Kirigami.MessageType.Warning, "dialog-warning");
//            }
//        }
//        onStatePlay: function (state) {
//            switch (state) {
//            case "stop":
//                playback_play.icon.name = "media-playback-stop";
//                playback_previous.enabled = false;
//                playback_next.enabled = false;
//                break;
//            case "pause":
//                playback_play.icon.name = "media-playback-start";
//                playback_previous.enabled = true;
//                playback_next.enabled = true;
//                break;
//            case "play":
//                playback_play.icon.name = "media-playback-pause";
//                playback_previous.enabled = true;
//                playback_next.enabled = true;
//                break;
//            }
//        }
//        onSongChange: function (pl_uuid, sg_uuid) {
//            var info = mpd_connector.getSongInfo(pl_uuid, sg_uuid);
//            media_title.text = info.title + " | " + info.artist;
//        }
//        onQueueStage: function (queue) {
//            qqueue.queue = queue;
//        }
    }
//
//    QQueue {
//        id: qqueue
//    }
//
//    Connections {
//        target: drun
//
//        function onStagePlaylist(name, group) {
//            mpd_connector.stageQueue(name, group);
//        }
//    }
//
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

//        ColumnLayout {
//            anchors.fill: parent
//
//            Kirigami.InlineMessage {
//                id: infoMessage
//                visible: false
//                onVisibleChanged: tmr.restart()
//
//                Layout.fillWidth: true
//
//                Timer {
//                    id: tmr
//                    interval: Kirigami.Units.humanMoment / 2
//                    onTriggered: infoMessage.visible = false
//                }
//            }
//
//            QQC2.HorizontalHeaderView {
//                id: queue_hheader
//
//                z: 1
//                Layout.fillWidth: true
//
//                syncView: queue_view
//
//                delegate: QQC2.TableViewDelegate {
//                    implicitWidth: columnWidth * queue_view.width
//
//                    required property real columnWidth
//                    required property string columnName
//
//                    Kirigami.Heading {
//                        anchors.fill: parent
//                        wrapMode: Text.Wrap
//                        horizontalAlignment: Text.AlignLeft
//                        text: parent.columnName
//                        level: 3
//                    }
//                }
//            }
//
//            TableView {
//                id: queue_view
//                Layout.fillWidth: true
//                Layout.fillHeight: true
//
//                rowSpacing: Kirigami.Units.smallSpacing
//
//                model: qqueue
//
//                //                                Connections {
//                //                                    target: control
//                //                                    function onSongChange(pl_uuid, sg_uuid) {
//                //                                        qplaylist.setActiveSong(sg_uuid);
//                //                                    }
//                //                                }
//
//                delegate: QQC2.TableViewDelegate {
//                    id: queue_delegate
//                    implicitWidth: columnWidth * queue_view.width
//
//                    //required property int column
//                    required property string cellValue
//                    required property bool songActive
//                    required property real columnWidth
//
//                    //                                    MouseArea {
//                    //                                        anchors.fill: parent
//                    //                                        onDoubleClicked: function () {
//                    //                                            control.stagePlaylist(itemDelegate.pl_uuid, pli_delegate.sgUuid);
//                    //                                        }
//                    //                                    }
//                    //
//
//                    function propagateWidthChange() {
//                        model.columnWidth = width / parent.width;
//                    }
//
//                    onWidthChanged: Qt.callLater(propagateWidthChange)
//
//                    Kirigami.Heading {
//                        id: song_play_text
//                        width: parent.width
//                        horizontalAlignment: Qt.AlignLeft
//                        text: queue_delegate.cellValue
//                        elide: Text.ElideRight
//                        level: 3
//                    }
//                }
//            }
//        }
    }
}
