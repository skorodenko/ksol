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
            value: QSettingsModel.backgroundBlur
            from: 0
            to: 100

            onValueChanged: {
                QSettingsModel.backgroundBlur = value;
            }
        }

        QQC2.SpinBox {
            Kirigami.FormData.label: "Background opacity:"
            value: QSettingsModel.backgroundOpacity
            from: 0
            to: 100

            onValueChanged: {
                QSettingsModel.backgroundOpacity = value;
            }
        }
    }
}
