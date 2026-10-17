import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

CheckBox {
    id: c

    spacing: 10
    opacity: enabled ? 1 : 0.5

    indicator: Rectangle {
        implicitWidth: 14
        implicitHeight: 14
        x: c.leftPadding
        y: c.topPadding + (c.availableHeight - height) / 2
        color: c.checked ? Theme.primary : "transparent"
        border.width: 1
        border.color: c.checked || c.hovered ? Theme.primary : Theme.border
        Text {
            anchors.centerIn: parent
            text: c.checked ? "✓" : ""
            color: Theme.background
            font.pixelSize: 10
            font.bold: true
        }
    }

    contentItem: Text {
        text: c.text
        color: Theme.textDim
        font.family: Theme.fontFamily
        font.pixelSize: 12
        leftPadding: c.indicator.width + c.spacing
        verticalAlignment: Text.AlignVCenter
        wrapMode: Text.WordWrap
    }
}
