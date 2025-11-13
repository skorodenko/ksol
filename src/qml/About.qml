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
            "displayName": "Ksol",
            "productName": "skorodenko/ksol",
            "shortDescription": "Lightweight, keyboard oriented Rust+Qtquick mpd client",
            "homepage": "",
            "bugAddress": "",
            "version": "1.0",
            "otherText": "",
            "authors": [
                {
                    "name": "skorodenko",
                    "task": "Main developer",
                    "emailAddress": "mskorodenko@gmail.com",
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
            "desktopFileName": "github.skorodenko.ksol"
        }
    }
}
