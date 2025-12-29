pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Effects
import org.kde.kirigami as Kirigami

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
        colorization: 0.5
        colorizationColor: Kirigami.Theme.backgroundColor
        blurEnabled: true
        blurMax: 64
        blur: 0.75

        NumberAnimation on opacity {
            id: createAnimation
            from: 0
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
        source: altImage
        anchors.fill: root
        blurEnabled: true
        blurMax: 64
        blur: 0.75

        NumberAnimation on opacity {
            id: destroyAnimation
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
