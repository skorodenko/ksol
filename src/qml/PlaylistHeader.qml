pragma ComponentBehavior: Bound

import QtQuick
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
    property int maximumColumnWidth: width
    property var color: "#32363b"
    property var highlightColor: Kirigami.Theme.highlightColor

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
                checked: QState.getHeaderColumn(index, ColumnType.Width) != 0.0
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
            QState.setHeaderColumn(column, ColumnType.Width, root.minimumColumnWidth / root.width);
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        } else {
            item.visible = false;
            QState.setHeaderColumn(column, ColumnType.Width, 0.0);
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        }
    }

    function updateFirstVisibleColumn() {
        for (var i = 0; i < root.columnCount; i++) {
            if (QState.getHeaderColumn(i, ColumnType.Width) != 0.0) {
                return i;
            }
        }
        return 0;
    }

    function updateLastVisibleColumn() {
        for (var i = root.columnCount - 1; i > 0; i--) {
            if (QState.getHeaderColumn(i, ColumnType.Width) != 0.0) {
                return i;
            }
        }
        return 0;
    }

    function visibleColumnCount() {
        var k = 0;
        for (var i = 0; i < root.columnCount; i++) {
            if (QState.getHeaderColumn(i, ColumnType.Width) != 0.0) {
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
            implicitWidth: 1
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
                SplitView.preferredWidth: QState.getHeaderColumn(index, ColumnType.Width) * root.width

                required property int index

                onWidthChanged: {
                    root.columnWidthChanged();
                    QState.setHeaderColumn(delegate.index, ColumnType.Width, delegate.SplitView.preferredWidth / root.width);
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true

                    //onClicked: {
                    //    QSettingsModel.toggleSortColumn(delegate.index);
                    //}

                    onContainsMouseChanged: {
                        if (containsMouse) {
                            highlight.visible = true;
                        } else {
                            highlight.visible = false;
                        }
                    }
                }

                Rectangle {
                    id: highlight
                    opacity: 0.5
                    visible: false
                    anchors.fill: parent
                    color: root.highlightColor
                }

                Text {
                    elide: Text.ElideRight
                    color: Kirigami.Theme.textColor
                    horizontalAlignment: Qt.AlignLeft
                    anchors.left: delegate.left
                    anchors.right: sortIndicator.left
                    anchors.verticalCenter: delegate.verticalCenter
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

                    state: QState.sortColumn == delegate.index ? QState.sortOrder : "0"

                    Kirigami.Icon {
                        id: sortIndicatorText
                        width: Kirigami.Units.iconSizes.small
                        height: Kirigami.Units.iconSizes.small
                        anchors.centerIn: parent
                    }

                    states: [
                        State {
                            name: "1"
                            PropertyChanges {
                                sortIndicatorText.source: "arrow-down"
                            }
                        },
                        State {
                            name: "0"
                            PropertyChanges {
                                sortIndicatorText.source: ""
                            }
                        },
                        State {
                            name: "-1"
                            PropertyChanges {
                                sortIndicatorText.source: "arrow-up"
                            }
                        }
                    ]
                }

                Rectangle {
                    implicitHeight: 1
                    color: "#595d61"
                    anchors.left: delegate.left
                    anchors.right: delegate.right
                    anchors.bottom: delegate.bottom
                }
            }
        }
    }
}
