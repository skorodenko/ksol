pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

QQC2.Control {
    id: root
    implicitHeight: 18

    property alias repeater: repeater

    required property var model
    required property int columnCount
    required property real tableWidth
    property var color: "#4f4f4f"

    Row {
        Repeater {
            id: repeater

            model: root.columnCount

            delegate: Rectangle {
                id: delegate
                width: root.tableWidth * root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnWidth)
                height: root.height
                color: root.color

                required property int index

                Text {
                    text: root.model.headerData(parent.index, Qt.Horizontal, QPlaylistModel.ColumnName)
                    color: "white"
                    anchors.centerIn: parent
                }

                Rectangle {
                    id: dragHandle
                    visible: false
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    width: 2
                    color: "#8d8d8d"
                }

                MouseArea {
                    hoverEnabled: true
                    anchors.fill: parent
                    onEntered: dragHandle.visible = true
                    onExited: dragHandle.visible = false
                }
            }
        }
    }
}
