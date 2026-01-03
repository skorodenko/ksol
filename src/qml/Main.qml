pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: mainPage

    Component.onCompleted: {
        if (QSettingsModel.initWizard) {
            initWizardDelay.start();
        } else {
            mpd_connector.connect();
        }
    }

    Timer {
        id: initWizardDelay
        interval: 150
        onTriggered: {
            initWizard.visible = true;
        }
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
            qplaylist_view.selectionTimeout = false;
            selectionTimeoutTimer.stop();
            filterSearchBox.visible = false;
            drun.visible = false;
        }
    }

    Shortcut {
        id: drun_open
        sequences: ["f"]
        enabled: !filterSearchBox.visible && !drun.visible
        onActivated: function () {
            mpd_connector.getPlaylists(drun.activeGroup);
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
                connectionStateLabel.text = "Connected";
                connectionStateLabelBackground.color = Kirigami.Theme.positiveBackgroundColor;
                connectionStateRestart.visible = false;
                var songPos = mpd_connector.activeSongPosition;
                qplaylist_view.selectionModel.setCurrentIndex(qplaylist.index(songPos, 0), ItemSelectionModel.Rows);
                break;
            case "connecting" || "disconnected":
                connectionStateLabel.text = "Connecting";
                connectionStateLabelBackground.color = Kirigami.Theme.neutralBackgroundColor;
                connectionStateRestart.visible = false;
                break;
            case "disconnected-action":
                connectionStateLabel.text = "Disconnected";
                connectionStateLabelBackground.color = Kirigami.Theme.negativeBackgroundColor;
                connectionStateRestart.visible = true;
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
            mpd_connector.sortPlaylist(QSettingsModel.sortColumn, QSettingsModel.sortOrder);
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
            if (!media_seeker.pressed) {
                media_seeker.value = elapsed;
            }
            var efm = Math.trunc(elapsed / 60).toString().padStart(2, '0');
            var efs = Math.floor(elapsed % 60).toString().padStart(2, '0');
            var dfm = Math.trunc(duration / 60).toString().padStart(2, '0');
            var dfs = Math.floor(duration % 60).toString().padStart(2, '0');
            media_duration.text = `${efm}:${efs} / ${dfm}:${dfs}`;
        }
        onBitrateUpdate: function (bitrate) {
            bitrateText.text = `${bitrate} b/s`;
        }
        onActiveSongChanged: function () {
            var songPos = mpd_connector.activeSongPosition;
            qplaylist_view.selectionModel.setCurrentIndex(qplaylist.index(songPos, 0), ItemSelectionModel.Rows);
            qplaylist_view.positionViewAtRow(songPos, Qt.AlignVCenter, 0);
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
        onAlbumArtUpdate: function (art) {
            tableBackground.source = art ? "file:" + art : "";
        }
    }

    QPlaylistModel {
        id: qplaylist
        filter: filterSearch.text
    }

    Connections {
        target: QSettingsModel

        function onUpdateSortColumn() {
            mpd_connector.sortPlaylist(QSettingsModel.sortColumn, QSettingsModel.sortOrder);
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

    InitWizard {
        id: initWizard
        width: 0.5 * root.width
        height: 0.5 * root.height

        onFinished: {
            mpd_connector.connect();
        }
    }

    Settings {
        id: settings
        visible: false
        onBackRequest: function (restartMpd) {
            root.pageStack.replace(mainPage);
            if (restartMpd) {
                mpd_connector.connect();
            }
        }
    }

    About {
        id: aboutPage
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
                        text: mpd_connector.activeSongTitle + " | " + mpd_connector.activeSongArtist
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
                    QQC2.MenuItem {
                        text: qsTr("Settings")
                        icon.name: "settings-configure"
                        onClicked: {
                            root.pageStack.replace(settings);
                        }
                    }
                    QQC2.MenuItem {
                        text: qsTr("About")
                        icon.name: "info"
                        onClicked: {
                            aboutPage.open();
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

            QQC2.Control {
                anchors.left: parent.left
                anchors.leftMargin: Kirigami.Units.largeSpacing
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                implicitWidth: connectionStateLabel.implicitWidth + connectionStateRestart.implicitWidth * connectionStateRestart.visible

                background: Rectangle {
                    id: connectionStateLabelBackground
                    Kirigami.Theme.inherit: false
                    Kirigami.Theme.colorSet: Kirigami.Theme.Window
                    color: Kirigami.Theme.negativeBackgroundColor
                }

                QQC2.Label {
                    id: connectionStateLabel
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    topPadding: 4
                    leftPadding: Kirigami.Units.smallSpacing
                    rightPadding: Kirigami.Units.smallSpacing

                    text: "Disconnected"
                }

                QQC2.Button {
                    id: connectionStateRestart
                    visible: false
                    flat: true
                    focusPolicy: Qt.NoFocus
                    icon.name: "media-repeat-all"
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.left: connectionStateLabel.right
                    onClicked: {
                        mpd_connector.connect();
                    }
                }
            }

            QQC2.Control {
                id: bitrateIndicator
                implicitWidth: bitrateText.implicitWidth
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.right: shuffleButton.left

                QQC2.Label {
                    id: bitrateText
                    text: "0 b/s"
                    anchors.centerIn: parent
                    topPadding: 4
                    leftPadding: Kirigami.Units.smallSpacing
                    rightPadding: Kirigami.Units.smallSpacing
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
                onClicked: {
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
            anchors.top: parent.top
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

            implicitHeight: 22
            anchors.top: infoMessage.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            z: 1

            model: qplaylist
            color: "#32363b"
            columnCount: qplaylist.columnCount()

            onColumnWidthChanged: {
                Qt.callLater(qplaylist_view.forceLayout);
            }
        }

        QQC2.ScrollBar {
            id: scrollBar
            width: visible ? implicitWidth : 0
            anchors.top: qplaylist_header.bottom
            anchors.right: parent.right
            anchors.bottom: filterSearchBox.top
            orientation: Qt.Vertical
        }

        BackgroundImage {
            id: tableBackground
            anchors.top: infoMessage.bottom
            anchors.bottom: filterSearchBox.top
            anchors.left: parent.left
            anchors.right: parent.right
        }

        TableView {
            id: qplaylist_view

            anchors.top: qplaylist_header.bottom
            anchors.left: parent.left
            anchors.right: scrollBar.left
            anchors.bottom: filterSearchBox.top
            anchors.topMargin: rowSpacing
            rowSpacing: Kirigami.Units.smallSpacing

            property bool selectionTimeout: false
            property int songPos: 0

            model: qplaylist

            boundsMovement: Flickable.StopAtBounds
            boundsBehavior: Flickable.StopAtBounds

            reuseItems: true
            keyNavigationEnabled: true
            selectionBehavior: TableView.SelectRows
            selectionMode: TableView.SingleSelection

            focus: true
            onFocusChanged: if (!focus) {
                Qt.callLater(forceActiveFocus);
            }

            Timer {
                id: selectionTimeoutTimer
                interval: 2500
                onTriggered: qplaylist_view.selectionTimeout = false
            }

            onCurrentRowChanged: {
                qplaylist_view.selectionTimeout = true;
                selectionTimeoutTimer.restart();
            }

            Keys.onReturnPressed: function () {
                if (qplaylist_view.selectionTimeout) {
                    var index = qplaylist_view.selectionModel.currentIndex;
                    var songId = qplaylist.data(index, QPlaylistModel.SongId);
                    mpd_connector.playSong(songId);
                }
            }

            Keys.forwardTo: [filterSearch]

            columnWidthProvider: function (column) {
                var item = qplaylist_header.repeater.itemAt(column);
                if (column == qplaylist_header.lastVisibleColumn) {
                    return item.width - 4 - scrollBar.implicitWidth * scrollBar.visible;
                } else {
                    return item.visible ? item.width + 2 : 0; // 2 is splitter width (which is not acounted in delegate width)
                }
            }

            selectionModel: ItemSelectionModel {}

            QQC2.ScrollBar.vertical: scrollBar

            delegate: Item {
                id: delegate
                implicitHeight: 18

                required property int row
                required property var songId
                required property string songDisplay
                property bool activeSongItem: mpd_connector.activeSongId == songId

                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        qplaylist_view.selectionModel.setCurrentIndex(qplaylist.index(delegate.row, 0), ItemSelectionModel.Rows);
                    }
                    onDoubleClicked: {
                        mpd_connector.playSong(parent.songId);
                    }
                }

                Rectangle {
                    anchors.fill: parent
                    visible: delegate.activeSongItem
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
                    anchors.fill: parent
                    anchors.leftMargin: Kirigami.Units.smallSpacing
                    anchors.verticalCenter: parent.verticalCenter
                    horizontalAlignment: Qt.AlignLeft
                    color: Kirigami.Theme.textColor
                    text: delegate.songDisplay
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

                onTextEdited: {
                    qplaylist_view.selectionModel.setCurrentIndex(qplaylist.index(0, 0), ItemSelectionModel.Rows);
                    qplaylist_view.positionViewAtRow(0, Qt.AlignTop, 0);
                    qplaylist_view.selectionTimeout = true;
                    selectionTimeoutTimer.restart();
                }

                Keys.onPressed: function (event) {
                    if (!(event.key > Qt.Key_Space || event.key < Qt.Key_AsciiTilde || event.key === Qt.Key_Backspace)) {
                        event.accepted = true;
                    }
                }
            }
        }
    }
}
