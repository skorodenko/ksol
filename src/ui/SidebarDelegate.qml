import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

ItemDelegate {
    id: delegate

    property real leadingPadding: 0
    property string iconName: ""
    property bool selected: delegate.highlighted || delegate.pressed
    property bool showDefaultIndicator: false

    contentItem: RowLayout {
        spacing: Kirigami.Units.smallSpacing

        Kirigami.IconTitleSubtitle {
            id: titleItem
            Layout.fillWidth: true
            Layout.leftMargin: delegate.leadingPadding
            icon.name: delegate.iconName 
            title: delegate.text
            selected: delegate.selected
        }

        Rectangle {
            Layout.alignment: Qt.AlignVCenter
            Layout.preferredWidth: Kirigami.Units.largeSpacing
            Layout.preferredHeight: Kirigami.Units.largeSpacing

            radius: width * 0.5
            visible: delegate.showDefaultIndicator
            Kirigami.Theme.colorSet: Kirigami.Theme.View
            color: Kirigami.Theme.neutralTextColor
        }
    }
}
