pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import org.kde.kirigami as Kirigami
import ksol

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
                property var item: QState.getHeaderColumn(index)

                text: item?.name
                checked: !item?.hidden
                nextCheckState: function () {
                    item.hidden = !item.hidden;
                    root.toggleColumn(index, item);
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

    function toggleColumn(column, item) {
        if (item) {
            //let item = repeater.itemAt(column);
            //item.visible = state;
            QState.setHeaderColumn(column, item);
            root.firstVisibleColumn = root.updateFirstVisibleColumn();
            root.lastVisibleColumn = root.updateLastVisibleColumn();
        }
    }

    function updateFirstVisibleColumn() {
        for (let i = 0; i < root.columnCount; i++) {
            let item = QState.getHeaderColumn(i);
            if (item.hidden === false) {
                return i;
            }
        }
        return 0;
    }

    function updateLastVisibleColumn() {
        for (let i = root.columnCount - 1; i > 0; i--) {
            let item = QState.getHeaderColumn(i);
            if (item.hidden === false) {
                return i;
            }
        }
        return 0;
    }

    function visibleColumnCount() {
        let k = 0;
        for (let i = 0; i < root.columnCount; i++) {
            let item = QState.getHeaderColumn(i);
            if (item.hidden === false) {
                k++;
            }
        }
        return k;
    }

    function resetColumnWidth() {
        let visibleColumnCount = root.visibleColumnCount();
        for (let i = 0; i < root.columnCount; i++) {
            let item = repeater.itemAt(i);
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
                width: 6
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
                visible: !item?.hidden
                SplitView.fillWidth: index == root.lastVisibleColumn
                SplitView.minimumWidth: root.minimumColumnWidth
                SplitView.maximumWidth: root.maximumColumnWidth
                SplitView.preferredWidth: item?.width * root.width

                required property int index
                property var item: QState.getHeaderColumn(index)

                onWidthChanged: if (item) {
                    item.width = delegate.SplitView.preferredWidth / root.width;
                    QState.setHeaderColumn(delegate.index, item);
                    root.columnWidthChanged();
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true

                    onClicked: {
                        QState.setSort(delegate.index);
                    }

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
                    text: parent?.item?.name
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
