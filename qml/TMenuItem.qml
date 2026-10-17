import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

// Glyphs ride on icon.name (a free string on both MenuItem and Action).
MenuItem {
    id: it

    // Tints the label and glyph with the theme accent (e.g. Move to Trash).
    property bool accent: false
    readonly property string glyph: icon.name
    readonly property string keys: action && action.shortcut ? String(action.shortcut) : ""

    implicitHeight: visible ? Theme.grow(28, Theme.menuFontSize) : 0
    height: visible ? implicitHeight : 0
    padding: 0
    leftPadding: 8
    rightPadding: 8
    hoverEnabled: true
    font.family: Theme.fontFamily
    font.pointSize: Theme.menuFontSize

    indicator: Item {}
    arrow: Text {
        x: it.width - width - 8
        anchors.verticalCenter: parent.verticalCenter
        visible: it.subMenu !== null
        text: G.chevronRight
        color: Theme.textDim
        font.family: Theme.glyphFamily
        font.pixelSize: Math.round(14 * Math.max(1, Theme.menuFontSize / 10))
    }

    contentItem: Item {
        implicitWidth: 18 + 10 + label.implicitWidth + (it.keys ? 28 + keysText.implicitWidth : 0) + (it.subMenu ? 24 : 4)
        implicitHeight: Theme.grow(28, Theme.menuFontSize)
        opacity: it.enabled ? 1 : 0.38

        Text {
            id: glyphText
            width: 18
            anchors.verticalCenter: parent.verticalCenter
            horizontalAlignment: Text.AlignHCenter
            text: it.checkable ? (it.checked ? G.check : "") : it.glyph
            color: it.highlighted ? Theme.selectionText : (it.checkable || it.accent ? Theme.primary : Theme.textDim)
            font.family: Theme.glyphFamily
            font.pixelSize: Math.round(15 * Math.max(1, Theme.menuFontSize / 10))
        }
        Text {
            id: label
            anchors.left: glyphText.right
            anchors.leftMargin: 10
            anchors.right: keysText.visible ? keysText.left : parent.right
            anchors.rightMargin: keysText.visible ? 20 : (it.subMenu ? 20 : 2)
            anchors.verticalCenter: parent.verticalCenter
            text: it.text.replace("&", "")
            elide: Text.ElideRight
            color: it.highlighted ? Theme.selectionText : it.accent ? Theme.primary : Theme.text
            font: it.font
        }
        Text {
            id: keysText
            visible: it.keys.length > 0
            anchors.right: parent.right
            anchors.rightMargin: it.subMenu ? 18 : 2
            anchors.verticalCenter: parent.verticalCenter
            text: it.keys
            color: it.highlighted ? Theme.selectionText : Theme.textFaint
            font.family: Theme.fontFamily
            font.pointSize: Math.max(6, Theme.menuFontSize - 1)
        }
    }

    background: Rectangle {
        implicitWidth: Math.round(300 * Math.max(1, Theme.menuFontSize / 10))
        radius: Theme.smallRadius
        color: it.highlighted ? Theme.selection : "transparent"
    }
}
