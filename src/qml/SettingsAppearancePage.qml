pragma ComponentBehavior: Bound

import QtQuick
import Qt.labs.synchronizer
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.Page {
    id: root

    Synchronizer {
        sourceObject: QAppSettings
        sourceProperty: "backgroundBlur"
        targetObject: backgroundBlurControl
        targetProperty: "value"
    }

    Synchronizer {
        sourceObject: QAppSettings
        sourceProperty: "backgroundColorization"
        targetObject: backgroundColorizationControl
        targetProperty: "value"
    }

    Kirigami.FormLayout {
        anchors.fill: parent

        Item {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: "Album background"
        }

        QQC2.SpinBox {
            id: backgroundBlurControl
            Kirigami.FormData.label: "Background blur:"
            from: 0
            to: 100
        }

        QQC2.SpinBox {
            id: backgroundColorizationControl
            Kirigami.FormData.label: "Background colorization:"
            from: 0
            to: 100
        }
    }
}
