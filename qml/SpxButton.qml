import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

// Ghost button: no fill; text and border share one tone. The selected button
// (what Enter fires) and hovered/focused ones take the full accent.
// Destructive buttons are filled with the theme accent and white text.
Button {
    id: b

    readonly property bool isSpxButton: true
    property bool selected: false
    property bool destructive: false
    readonly property color accent: Theme.primary
    readonly property color fill: pressed ? Qt.darker(Theme.accent, 1.3)
        : lit ? Qt.lighter(Theme.accent, 1.25)
        : Theme.accent
    readonly property bool lit: hovered || visualFocus
    readonly property color tone: pressed ? Qt.darker(accent, 1.3)
        : selected || lit ? accent
        : Theme.textFaint

    hoverEnabled: true
    activeFocusOnTab: true
    implicitWidth: Math.max(80, contentItem.implicitWidth + 28)
    implicitHeight: 28
    opacity: enabled ? 1 : 0.35

    background: Rectangle {
        color: b.destructive ? b.fill : b.selected ? Qt.alpha(b.accent, 0.08) : "transparent"
        border.width: 1
        border.color: b.destructive ? (b.selected || b.lit ? "#ffffff" : b.fill) : b.tone
    }

    contentItem: Text {
        text: b.text
        color: b.destructive ? "#ffffff" : b.tone
        font.family: Theme.fontFamily
        font.pixelSize: 11
        font.letterSpacing: 1.3
        font.capitalization: Font.AllUppercase
        font.bold: b.selected
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }
}
