pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
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
    property int minimumColumnWidth: 60
    property int mediumColumnWidth: 75
    property int maximumColumnWidth: width
    property var color: "#32363b"

    Component.onCompleted: {
        root.firstVisibleColumn = root.updateFirstVisibleColumn();
        root.lastVisibleColumn = root.updateLastVisibleColumn();
    }

    ContextMenu.menu: Menu {
        id: playlistHeaderMenu

        Repeater {
            model: root.columnCount
            delegate: CheckBox {
                required property int index
                text: root.model.headerData(index, Qt.Horizontal, QPlaylistModel.ColumnName)
                checked: QSettingsModel.getColumnWidth(index) != 0
                nextCheckState: function () {
                    root.toggleColumn(index, !checked);
                    return checked ? Qt.Unchecked : Qt.Checked;
                }
            }
        }

        MenuItem {
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
            QSettingsModel.setColumnWidth(column, root.mediumColumnWidth);
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        } else {
            item.visible = false;
            QSettingsModel.setColumnWidth(column, 0);
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        }
    }

    function updateFirstVisibleColumn() {
        for (var i = 0; i < root.columnCount; i++) {
            var itemDelegate = repeater.itemAt(i);
            if (itemDelegate.visible) {
                return i;
            }
        }
        return 0;
    }

    function updateLastVisibleColumn() {
        for (var i = root.columnCount - 1; i > 0; i--) {
            var itemDelegate = repeater.itemAt(i);
            if (itemDelegate.visible) {
                return i;
            }
        }
        return 0;
    }

    function visibleColumnCount() {
        var k = 0;
        for (var i = 0; i < root.columnCount; i++) {
            var itemDelegate = repeater.itemAt(i);
            if (itemDelegate.visible) {
                k++;
            }
        }
        return k;
    }

    function resetColumnWidth() {
        var visibleColumnCount = root.visibleColumnCount();
        for (var i = 0; i < root.columnCount; i++) {
            var item = repeater.itemAt(i);
            if (item.visible) {
                item.SplitView.preferredWidth = root.width / visibleColumnCount;
            }
        }
    }

    SplitView {
        id: view
        spacing: 0
        anchors.fill: parent
        orientation: Qt.Horizontal

        handle: Rectangle {
            id: handleRect
            implicitWidth: 2
            color: "#595d61"

            containmentMask: Item {
                x: (handleRect.width - width) / 2
                width: 4
                height: view.height
            }
        }

        Repeater {
            id: repeater

            model: root.columnCount

            delegate: Rectangle {
                id: delegate
                color: root.color

                enabled: visible
                visible: SplitView.preferredWidth == 0 ? false : true
                SplitView.fillWidth: index == root.lastVisibleColumn
                SplitView.minimumWidth: root.minimumColumnWidth
                SplitView.maximumWidth: root.maximumColumnWidth
                SplitView.preferredWidth: QSettingsModel.getColumnWidth(index) * root.width

                required property int index

                onWidthChanged: {
                    root.columnWidthChanged();
                    QSettingsModel.setColumnWidth(delegate.index, delegate.SplitView.preferredWidth / root.width);
                }

                Text {
                    elide: Text.ElideRight
                    color: Kirigami.Theme.textColor
                    horizontalAlignment: Qt.AlignLeft
                    anchors.left: delegate.left
                    anchors.right: sortIndicator.left
                    anchors.leftMargin: Kirigami.Units.smallSpacing
                    text: root.model.headerData(delegate.index, Qt.Horizontal, QPlaylistModel.ColumnName)
                }

                Item {
                    id: sortIndicator
                    implicitWidth: 8
                    anchors.top: delegate.top
                    anchors.bottom: delegate.bottom
                    anchors.right: delegate.right
                    anchors.rightMargin: Kirigami.Units.smallSpacing

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
