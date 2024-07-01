import QtQuick 2.15
import QtQuick.Layouts 1.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.20 as Kirigami
import controllers 1.0

Kirigami.ApplicationWindow {
    id: root
    title: qsTr("Ksol")
    pageStack.initialPage: [playlists, player]

    Component.onCompleted: mpd_connector.connect()
    Component.onDestruction: mpd_connector.disconnect()

    MPDConnector {
        id: mpd_connector
    }

    Kirigami.Page {
        id: player
        globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

        header: QQC2.ToolBar {
            implicitHeight: 48

            RowLayout {
                spacing: 8
                anchors.rightMargin: 8
                anchors.fill: parent

                RowLayout {
                    id: media_playback_controls
                    spacing: 0

                    QQC2.Button {
                        id: playback_backward
                        icon.name: "media-skip-backward"
                        flat: true
                    }
                    QQC2.Button {
                        id: playback_play
                        icon.name: "media-playback-start"
                        flat: true
                    }
                    QQC2.Button {
                        id: playback_forward
                        icon.name: "media-skip-forward"
                        flat: true
                    }
                }

                ColumnLayout {
                    spacing: 0

                    RowLayout {
                        QQC2.Label {
                            id: media_title
                            text: "Title | Author"
                            Layout.fillWidth: true
                        }
                        QQC2.Label {
                            id: media_duration
                            text: "0:00 / 0:00"
                            Layout.rightMargin: Qt.Infinity
                        }
                    }

                    RowLayout {
                        QQC2.Slider {
                            id: media_seeker
                            Layout.fillWidth: true
                        }
                    }
                }
            }
        }
    }

    Kirigami.ScrollablePage {
        id: playlists
        globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

        header: QQC2.ToolBar {
            implicitHeight: 48
            RowLayout {
                anchors.fill: parent

                Kirigami.Heading {
                    text: "Group by:"
                }

                QQC2.ComboBox {
                    id: group_combo
                    textRole: "name"
                    valueRole: "value"
                    //onActivated: groups_model.setActive(currentValue)
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignHCenter
                    //model: groups_model
                }
            }
        }

        //        UIM.PlaylistsGroup {
        //            id: groups_model
        //            onUpdated: function (index) {
        //                playlists_model.refresh(index);
        //            }
        //        }
        //
        //        UIM.Playlists {
        //            id: playlists_model
        //        }

        //            Connections {
        //                target: UIC.Main
        //                function onConnected() {
        //                    groups_model.refresh()
        //                }
        //            }

        ListView {
            id: playlists_view
            anchors.fill: parent
            //            model: playlists_model
            delegate: Item {
                height: 30

                //                width: playlists_view.width

                required property string name

                QQC2.Label {
                    text: name
                    font.pixelSize: 14
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: 15
                    anchors.rightMargin: 15
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideRight
                }
            }
        }
    }
}
