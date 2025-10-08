pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Flickable {
    id: root

    property alias repeater: repeater
    property alias contentWidth: row.width
    signal columnWidthChanged

    required property var model
    required property int columnCount
    required property real tableWidth
    property var color: "#32363b"

    Row {
        id: row
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
                    width: parent.width
                    elide: Text.ElideRight
                    color: Kirigami.Theme.textColor
                    horizontalAlignment: Qt.AlignLeft
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.model.headerData(parent.index, Qt.Horizontal, QPlaylistModel.ColumnName)
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
