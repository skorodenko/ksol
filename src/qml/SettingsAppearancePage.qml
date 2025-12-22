pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Kirigami.Page {
    id: root

    ColumnLayout {
        anchors.fill: parent
        RowLayout {
            QQC2.Label {
                text: "Opacity: "
            }
            QQC2.Slider {
                Layout.fillWidth: true
                from: 0
                to: 1
            }
        }
    }
}
