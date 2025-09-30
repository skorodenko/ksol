pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Item {
    id: root
    implicitHeight: 18
    x: -root.tableOffset
    z: 1

    signal columnWidthUpdate(int column, real width)

    required property var model
    required property int columnCount
    required property real tableOffset
    required property real tableWidth

    Row {
        Repeater {
            model: root.columnCount

            delegate: Rectangle {
                width: root.tableWidth * root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnWidth) 
                height: root.height
                color: Kirigami.Theme.disabledTextColor

                required property int index

                Component.onCompleted: {
                    root.columnWidthUpdate(index, width);
                }

                Text {
                    text: root.model.headerData(parent.index, Qt.Horizontal, QPlaylistModel.ColumnName) 
                    anchors.centerIn: parent
                }
            }
        }
    }
}
