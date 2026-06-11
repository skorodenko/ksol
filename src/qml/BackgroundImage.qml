pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Effects
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Item {
    id: root

    property string source
    property string sourceTmp

    onSourceChanged: {
        destroyAnimation.start();
        createAnimation.start();
        destroyAnimationS.start();
        createAnimationS.start();
    }

    MultiEffect {
        z: -1
        source: mainImage
        anchors.fill: root
        colorization: QAppSettings.backgroundOpacity / 100
        colorizationColor: Kirigami.Theme.backgroundColor
        blurEnabled: true
        blurMax: 64
        blur: QAppSettings.backgroundBlur / 100

        NumberAnimation on opacity {
            id: createAnimation
            from: 0.4
            to: 1
            duration: Kirigami.Units.shortDuration

            onRunningChanged: {
                if (!running) {
                    root.sourceTmp = root.source;
                }
            }
        }
    }

    MultiEffect {
        z: -1
        source: altImage
        anchors.fill: root
        colorization: QAppSettings.backgroundOpacity / 100
        colorizationColor: Kirigami.Theme.backgroundColor
        blurEnabled: true
        blurMax: 64
        blur: QAppSettings.backgroundBlur / 100

        NumberAnimation on opacity {
            id: destroyAnimation
            to: 0.3
            duration: 0.7 * Kirigami.Units.shortDuration
            onRunningChanged: {
                if (!running) {}
            }
        }
    }

    MultiEffect {
        z: -2
        source: mainImageS
        anchors.fill: root
        colorization: QAppSettings.backgroundOpacity / 100
        colorizationColor: Kirigami.Theme.backgroundColor
        blurEnabled: true
        blurMax: 64
        blurMultiplier: 3.0
        blur: QAppSettings.backgroundBlur / 100

        NumberAnimation on opacity {
            id: createAnimationS
            from: 0.4
            to: 1
            duration: Kirigami.Units.shortDuration

            onRunningChanged: {
                if (!running) {
                    root.sourceTmp = root.source;
                }
            }
        }
    }

    MultiEffect {
        z: -2
        source: altImageS
        anchors.fill: root
        colorization: QAppSettings.backgroundOpacity / 100
        colorizationColor: Kirigami.Theme.backgroundColor
        blurEnabled: true
        blurMax: 64
        blurMultiplier: 3.0
        blur: QAppSettings.backgroundBlur / 100

        NumberAnimation on opacity {
            id: destroyAnimationS
            to: 0.3
            duration: 0.7 * Kirigami.Units.shortDuration
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
        fillMode: Image.PreserveAspectFit
    }

    Image {
        id: mainImageS
        source: root.source
        visible: false
        anchors.fill: parent
        fillMode: Image.Stretch
    }

    Image {
        id: altImage
        visible: false
        source: root.sourceTmp
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
    }

    Image {
        id: altImageS
        visible: false
        source: root.sourceTmp
        anchors.fill: parent
        fillMode: Image.Stretch
    }
}
