import QtQuick.Controls
import QtQuick.Window
import test

ApplicationWindow {
    id: mainWindow

    visible: true

    Rectangle {
        anchors.fill: parent
        color: '#00ff00'

        MouseArea {
            anchors.fill: parent
            onClicked: testController.test_slot(5)
        }

    }

    TestController {
        id: testController

        onStarted: function() {
            console.log("started");
        }
        onFinished: function() {
            console.log("finished");
        }
        onValueChanged: function(i) {
            console.log(i);
        }
    }

}
