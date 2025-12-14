pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Effects
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Item {
    id: root

    property string source
    property string sourceTmp

    onSourceChanged: {
        destroyAnimation.start();
        createAnimation.start();
    }

    MultiEffect {
        source: mainImage
        anchors.fill: root
        brightness: -0.15
        blurEnabled: true
        blurMax: 64
        blur: 0.75

        NumberAnimation on opacity {
            id: createAnimation
            from: 0
            to: 1
            duration: Kirigami.Units.longDuration

            onRunningChanged: {
                if (!running) {
                    root.sourceTmp = root.source;
                }
            }
        }
    }

    MultiEffect {
        source: altImage
        anchors.fill: root
        brightness: -0.15
        blurEnabled: true
        blurMax: 64
        blur: 0.75

        NumberAnimation on opacity {
            id: destroyAnimation
            to: 0.3
            duration: 0.7 * Kirigami.Units.longDuration
            onRunningChanged: {
                if (!running) {}
            }
        }
    }

    Image {
        id: mainImage
        source: root.source
        visible: false
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop
    }

    Image {
        id: altImage
        visible: false
        source: root.sourceTmp
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop
    }
}
