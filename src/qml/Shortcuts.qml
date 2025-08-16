import QtQuick

Item {
    readonly property alias drun_open: drun_open
    readonly property alias drun_close: drun_close
    readonly property alias drun_group1: drun_group1
    readonly property alias drun_group2: drun_group2
    readonly property alias drun_group3: drun_group3
    readonly property alias drun_group4: drun_group4

    Shortcut {
        id: drun_open
        sequences: ["Ctrl+f"]
        context: Qt.ApplicationShortcut
    }

    Shortcut {
        id: drun_close
        sequences: ["Escape"]
        context: Qt.ApplicationShortcut
    }

    Shortcut {
        id: drun_group1
        sequences: ["F1"]
        context: Qt.ApplicationShortcut
    }

    Shortcut {
        id: drun_group2
        sequences: ["F2"]
        context: Qt.ApplicationShortcut
    }

    Shortcut {
        id: drun_group3
        sequences: ["F3"]
        context: Qt.ApplicationShortcut
    }

    Shortcut {
        id: drun_group4
        sequences: ["F4"]
        context: Qt.ApplicationShortcut
    }
}
