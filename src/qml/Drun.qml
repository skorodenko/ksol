pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
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

    signal stagePlaylist(string name, int group)

    property alias playlists_list: playlists_list
    property alias playlists_group: playlists_group

    QPlaylistsGroupModel {
        id: playlists_group
    }

    QPlaylistsListModel {
        id: playlists_list
        filter: search.text

        onLayoutChanged: function () {
            if (playlists_list.rowCount() > 0) {
                listView.currentIndex = 0;
                listView.forceActiveFocus();
            }
        }
    }

    Shortcut {
        id: drun_open
        sequences: ["f"]
        context: Qt.ApplicationShortcut
        enabled: !root.visible
        onActivated: function () {
            root.visible = true;
        }
    }

    Shortcut {
        id: drun_close
        sequences: ["Escape"]
        context: Qt.ApplicationShortcut
        enabled: root.visible
        onActivated: function () {
            root.visible = false;
        }
    }

    Shortcut {
        id: drun_group1
        sequences: ["F1"]
        context: Qt.ApplicationShortcut
        enabled: root.visible
        onActivated: function () {
            group_repeater.itemAt(0).click();
        }
    }

    Shortcut {
        id: drun_group2
        sequences: ["F2"]
        context: Qt.ApplicationShortcut
        enabled: root.visible
        onActivated: function () {
            group_repeater.itemAt(1).click();
        }
    }

    Shortcut {
        id: drun_group3
        sequences: ["F3"]
        context: Qt.ApplicationShortcut
        enabled: root.visible
        onActivated: function () {
            group_repeater.itemAt(2).click();
        }
    }

    Shortcut {
        id: drun_group4
        sequences: ["F4"]
        context: Qt.ApplicationShortcut
        enabled: root.visible
        onActivated: function () {
            group_repeater.itemAt(3).click();
        }
    }

    RowLayout {
        id: group_change
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right

        QQC2.TextField {
            id: search
            focusPolicy: Qt.NoFocus
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

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.alignment: Qt.AlignRight

            spacing: Kirigami.Units.largeSpacing

            Repeater {
                id: group_repeater
                model: playlists_group

                QQC2.Button {
                    required property string name
                    required property int value

                    Kirigami.Heading {
                        anchors.centerIn: parent
                        text: parent.name
                    }

                    focusPolicy: Qt.NoFocus
                    Layout.fillWidth: true
                    Layout.fillHeight: true

                    onClicked: {
                        playlists_group.activeGroup = value;
                    }

                    background: Rectangle {
                        color: parent.value === playlists_group.activeGroup ? Kirigami.Theme.activeBackgroundColor : Kirigami.Theme.alternateBackgroundColor
                        radius: Kirigami.Units.cornerRadius
                    }
                }
            }
        }
    }

    ListView {
        id: listView
        anchors.topMargin: Kirigami.Units.largeSpacing * 3
        anchors.top: group_change.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        boundsBehavior: Flickable.StopAtBounds
        boundsMovement: Flickable.StopAtBounds

        model: playlists_list

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
                root.visible = false;
                root.stagePlaylist(listView.currentItem.name, playlists_group.activeGroup);
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
                        root.visible = false;
                        root.stagePlaylist(delegateItem.name, playlists_group.activeGroup);
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
        opacity: 0.5
    }
}
