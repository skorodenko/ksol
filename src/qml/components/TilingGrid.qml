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
                required property string name
                required property int tileIndex
                required property var tilingStruct

                property int wSpan: tilingStruct[0]
                property int hSpan: tilingStruct[1]

                Layout.columnSpan: wSpan
                Layout.rowSpan: hSpan
                Layout.fillWidth: true
                Layout.fillHeight: true

                Component.onCompleted: function() {
                    scale = 1;
                }

                Behavior on scale {
                    NumberAnimation {
                        easing.type: Easing.OutCubic
                        duration: 150
                    }
                }

                color: "blue"
                radius: 4
                border.width: 2
                border.color: "red"
                scale: 0.4

                Text {
                    anchors.centerIn: parent
                    text: parent.tileIndex
                }
            }
        }
    }
}
