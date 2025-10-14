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

    QQC2.ContextMenu.menu: QQC2.Menu {
        id: playlistHeaderMenu

        Repeater {
            model: root.columnCount
            delegate: QQC2.CheckBox {
                required property int index
                text: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnName)
                checked: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnWidth) != 0.0
                nextCheckState: function () {
                    root.toggleColumn(index, !checked);
                    return checked ? Qt.Unchecked : Qt.Checked;
                }
            }
        }

        QQC2.MenuItem {
            text: qsTr("Reset width")
            onTriggered: {
                root.resetColumnWidth();
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
    }

    function toggleColumn(column, state) {
        var item = repeater.itemAt(column);
        if (state == true) {
            item.visible = true;
            item.enabled = true;
            item.Layout.preferredWidth = 45;
            root.model.updateColumnWidth(column, item.width / root.tableWidth);
        } else {
            item.visible = false;
            item.enabled = false;
            root.model.updateColumnWidth(column, 0.0);
        }
    }

    function visibleColumnCount() {
        var k = 0;
        for (var i = 0; i < repeater.count; i++) {
            var item = repeater.itemAt(i);
            if (item.width != 0.0) {
                k++;
            }
        }
        return k;
    }

    function resetColumnWidth() {
        var visibleCols = root.visibleColumnCount();
        for (var i = 0; i < repeater.count; i++) {
            var item = repeater.itemAt(i);
            if (item.width != 0.0) {
                item.Layout.preferredWidth = root.tableWidth / visibleCols;
                root.model.updateColumnWidth(i, 1 / visibleCols);
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
                    enabled: delegate.width > 0.0 
                    
                    property int oldMouseX

                    onPressed: {
                        oldMouseX = mouseX;
                    }
                    onPositionChanged: {
                        if (pressed && width != 0.0) {
                            var widthDelta = (mouseX - oldMouseX);
                            var newWidth = delegate.Layout.preferredWidth + widthDelta;
                            if (newWidth >= 45) {
                                delegate.Layout.preferredWidth = newWidth;
                            } else {
                                delegate.Layout.preferredWidth = 45;
                            }
                            root.model.updateColumnWidth(delegate.index, delegate.width / root.tableWidth);
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
