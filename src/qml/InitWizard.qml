pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Window {
    id: root
    title: "Init Wizard"
    visible: false

    flags: Qt.Dialog
    modality: Qt.WindowModal

    signal finished

    property alias currentPage: view.currentIndex
    property bool nativeServer: QSettingsModel.mpdBinaryAvailable()

    function nextPage() {
        if (view.currentIndex == 1 && root.nativeServer) {
            view.currentIndex = 3;
        } else if (view.currentIndex == 1 && !root.nativeServer) {
            nextButton.enabled = false;
            view.currentIndex = view.currentIndex + 1;
        } else {
            view.currentIndex = view.currentIndex + 1;
        }
    }

    function previousPage() {
        if (view.currentIndex == 100500) {} else {
            view.currentIndex = view.currentIndex - 1;
        }
    }

    Connections {
        target: QSettingsModel

        function onCheckServerConnectionResult(result) {
            nextButton.enabled = result;
        }
    }

    QQC2.SwipeView {
        id: view

        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
            bottom: footer.top
        }

        Item {
            id: firstPage

            Kirigami.Heading {
                text: "Welcome to ksol"
                color: Kirigami.Theme.textColor
                anchors.centerIn: parent
            }
        }

        Item {
            id: secondPage

            Item {
                anchors.fill: parent
                anchors.margins: 18

                Kirigami.Heading {
                    id: spHeading
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    text: "Choose configuration preset for ksol:"
                }
                QQC2.RadioButton {
                    id: spRadioButton1
                    enabled: QSettingsModel.mpdBinaryAvailable()
                    anchors.top: spHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.left: parent.left
                    anchors.right: parent.right
                    checked: root.nativeServer
                    text: "Native server: mpd managed by ksol (mpd binary should be visible in PATH)"
                    contentItem: QQC2.Label {
                        text: parent.text
                        font: parent.font
                        horizontalAlignment: Text.AlignLeft
                        verticalAlignment: Text.AlignVCenter
                        leftPadding: parent.indicator.width + parent.spacing
                        wrapMode: QQC2.Label.Wrap
                    }
                }
                QQC2.RadioButton {
                    id: spRadioButton2
                    anchors.top: spRadioButton1.bottom
                    anchors.left: parent.left
                    anchors.right: parent.right
                    checked: !root.nativeServer
                    text: "External server: mpd managed externally"
                    contentItem: QQC2.Label {
                        text: parent.text
                        font: parent.font
                        horizontalAlignment: Text.AlignLeft
                        verticalAlignment: Text.AlignVCenter
                        leftPadding: parent.indicator.width + parent.spacing
                        wrapMode: QQC2.Label.Wrap
                    }
                }
            }
        }

        Item {
            id: thirdPage

            Item {
                anchors.fill: parent
                anchors.margins: 18

                Kirigami.Heading {
                    id: tpHeading
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    wrapMode: Text.WordWrap
                    text: "Enter uri to connect to mpd server"
                }

                QQC2.TextField {
                    id: tpAdressField
                    anchors.top: tpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.left: parent.left
                    anchors.right: tpAddressCheck.left
                    anchors.rightMargin: Kirigami.Units.mediumSpacing
                    text: QSettingsModel.mpdSocket
                    placeholderText: "http://... or Unix socket"
                }

                QQC2.Button {
                    id: tpAddressCheck
                    anchors.top: tpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.right: parent.right
                    text: "Check connection"
                    onClicked: QSettingsModel.checkServerConnection()
                }
            }
        }

        Item {
            id: fourthPage

            Kirigami.Heading {
                text: "Initial config successfull"
                color: Kirigami.Theme.textColor
                anchors.centerIn: parent
            }
        }
    }

    QQC2.Control {
        id: footer
        implicitHeight: 48
        padding: 12

        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
        }

        QQC2.Button {
            id: cancelButton
            text: "Cancel"
            visible: view.currentIndex < view.count - 1

            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 6

            onClicked: Qt.quit()
        }

        QQC2.Button {
            id: previousButton
            text: "Previous"
            visible: view.currentIndex < view.count - 1 && view.currentIndex != 0

            anchors.right: nextButton.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 6

            onClicked: root.previousPage()
        }

        QQC2.Button {
            id: nextButton
            text: "Next"
            visible: view.count > view.currentIndex

            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 6

            onClicked: root.nextPage()
        }

        QQC2.Button {
            id: finishButton
            text: "Finish"
            visible: view.count - 1 == view.currentIndex

            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 6

            onClicked: {
                root.visible = false;
                root.finished();
            }
        }
    }

    Rectangle {
        z: -1
        anchors.fill: parent
        color: Kirigami.Theme.backgroundColor
    }
}
