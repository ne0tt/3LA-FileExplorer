import QtQuick
import QtQuick.Shapes
import ThreeLA.FileExplorer

// Dialog chrome modelled on the 3LA-MilSpec Hyprland plugin: a 1px stroke
// with the top-left and bottom-right corners chamfered at 45°, a background
// fill, and an inset title bar whose left end follows the top-left cut.
Item {
    id: card

    property string header: ""
    property string tag: ""
    property color accent: Theme.primary
    property bool danger: false

    // Mirrors plugins.lua: chamfer 14, thickness 1, titlebar offset 5,
    // titlebar height 28, titlebar gap 6. Square corners when MilSpec is off.
    readonly property int chamfer: Theme.milSpec ? 14 : 0
    readonly property int inset: 5
    readonly property int barHeight: 28
    readonly property int headerHeight: inset + barHeight + 6

    // Same colour as the MilSpec window frame (col.active = primary), on
    // every dialog; danger is conveyed by the buttons and wording instead.
    readonly property color stroke: Theme.primary
    readonly property color fill: Theme.background
    readonly property color barFill: Theme.accent
    // Black or white, whichever contrasts with the bar (MilSpec text default).
    readonly property color barText: (0.299 * barFill.r + 0.587 * barFill.g + 0.114 * barFill.b) > 0.55 ? "#000000" : "#ffffff"

    // Swallow clicks on the chrome so they never reach the scrim.
    MouseArea { anchors.fill: parent }

    // Vector frame: drawn by the scene graph, so no per-dialog backing image
    // (a Canvas keeps a full-size QImage alive for every dialog shown).
    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            fillColor: card.fill
            strokeColor: card.stroke
            strokeWidth: 1
            startX: card.chamfer
            startY: 0.5
            PathLine { x: card.width - 0.5; y: 0.5 }
            PathLine { x: card.width - 0.5; y: card.height - card.chamfer }
            PathLine { x: card.width - card.chamfer; y: card.height - 0.5 }
            PathLine { x: 0.5; y: card.height - 0.5 }
            PathLine { x: 0.5; y: card.chamfer }
            PathLine { x: card.chamfer; y: 0.5 }
        }

        // Title bar, inset from the stroke; its top-left cut runs parallel
        // to the frame's chamfer.
        ShapePath {
            id: bar
            readonly property real i: card.inset + 1
            readonly property real cut: Math.max(0, card.chamfer - i * (2 - Math.SQRT2))
            fillColor: card.barFill
            strokeColor: "transparent"
            strokeWidth: -1
            startX: bar.i + bar.cut
            startY: bar.i
            PathLine { x: card.width - bar.i; y: bar.i }
            PathLine { x: card.width - bar.i; y: bar.i + card.barHeight }
            PathLine { x: bar.i; y: bar.i + card.barHeight }
            PathLine { x: bar.i; y: bar.i + bar.cut }
            PathLine { x: bar.i + bar.cut; y: bar.i }
        }
    }

    Text {
        x: card.inset + Math.max(card.chamfer, 6) + 4
        y: card.inset + 1
        height: card.barHeight
        verticalAlignment: Text.AlignVCenter
        text: card.header
        color: card.barText
        font.family: "Bebas Neue"
        font.pixelSize: 16
        font.letterSpacing: 1
        font.capitalization: Font.AllUppercase
    }

    Rectangle {
        visible: card.tag !== ""
        anchors.right: parent.right
        anchors.rightMargin: card.inset + 10
        y: card.inset + 1 + (card.barHeight - height) / 2
        width: tagText.implicitWidth + 12
        height: tagText.implicitHeight + 2
        color: "transparent"
        border.width: 1
        border.color: Qt.alpha(card.barText, 0.7)
        Text {
            id: tagText
            anchors.centerIn: parent
            text: card.tag
            color: card.barText
            font.family: Theme.fontFamily
            font.pixelSize: 10
            font.letterSpacing: 1.2
        }
    }
}
