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
            console.info(positions);
            for (var i = 0; i < tiling_stack.size; i++) {
                var currentPos = positions[i];
                var currentTile = repeater.itemAt(i);
                currentTile.wSpan = currentPos[0];
                currentTile.hSpan = currentPos[1];
            }
        }
    }

    contentItem: Grid {
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

                width: control.cellWidth * wSpan
                height: control.cellHeight * hSpan

                color: "blue"
                border.width: 2
                border.color: "red"

                Text {
                    anchors.centerIn: parent
                    text: parent.tileIndex
                }
            }
        }
    }
}
