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
    property int firstVisibleColumn: 0
    property int lastVisibleColumn: 0
    property var color: "#32363b"

    Component.onCompleted: {
        root.firstVisibleColumn = root.updateFirstVisibleColumn();
        root.lastVisibleColumn = root.updateLastVisibleColumn();
    }

    QQC2.ContextMenu.menu: QQC2.Menu {
        id: playlistHeaderMenu

        Repeater {
            model: root.columnCount
            delegate: QQC2.CheckBox {
                required property int index
                text: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnName)
                checked: QSettingsModel.getColumnWidth(index) != 0
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
            item.Layout.horizontalStretchFactor = 100;
            QSettingsModel.setColumnWidth(column, 100);
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        } else {
            item.visible = false;
            QSettingsModel.setColumnWidth(column, 0);
            item.Layout.horizontalStretchFactor = 0;
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        }
    }

    function updateFirstVisibleColumn() {
        for (var i = 0; i < root.columnCount; i++) {
            var itemDelegate = repeater.itemAt(i);
            if (itemDelegate.width > 0) {
                return i;
            }
        }
        return 0;
    }

    function updateLastVisibleColumn() {
        for (var i = root.columnCount - 1; i > 0; i--) {
            var itemDelegate = repeater.itemAt(i);
            if (itemDelegate.width > 0) {
                return i;
            }
        }
        return 0;
    }

    function resetColumnWidth() {
        for (var i = 0; i < root.columnCount; i++) {
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
                Layout.horizontalStretchFactor: QSettingsModel.getColumnWidth(index)

                required property int index

                onWidthChanged: {
                    root.columnWidthChanged();
                }

                function applyWidthDelta(delta) {
                    var newWidth = delegate.Layout.horizontalStretchFactor + delta;
                    if (newWidth >= 1000) {
                        delegate.Layout.horizontalStretchFactor = 1000;
                        QSettingsModel.setColumnWidth(delegate.index, 1000);
                    } else if (newWidth <= 10) {
                        delegate.Layout.horizontalStretchFactor = 10;
                        QSettingsModel.setColumnWidth(delegate.index, 10);
                    } else {
                        delegate.Layout.horizontalStretchFactor = newWidth;
                        QSettingsModel.setColumnWidth(delegate.index, newWidth);
                    }
                }

                Item {
                    id: splitter
                    implicitWidth: 4
                    anchors.top: delegate.top
                    anchors.bottom: delegate.bottom
                    anchors.left: delegate.left
                    visible: delegate.index != root.firstVisibleColumn

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
                            if (delegate.index == root.lastVisibleColumn) {
                                delegate.applyWidthDelta(-widthDelta);
                                for (var i = delegate.index - 1; i >= 0; i--) {
                                    var itemDelegate = repeater.itemAt(i);
                                    if (itemDelegate.width > 0) {
                                        itemDelegate.applyWidthDelta(widthDelta);
                                        break;
                                    }
                                }
                            } else if (widthDelta > 0) {
                                for (var i = delegate.index - 1; i >= 0; i--) {
                                    var itemDelegate = repeater.itemAt(i);
                                    if (itemDelegate.width > 0) {
                                        itemDelegate.applyWidthDelta(widthDelta);
                                        break;
                                    }
                                }
                            } else {
                                for (var i = delegate.index - 1; i >= 0; i--) {
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

                    state: QSettingsModel.sortColumn == delegate.index ? QSettingsModel.sortOrder : "0"

                    MouseArea {
                        anchors.fill: sortIndicator
                        onClicked: {
                            QSettingsModel.toggleSortColumn(delegate.index);
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
                                sortIndicatorText.text: "v"
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
                                sortIndicatorText.text: "ʌ"
                            }
                        }
                    ]
                }
            }
        }
    }
}
