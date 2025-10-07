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
    signal columnWidthChanged

    required property var model
    required property int columnCount
    required property real tableWidth
    property var color: "#32363b"

    Row {
        Repeater {
            id: repeater

            model: root.columnCount

            delegate: Rectangle {
                id: delegate
                width: splitter.x + 6
                height: root.height
                color: root.color

                required property int index

                onWidthChanged: {
                    root.columnWidthChanged();
                }

                Text {
                    color: "wheat"
                    text: root.model.headerData(parent.index, Qt.Horizontal, QPlaylistModel.ColumnName)
                    anchors.centerIn: parent
                }

                Item {
                    id: splitter
                    x: root.tableWidth * root.model.headerData(parent.index, Qt.Horizontal, QPlaylistModel.ColumnWidth) - 6
                    width: 6
                    height: delegate.height

                    DragHandler {
                        id: dragHandler
                        target: splitter
                        yAxis.enabled: false
                        onActiveChanged: {
                            if (!active) {
                                if (splitter.x <= 36) {
                                    splitter.x = 36;
                                }
                            }
                        }
                    }
                    
                    Rectangle {
                        id: splitterRect
                        width: 2
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        anchors.horizontalCenter: parent.horizontalCenter
                        color: "#595d61"
                    }
                }
            }
        }
    }
}
