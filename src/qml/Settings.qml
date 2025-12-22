pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.Page {
    id: root
    padding: 0
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    signal backRequest(restartMpd: bool)

    property string selectedPage: "General"
    property bool restartMpd: false

    Connections {
        target: QSettingsModel

        function onMpdSettingsUpdate() {
            root.restartMpd = true;
        }
    }

    onVisibleChanged: {
        root.restartMpd = false;
        mpdPage.reset();
    }

    onSelectedPageChanged: {
        switch (root.selectedPage) {
        case "General":
            view.pop();
            view.push(mpdPage);
            break;
        case "Appearence":
            view.pop();
            view.push(appearencePage);
            break;
        }
    }

    Kirigami.PageRow {
        id: view

        initialPage: [selectionMenu, mpdPage]

        anchors.fill: parent
        defaultColumnWidth: 10 * Kirigami.Units.gridUnit

        Kirigami.ScrollablePage {
            id: selectionMenu
            padding: 0

            ColumnLayout {
                SidebarDelegate {
                    text: "Return"
                    iconName: "go-previous-symbolic"
                    Layout.fillWidth: true
                    onClicked: root.backRequest(root.restartMpd)
                }
                SidebarDelegate {
                    text: "General"
                    iconName: "preferences-desktop-multimedia"
                    Layout.fillWidth: true
                    highlighted: root.selectedPage == text
                    onClicked: root.selectedPage = text
                }
                SidebarDelegate {
                    text: "Appearence"
                    iconName: "preferences-desktop-theme"
                    Layout.fillWidth: true
                    highlighted: root.selectedPage == text
                    onClicked: root.selectedPage = text
                }
            }
        }

        SettingsMPDPage {
            id: mpdPage
        }

        SettingsAppearancePage {
            id: appearencePage
        }
    }
}
