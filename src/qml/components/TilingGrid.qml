import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import models 1.0

QQC2.Control {
    id: control

    property alias model: repeater.model
    property real cellWidth: control.width / 2
    property real cellHeight: control.height / 2

    QTilingStack {
        id: tiling_stack

        onTileAddStart: function (positions) {
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
        rowSpacing: 6
        columnSpacing: 6

        Repeater {
            id: repeater
            model: tiling_stack

            delegate: Rectangle {
                id: itemDelegate
                required property var uuid
                required property string name
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
                        duration: 150
                    }
                }

                radius: 4
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
                            onClicked: function() {
                                tiling_stack.deleteTile(itemDelegate.tileIndex);   
                            }
                        }
                    }

                    ListView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                    }
                }
            }
        }
    }
}
