import QtQuick
import ThreeLA.FileExplorer

// Picker row: accent tint + border when current.
Rectangle {
    property bool current: false
    property bool hot: false

    color: current ? Qt.alpha(Theme.primary, 0.10) : hot ? Theme.hover : "transparent"
    border.width: current ? 1 : 0
    border.color: Theme.primary
}
