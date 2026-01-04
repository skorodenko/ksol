pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.Page {
    id: root

    Kirigami.FormLayout {
        anchors.fill: parent
        
        Item {
            Kirigami.FormData.isSection: true
            Kirigami.FormData.label: "Album background"
        }

        QQC2.SpinBox {
            Kirigami.FormData.label: "Background blur:"
            from: 0
            to: 100
        }

        QQC2.SpinBox {
            Kirigami.FormData.label: "Background opacity:"
            from: 0
            to: 100
        }
    }
}
