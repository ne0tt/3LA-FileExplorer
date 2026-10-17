import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

// The file or folder a dialog acts on: one line under the title, smaller than
// it, keeping the name's case and eliding the middle if it is still too long.
Text {
    color: Theme.text
    font.family: Theme.displayFamily
    font.weight: Font.DemiBold
    font.pixelSize: 15
    font.letterSpacing: 0.5
    elide: Text.ElideMiddle
    maximumLineCount: 1
    visible: text.length > 0
    Layout.fillWidth: true
}
