import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

RowLayout {
    property string label
    property string value
    property color valueColor: Theme.text

    Layout.fillWidth: true

    Text {
        text: parent.label
        color: Theme.textFaint
        font.family: Theme.fontFamily
        font.pixelSize: 9
    }
    Item { Layout.fillWidth: true }
    Text {
        text: parent.value
        color: parent.valueColor
        font.family: Theme.fontFamily
        font.pixelSize: 11
    }
}
