pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Item {
    id: root

    property alias repeater: repeater
    signal columnWidthChanged

    required property var model
    required property int columnCount
    required property real tableWidth
    property var color: "#32363b"

    Rectangle {
        anchors.fill: parent
        color: parent.color
    }

    function visibleColumnCount() {
        var k = 0;
        for(var i = 0; i < repeater.count; i++) {
            var item = repeater.itemAt(i);
            if (item.width != 0.0) {
                k++;
            }
        }
        return k
    }

    function resetColumnWidth() {
        var visibleCols = root.visibleColumnCount();
        for(var i = 0; i < repeater.count; i++) {
            var item = repeater.itemAt(i);
            if (item.width != 0.0) {
                item.Layout.preferredWidth = root.tableWidth / visibleCols;
            }
        }
    }

    RowLayout {
        id: row
        spacing: 0
        anchors.fill: parent

        Repeater {
            id: repeater

            model: root.columnCount

            delegate: Rectangle {
                id: delegate
                color: root.color

                Layout.fillWidth: true
                Layout.preferredWidth: root.width * root.model.headerData(delegate.index, Qt.Horizontal, QPlaylistModel.ColumnWidth)
                Layout.preferredHeight: root.height

                required property int index

                onWidthChanged: {
                    root.columnWidthChanged();
                }

                Text {
                    elide: Text.ElideRight
                    color: Kirigami.Theme.textColor
                    horizontalAlignment: Qt.AlignLeft
                    anchors.left: delegate.left
                    anchors.right: splitter.left
                    text: root.model.headerData(delegate.index, Qt.Horizontal, QPlaylistModel.ColumnName)
                }

                MouseArea {
                    anchors.fill: splitter

                    property int oldMouseX

                    onPressed: {
                        oldMouseX = mouseX;
                    }
                    onPositionChanged: {
                        if (pressed) {
                            var widthDelta = (mouseX - oldMouseX);
                            var newWidth = delegate.Layout.preferredWidth + widthDelta;
                            if (newWidth >= 36) {
                                delegate.Layout.preferredWidth = newWidth;
                            } else {
                                delegate.Layout.preferredWidth = 36;
                            }
                        }
                    }
                }

                Item {
                    id: splitter
                    implicitWidth: 6
                    anchors.top: delegate.top
                    anchors.bottom: delegate.bottom
                    anchors.right: delegate.right
                    visible: delegate.index != root.model.lastVisibleColumn

                    Rectangle {
                        id: splitterRect
                        width: 2
                        color: "#595d61"

                        anchors {
                            top: splitter.top
                            bottom: splitter.bottom
                            horizontalCenter: splitter.horizontalCenter
                        }
                    }
                }
            }
        }
    }
}
