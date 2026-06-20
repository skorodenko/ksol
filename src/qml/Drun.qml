pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Effects
import QtQuick.Layouts
import QtQml.Models
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

QQC2.Popup {
    id: root
    modal: true

    onVisibleChanged: {
        search.text = "";
        listView.currentIndex = 0;
        listView.forceActiveFocus();
    }

    signal activeGroupChanged(string group)
    signal stagePlaylist(string name, string group)

    property alias playlists_list: playlists_list

    function changeGroup(number) {
        var value = groupSelect.model[number];
        QState.activeGroup = value;
        root.activeGroupChanged(value);
    }

    QPlaylistsListModel {
        id: playlists_list

        onLayoutChanged: function () {
            if (playlists_list.rowCount() > 0) {
                listView.currentIndex = 0;
                listView.forceActiveFocus();
            }
        }
    }

    RowLayout {
        id: group_change
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right

        Kirigami.SearchField {
            id: search
            focusPolicy: Qt.NoFocus
            placeholderText: "Filter by group ..."
            onTextChanged: proxyModel.invalidate()
            Layout.alignment: Qt.AlignLeft
            Layout.preferredWidth: parent.width * 0.3
            Keys.onPressed: function (event) {
                if (!(event.key > Qt.Key_Space || event.key < Qt.Key_AsciiTilde || event.key === Qt.Key_Backspace)) {
                    event.accepted = true;
                }
            }
        }

        Item {
            Layout.preferredWidth: parent.width * 0.1
        }

        QQC2.ComboBox {
            id: groupSelect
            model: ["Directory", "Artist", "Album", "Date", "Genre", "Composer", "Albumartist"]
            Layout.alignment: Qt.AlignRight
            Layout.preferredWidth: parent.width * 0.2
            currentValue: QState.activeGroup
            onActivated: {
                QState.activeGroup = currentValue;
                root.activeGroupChanged(currentValue);
            }
        }
    }

    SortFilterProxyModel {
        id: proxyModel
        model: playlists_list
        filters: [
            FunctionFilter {
                component RoleData: QtObject {
                    property string name
                }

                function filter(data: RoleData) : bool {
                    var searchTxt = search.text.toLowerCase();
                    return data.name.toLowerCase().includes(searchTxt)
                }
            }
        ]
    }

    ListView {
        id: listView
        clip: true
        anchors.topMargin: Kirigami.Units.largeSpacing * 3
        anchors.top: group_change.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        boundsBehavior: Flickable.StopAtBounds
        boundsMovement: Flickable.StopAtBounds

        //model: playlists_list
        model: proxyModel

        implicitWidth: Math.min(Kirigami.Units.gridUnit * 20, parent.width)

        Keys.forwardTo: [search]

        QQC2.ScrollBar.vertical: QQC2.ScrollBar {
            id: scrollbar
            policy: QQC2.ScrollBar.AlwaysOn
        }

        delegate: Item {
            id: delegateItem
            height: 30
            width: ListView.view.width - scrollbar.width

            required property string name
            required property int index

            Keys.onReturnPressed: function () {
                root.stagePlaylist(listView.currentItem.name, QState.activeGroup);
                root.visible = false;
            }

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton

                onClicked: function (mouse) {
                    if (mouse.button == Qt.LeftButton) {
                        listView.currentIndex = delegateItem.index;
                    }
                }

                onDoubleClicked: function (mouse) {
                    if (mouse.button == Qt.LeftButton) {
                        root.stagePlaylist(delegateItem.name, QState.activeGroup);
                        root.visible = false;
                    }
                }
            }

            QQC2.Label {
                text: parent.name
                font.pixelSize: Kirigami.Theme.defaultFont.pixelSize
                elide: Text.ElideRight
                anchors.rightMargin: Kirigami.Units.largeSpacing
                anchors.leftMargin: Kirigami.Units.largeSpacing
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.right: parent.right
            }
        }

        highlight: Rectangle {
            height: 30
            width: ListView.view.width - scrollbar.width
            color: Kirigami.Theme.neutralBackgroundColor
            radius: Kirigami.Units.cornerRadius
        }
    }

    background: Rectangle {
        anchors.fill: parent
        radius: Kirigami.Units.cornerRadius
        color: Kirigami.Theme.backgroundColor
    }

    QQC2.Overlay.modal: Rectangle {
        id: overlay
        color: Kirigami.Theme.alternateBackgroundColor
        opacity: 0.75
    }
}
