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

    Component.onCompleted: {
        mpd_connector.connect();
        mpd_connector.syncState();
    }

    function message(message, type, iconName = null) {
        infoMessage.visible = false;
        infoMessage.visible = true;
        infoMessage.text = message;
        infoMessage.type = type;
        infoMessage.icon.source = iconName;
    }

    Shortcut {
        sequences: ["Escape"]
        onActivated: function () {
            filterSearchBox.visible = false;
            drun.visible = false;
        }
    }

    Shortcut {
        id: drun_open
        sequences: ["f"]
        enabled: !filterSearchBox.visible && !drun.visible
        onActivated: function () {
            drun.visible = true;
        }
    }

    Shortcut {
        sequences: ["/"]
        context: Qt.ApplicationShortcut
        enabled: !filterSearchBox.visible && !drun.visible
        onActivated: function () {
            filterSearchBox.visible = true;
        }
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
        onActiveSongChanged: function (songId) {
            qplaylist.activeSongId = songId;
        }
        onUpdateOptions: function () {
            var shuffle = mpd_connector.shuffle;
            var repeat = mpd_connector.repeat;
            var single = mpd_connector.single;
            if (shuffle) {
                shuffleButton.icon.name = "media-playlist-shuffle";
            } else {
                shuffleButton.icon.name = "media-playlist-normal";
            }
            if (repeat == false) {
                repeatButton.icon.name = "media-repeat-none";
            } else if (single) {
                repeatButton.icon.name = "media-repeat-single";
            } else {
                repeatButton.icon.name = "media-repeat-all";
            }
        }
    }

    QPlaylistModel {
        id: qplaylist

        filter: filterSearch.text

        onUpdateSort: {
            mpd_connector.sortPlaylist(qplaylist.sortColumn, qplaylist.sortOrder);
        }
    }

    Connections {
        target: drun

        function onStagePlaylist(name, group) {
            mpd_connector.stagePlaylist(name, group);
            mpd_connector.sortPlaylist(qplaylist.sortColumn, qplaylist.sortOrder);
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
                    focusPolicy: Qt.NoFocus
                    icon.name: "media-skip-backward"
                    onClicked: function () {
                        mpd_connector.playPrevious();
                    }
                }
                QQC2.Button {
                    id: playback_play
                    flat: true
                    focusPolicy: Qt.NoFocus
                    icon.name: "media-playback-stop"
                    onClicked: function () {
                        mpd_connector.playToggle();
                    }
                }
                QQC2.Button {
                    id: playback_next
                    flat: true
                    enabled: false
                    focusPolicy: Qt.NoFocus
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
                        focusPolicy: Qt.NoFocus
                        Layout.fillWidth: true

                        onMoved: {
                            mpd_connector.playSeek(media_seeker.value);
                        }
                    }
                }
            }

            QQC2.ToolButton {
                icon.name: "application-menu"
                visible: true
                focusPolicy: Qt.NoFocus

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
        implicitHeight: 22
        padding: 0

        background: Rectangle {
            Kirigami.Theme.inherit: false
            Kirigami.Theme.colorSet: Kirigami.Theme.Header
            color: Kirigami.Theme.backgroundColor
        }

        Item {
            anchors.fill: parent

            QQC2.Label {
                id: connectionStateLabel
                anchors.left: parent.left
                anchors.leftMargin: Kirigami.Units.largeSpacing
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                topPadding: 2
                leftPadding: Kirigami.Units.smallSpacing
                rightPadding: Kirigami.Units.smallSpacing

                text: "Disconnected"

                background: Rectangle {
                    id: connectionStateLabelBackground
                    Kirigami.Theme.inherit: false
                    Kirigami.Theme.colorSet: Kirigami.Theme.Window
                    color: Kirigami.Theme.negativeBackgroundColor
                }
            }

            QQC2.Button {
                id: shuffleButton
                focusPolicy: Qt.NoFocus
                icon.name: "media-playlist-normal"
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.right: repeatButton.left
                onClicked: {
                    mpd_connector.shuffleToggle(mpd_connector.shuffle);
                }
            }

            QQC2.Button {
                id: repeatButton
                focusPolicy: Qt.NoFocus
                icon.name: "media-repeat-all"
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.right: parent.right
                anchors.rightMargin: Kirigami.Units.largeSpacing
                onClicked: function () {
                    mpd_connector.repeatToggle(mpd_connector.repeat, mpd_connector.single);
                }
            }
        }
    }

    Kirigami.Page {
        id: mainPage

        padding: 0
        globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

        Kirigami.InlineMessage {
            id: infoMessage

            visible: false
            anchors.left: parent.left
            anchors.right: parent.right
            onVisibleChanged: tmr.restart()

            Timer {
                id: tmr
                interval: Kirigami.Units.humanMoment / 2
                onTriggered: infoMessage.visible = false
            }
        }

        PlaylistHeader {
            id: qplaylist_header

            anchors.top: infoMessage.bottom
            anchors.left: parent.left
            anchors.right: parent.right

            implicitHeight: 18
            z: 1

            model: qplaylist
            color: "#32363b"
            columnCount: qplaylist.columnCount()
            tableWidth: qplaylist_view.width

            onColumnWidthChanged: {
                qplaylist_view.forceLayout();
            }
        }

        TableView {
            id: qplaylist_view

            anchors.top: qplaylist_header.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: filterSearchBox.top

            property bool selectionTimeout: false

            rowSpacing: Kirigami.Units.smallSpacing
            model: qplaylist

            keyNavigationEnabled: true
            pointerNavigationEnabled: true
            selectionBehavior: TableView.SelectRows
            selectionMode: TableView.SingleSelection

            focus: true
            onFocusChanged: if (!focus) {
                Qt.callLater(forceActiveFocus);
            }

            Timer {
                id: selectionTimeoutTimer
                interval: 1000
                onTriggered: qplaylist_view.selectionTimeout = false
            }

            onCurrentRowChanged: {
                qplaylist_view.selectionTimeout = true;
                selectionTimeoutTimer.restart();
            }

            Keys.onReturnPressed: function () {
                var index = qplaylist_view.selectionModel.currentIndex;
                var songId = qplaylist.data(index, QPlaylistModel.SongId);
                mpd_connector.playSong(songId);
            }

            Keys.forwardTo: [filterSearch]

            columnWidthProvider: function (column) {
                return qplaylist_header.repeater.itemAt(column).width;
            }

            selectionModel: ItemSelectionModel {
                model: qplaylist_view.model
                onCurrentRowChanged: function (current, previous) {}
            }

            QQC2.ScrollBar.horizontal: QQC2.ScrollBar {
                id: hbar
            }
            QQC2.ScrollBar.vertical: QQC2.ScrollBar {
                id: vbar
            }

            delegate: Item {
                id: queue_delegate
                implicitHeight: 18

                required property int row
                required property var songId
                required property string songDisplay

                MouseArea {
                    anchors.fill: parent
                    onDoubleClicked: function () {
                        mpd_connector.playSong(parent.songId);
                    }
                }

                Rectangle {
                    anchors.fill: parent
                    visible: qplaylist.activeSongId == parent.songId
                    color: Kirigami.Theme.focusColor
                }

                Rectangle {
                    z: -1
                    anchors.fill: parent
                    visible: qplaylist_view.selectionTimeout && qplaylist_view.currentRow == parent.row
                    color: Kirigami.Theme.activeBackgroundColor
                }

                Text {
                    id: song_play_text
                    width: parent.width
                    anchors.verticalCenter: parent.verticalCenter
                    horizontalAlignment: Qt.AlignLeft
                    color: Kirigami.Theme.textColor
                    text: queue_delegate.songDisplay
                    elide: Text.ElideRight
                }
            }
        }

        RowLayout {
            id: filterSearchBox

            visible: false
            height: 0

            onVisibleChanged: {
                if (visible) {
                    filterSearchBox.height = 28;
                    filterSearchBox.enabled = true;
                } else {
                    filterSearchBox.height = 0;
                    filterSearch.text = "";
                    filterSearchBox.enabled = false;
                }
            }

            anchors {
                left: parent.left
                right: parent.right
                bottom: parent.bottom
            }

            QQC2.TextField {
                id: filterSearch
                focusPolicy: Qt.NoFocus
                placeholderText: "Filter by song title/artist ..."
                Layout.alignment: Qt.AlignLeft
                Layout.preferredWidth: parent.width
                Layout.preferredHeight: parent.height
                Keys.onPressed: function (event) {
                    if (!(event.key > Qt.Key_Space || event.key < Qt.Key_AsciiTilde || event.key === Qt.Key_Backspace)) {
                        event.accepted = true;
                    }
                }
            }
        }
    }
}
