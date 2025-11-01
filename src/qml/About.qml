pragma ComponentBehavior: Bound

import QtQuick
import org.kde.kirigami as Kirigami

Kirigami.Dialog {
    id: root

    title: "About"
    preferredWidth: Kirigami.Units.gridUnit * 32

    standardButtons: Kirigami.Dialog.Cancel

    flatFooterButtons: true
    showCloseButton: false

    onAccepted: root.close()

    Kirigami.AboutPage {
        aboutData: {
            "displayName": "KirigamiApp",
            "productName": "kirigami/app",
            "componentName": "kirigamiapp",
            "shortDescription": "A Kirigami example",
            "homepage": "",
            "bugAddress": "submit@bugs.kde.org",
            "version": "5.14.80",
            "otherText": "",
            "authors": [
                {
                    "name": "...",
                    "task": "",
                    "emailAddress": "somebody@kde.org",
                    "webAddress": "",
                    "ocsUsername": ""
                }
            ],
            "credits": [],
            "translators": [],
            "licenses": [
                {
                    "name": "GPL v2",
                    "text": "long, boring, license text",
                    "spdx": "GPL-2.0"
                }
            ],
            "copyrightStatement": "© 2010-2018 Plasma Development Team",
            "desktopFileName": "org.kde.kirigamiapp"
        }
    }
}
