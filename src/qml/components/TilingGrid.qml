pragma ComponentBehavior: Bound
pragma NativeMethodBehavior: AcceptThisObject

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQuick.Controls.impl
import org.kde.kirigami as Kirigami
import models 1.0

QQC2.Control {
    id: control

    signal songChange(var pl_uuid, var sg_uuid)

    property alias model: repeater.model
    property real cellWidth: control.width / 2
    property real cellHeight: control.height / 2

    required property var stagePlaylist

    QTilingStack {
        id: tiling_stack

        onTileGridUpdate: function (positions) {
            for (var i = 0; i < tiling_stack.size; i++) {
                var currentPos = positions[i];
                var currentTile = repeater.itemAt(i);
                currentTile.wSpan = currentPos[0];
                currentTile.hSpan = currentPos[1];
            }
        }
    }

    contentItem: GridLayout {
        id: grid

        rows: 2
        columns: 2
        rowSpacing: Kirigami.Units.largeSpacing
        columnSpacing: Kirigami.Units.largeSpacing

        Repeater {
            id: repeater
            model: tiling_stack

            delegate: Rectangle {
                id: itemDelegate

                required property var pl_uuid
                required property var sg_uuid
                required property string name
                required property var playlist
                required property int tileIndex
                required property var tilingStruct

                property int wSpan: tilingStruct[0]
                property int hSpan: tilingStruct[1]

                Layout.columnSpan: wSpan
                Layout.rowSpan: hSpan
                Layout.fillWidth: true
                Layout.fillHeight: true

                Component.onCompleted: function () {
                    itemDelegate.scale = 1;
                }

                Behavior on scale {
                    NumberAnimation {
                        easing.type: Easing.OutCubic
                        duration: Kirigami.Units.longDuration
                    }
                }

                QPlaylist {
                    id: qplaylist
                    playlist: itemDelegate.playlist
                }

                radius: Kirigami.Units.cornerRadius
                scale: 0.4
                color: Kirigami.Theme.alternateBackgroundColor

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.largeSpacing

                    RowLayout {
                        Layout.fillWidth: true

                        Kirigami.Heading {
                            Layout.alignment: Qt.AlignLeft
                            horizontalAlignment: Text.AlignHCenter
                            text: itemDelegate.name
                            wrapMode: Text.Wrap
                            level: 3
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        QQC2.Button {
                            icon.name: "window-close"
                            Layout.alignment: Qt.AlignRight
                            onClicked: function () {
                                tiling_stack.deleteTile(itemDelegate.tileIndex);
                            }
                        }
                    }

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

                            delegate: Item {
                                id: pli_delegate
                                implicitHeight: 20
                                implicitWidth: TableView.view.width / qplaylist.columnCount()

                                required property int row
                                required property int column
                                required property var sgUuid
                                required property string display

                                Connections {
                                    target: control
                                    function onSongChange(pl_uuid, sg_uuid) {
                                        if (column == 0 && sgUuid == sg_uuid) {
                                            song_play_icon.source = "media-playback-start";
                                        } else {
                                            song_play_icon.source = "";
                                        }
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onDoubleClicked: function () {
                                        control.stagePlaylist(itemDelegate.pl_uuid, pli_delegate.sgUuid);
                                    }
                                }

                                RowLayout {
                                    clip: true
                                    Kirigami.Icon {
                                        id: song_play_icon
                                        implicitHeight: song_play_text.contentHeight
                                        //source: "media-playback-start"
                                    }
                                    QQC2.Label {
                                        id: song_play_text
                                        clip: true
                                        //text: "media-playback-stop"
                                        horizontalAlignment: Qt.AlignLeft
                                        text: pli_delegate.display
                                        //anchors.fill: parent
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
