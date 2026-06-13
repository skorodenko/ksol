pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Dialogs
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
    property int selectedOption: QState.mpdBinaryAvailable() ? 0 : 1
    property string customServerUrl: ""

    function nextPage() {
        if (view.currentIndex == 1 && root.selectedOption == 0) {
            view.currentIndex = 3;
        } else if (view.currentIndex == 1 && root.selectedOption == 1) {
            root.customServerUrl = "";
            nextButton.enabled = false;
            view.currentIndex = 2;
        } else if (view.currentIndex == 2) {
            view.currentIndex = 4;
        } else {
            view.currentIndex = view.currentIndex + 1;
        }
    }

    function previousPage() {
        if (view.currentIndex == 2) {
            nextButton.enabled = true;
            view.currentIndex = 1;
        } else if (view.currentIndex == 3) {
            view.currentIndex = 1;
        } else {
            view.currentIndex = view.currentIndex - 1;
        }
    }

    onClosing: {
        if (root.currentPage != view.count - 1) {
            Qt.quit();
        }
    }

    Kirigami.InlineMessage {
        id: infoMessage

        visible: false
        anchors.bottom: footer.top
        anchors.left: parent.left
        anchors.right: parent.right
        onVisibleChanged: tmr.restart()

        Timer {
            id: tmr
            interval: Kirigami.Units.humanMoment
            onTriggered: infoMessage.visible = false
        }
    }

    QQC2.SwipeView {
        id: view

        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
            bottom: infoMessage.top
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
                    //enabled: QSettingsModel.mpdBinaryAvailable()
                    anchors.top: spHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.left: parent.left
                    anchors.right: parent.right
                    checked: root.selectedOption == 0
                    onClicked: root.selectedOption = 0
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
                    checked: root.selectedOption == 1
                    onClicked: root.selectedOption = 1
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
                    Binding {
                        target: root
                        property: "customServerUrl"
                        value: tpAdressField.text
                    }
                    placeholderText: "http://... or Unix socket"
                }

                QQC2.Button {
                    id: tpAddressCheck
                    anchors.top: tpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.right: parent.right
                    text: "Check connection"
                    onClicked: {
                        //let result = QSettingsModel.checkServerConnection(root.customServerUrl);
                        infoMessage.visible = false;
                        infoMessage.visible = true;
                        infoMessage.text = result ? "Successfully connected to server" : "Failed to connect to server";
                        infoMessage.type = result ? Kirigami.MessageType.Positive : Kirigami.MessageType.Error;
                        nextButton.enabled = result;
                    }
                }
            }
        }

        Item {
            id: fourthPage

            Item {
                anchors.fill: parent
                anchors.margins: 18

                Kirigami.Heading {
                    id: fpHeading
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    wrapMode: Text.WordWrap
                    text: "Select where your music is located"
                }

                QQC2.TextField {
                    id: fpMusicFolder
                    anchors.top: fpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.left: parent.left
                    anchors.right: fpChooseMusicFolder.left
                    anchors.rightMargin: Kirigami.Units.mediumSpacing
                    text: QAppSettings.nativeMusicDir
                    placeholderText: "Music folder ..."
                }

                FolderDialog {
                    id: folderDialog
                    title: "Please choose music folder"

                    onAccepted: {
                        fpMusicFolder.text = folderDialog.selectedFolder;
                        QAppSettings.nativeMusicDir = folderDialog.selectedFolder;
                    }
                }

                QQC2.Button {
                    id: fpChooseMusicFolder
                    anchors.top: fpHeading.bottom
                    anchors.topMargin: Kirigami.Units.largeSpacing
                    anchors.right: parent.right
                    icon.name: "folder"
                    onClicked: folderDialog.open()
                }
            }
        }

        Item {
            id: fifthPage

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
                QAppSettings.initWizard = false;
                if (root.selectedOption == 1) {
                    QAppSettings.mpdSocket = root.customServerUrl;
                }
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
