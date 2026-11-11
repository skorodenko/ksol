pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls as QQC2

QQC2.Control {
    id: root

    signal openDrun
    signal openFilter
    signal closeMenu
    signal drunGroupChange(int group)
    signal moveToSong(int mode)

    required property bool drunVisible
    required property bool filterSearchVisible

    Shortcut {
        sequences: ["Escape"]
        onActivated: root.closeMenu()
    }

    Shortcut {
        id: drun_open
        sequences: ["f"]
        enabled: !root.filterSearchVisible && !root.drunVisible
        onActivated: root.openDrun()
    }

    Shortcut {
        sequences: ["g"]
        enabled: !root.filterSearchVisible && !root.drunVisible
        onActivated: root.moveToSong(1)
    }

    Shortcut {
        sequences: ["Shift+g"]
        enabled: !root.filterSearchVisible && !root.drunVisible
        onActivated: root.moveToSong(-1)
    }

    Shortcut {
        sequences: ["c"]
        enabled: !root.filterSearchVisible && !root.drunVisible
        onActivated: root.moveToSong(0)
    }

    Shortcut {
        sequences: ["/"]
        context: Qt.ApplicationShortcut
        enabled: !root.filterSearchVisible && !root.drunVisible
        onActivated: root.openFilter()
    }

    Shortcut {
        id: drun_group1
        sequences: ["F1"]
        enabled: root.drunVisible
        context: Qt.ApplicationShortcut
        onActivated: root.drunGroupChange(0)
    }

    Shortcut {
        id: drun_group2
        sequences: ["F2"]
        enabled: root.drunVisible
        context: Qt.ApplicationShortcut
        onActivated: root.drunGroupChange(1)
    }

    Shortcut {
        id: drun_group3
        sequences: ["F3"]
        enabled: root.drunVisible
        context: Qt.ApplicationShortcut
        onActivated: root.drunGroupChange(2)
    }

    Shortcut {
        id: drun_group4
        sequences: ["F4"]
        enabled: root.drunVisible
        context: Qt.ApplicationShortcut
        onActivated: root.drunGroupChange(3)
    }
}
