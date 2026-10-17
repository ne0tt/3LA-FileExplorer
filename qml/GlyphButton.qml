import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

AbstractButton {
    id: b

    property string glyph
    property string tip
    property int glyphSize: 16
    property color glyphColor: Theme.text

    hoverEnabled: true
    focusPolicy: Qt.NoFocus
    padding: 5
    leftPadding: text ? 8 : 5
    rightPadding: text ? 10 : 5
    implicitHeight: 30
    implicitWidth: Math.max(30, implicitContentWidth + leftPadding + rightPadding)
    opacity: enabled ? 1 : 0.38

    contentItem: Item {
        implicitWidth: row.implicitWidth
        implicitHeight: row.implicitHeight
    Row {
        id: row
        anchors.centerIn: parent
        spacing: 6
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: b.glyph
            color: b.checked ? Theme.selectionText : b.glyphColor
            font.family: Theme.glyphFamily
            font.pixelSize: b.glyphSize
            visible: b.glyph.length > 0
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: b.text
            color: b.checked ? Theme.selectionText : Theme.text
            font.family: Theme.fontFamily
            font.pointSize: Theme.fontSize
            font.bold: b.font.bold
            visible: b.text.length > 0
        }
    }
    }

    background: Rectangle {
        radius: Theme.radius
        color: b.down ? Theme.pressed : b.checked ? Theme.selection : b.hovered ? Theme.hover : "transparent"
        Behavior on color { ColorAnimation { duration: Theme.animMs } }
    }

    ToolTip.visible: hovered && tip.length > 0
    ToolTip.text: tip
    ToolTip.delay: 600
}
