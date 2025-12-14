pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Effects
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import github.skorodenko.ksol 1.0

Item {
    id: root

    required property string imageSource

    states: [
        State {
            name: "main"
            PropertyChanges { mainImage { opacity: 1 } }
            PropertyChanges { altImage { opacity: 0 } }
        },
        State {
            name: "alt"
            PropertyChanges { mainImage { opacity: 0 } }
            PropertyChanges { altImage { opacity: 1 } }
        }
    ]

    MultiEffect {
        source: root.state == "main" ? mainImage : altImage
        anchors.fill: root
        brightness: -0.15
        blurEnabled: true
        blurMax: 64
        blur: 0.75
    }

    Image {
        id: mainImage
        cache: false
        asynchronous: true
        retainWhileLoading: true
        mipmap: true
        visible: false
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop
        
        transitions: Transition {
            NumberAnimation { properties: "opacity"; easing.type: Easing.InOutQuad }
        }
    }

    Image {
        id: altImage
        cache: false
        asynchronous: true
        retainWhileLoading: true
        mipmap: true
        visible: false
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop

        transitions: Transition {
            NumberAnimation { properties: "opacity"; easing.type: Easing.InOutQuad }
        }
    }
}
