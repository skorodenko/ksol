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
    property var color: "#32363b"

    QQC2.ContextMenu.menu: QQC2.Menu {
        id: playlistHeaderMenu

        Repeater {
            model: root.columnCount
            delegate: QQC2.CheckBox {
                required property int index
                text: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnName)
                checked: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnWidth) != 0
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
            root.model.updateColumnWidth(column, 100);
        } else {
            item.visible = false;
            root.model.updateColumnWidth(column, 0);
        }
    }

    function resetColumnWidth() {
        for (var i = 0; i < root.model.rowCount(); i++) {
            var item = repeater.itemAt(i);
            if (item.visible) {
                item.Layout.horizontalStretchFactor = 100;
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

                enabled: visible
                visible: Layout.horizontalStretchFactor == 0 ? false : true
                Layout.fillWidth: true
                Layout.preferredWidth: 35
                Layout.preferredHeight: root.height
                Layout.horizontalStretchFactor: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnWidth)

                required property int index

                onWidthChanged: {
                    Qt.callLater(root.columnWidthChanged);
                }

                function applyWidthDelta(delta) {
                    var newWidth = delegate.Layout.horizontalStretchFactor + delta;
                    if (newWidth >= 1000) {
                        delegate.Layout.horizontalStretchFactor = 1000;
                        root.model.updateColumnWidth(delegate.index, 1000);
                    } else if (newWidth <= 10) {
                        delegate.Layout.horizontalStretchFactor = 10;
                        root.model.updateColumnWidth(delegate.index, 10);
                    } else {
                        delegate.Layout.horizontalStretchFactor = newWidth;
                        root.model.updateColumnWidth(delegate.index, newWidth);
                    }
                }

                Item {
                    id: splitter
                    implicitWidth: 4
                    anchors.top: delegate.top
                    anchors.bottom: delegate.bottom
                    anchors.left: delegate.left
                    visible: delegate.index != root.model.firstVisibleColumn

                    Rectangle {
                        id: splitterRect
                        implicitWidth: 2
                        color: "#595d61"

                        anchors {
                            top: splitter.top
                            bottom: splitter.bottom
                            left: splitter.left
                        }
                    }
                }

                MouseArea {
                    anchors.fill: splitter
                    enabled: delegate.visible

                    property int oldMouseX

                    onPressed: {
                        oldMouseX = mouseX;
                    }
                    onPositionChanged: {
                        if (pressed) {
                            var widthDelta = 50 * (mouseX - oldMouseX) / root.width;
                            if (delegate.index == root.model.lastVisibleColumn) {
                                delegate.applyWidthDelta(-widthDelta);
                                for (var i = delegate.index - 1; i >= root.model.firstVisibleColumn; i--) {
                                    var itemDelegate = repeater.itemAt(i);
                                    if (itemDelegate.width > 0) {
                                        itemDelegate.applyWidthDelta(widthDelta);
                                        break;
                                    }
                                }
                            } else if (widthDelta > 0) {
                                for (var i = delegate.index - 1; i >= root.model.firstVisibleColumn; i--) {
                                    var itemDelegate = repeater.itemAt(i);
                                    if (itemDelegate.width > 0) {
                                        itemDelegate.applyWidthDelta(widthDelta);
                                        break;
                                    }
                                }
                            } else {
                                for (var i = delegate.index - 1; i >= root.model.firstVisibleColumn; i--) {
                                    var itemDelegate = repeater.itemAt(i);
                                    if (itemDelegate.width > 0) {
                                        itemDelegate.applyWidthDelta(widthDelta);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                Text {
                    elide: Text.ElideRight
                    color: Kirigami.Theme.textColor
                    horizontalAlignment: Qt.AlignLeft
                    anchors.left: splitter.right
                    anchors.right: sortIndicator.left
                    text: root.model.headerData(delegate.index, Qt.Horizontal, QPlaylistModel.ColumnName)
                }

                Item {
                    id: sortIndicator
                    implicitWidth: 8
                    anchors.top: delegate.top
                    anchors.bottom: delegate.bottom
                    anchors.right: delegate.right

                    state: root.model.sortColumn == delegate.index ? root.model.sortOrder : "0"

                    MouseArea {
                        anchors.fill: sortIndicator
                        onClicked: {
                            root.model.sortPlaylist(delegate.index);
                        }
                    }

                    Text {
                        id: sortIndicatorText
                        color: Kirigami.Theme.textColor
                        anchors.centerIn: parent
                    }

                    states: [
                        State {
                            name: "1"
                            PropertyChanges {
                                sortIndicatorText.text: "V"
                            }
                        },
                        State {
                            name: "0"
                            PropertyChanges {
                                sortIndicatorText.text: "-"
                            }
                        },
                        State {
                            name: "-1"
                            PropertyChanges {
                                sortIndicatorText.text: "Ʌ"
                            }
                        }
                    ]
                }
            }
        }
    }
}
