import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

TextField {
    id: f

    color: Theme.text
    font.family: Theme.fontFamily
    font.pixelSize: 12
    selectionColor: Theme.primary
    selectedTextColor: Theme.background
    placeholderTextColor: Theme.textFaint
    selectByMouse: true
    leftPadding: 10
    rightPadding: 10
    topPadding: 7
    bottomPadding: 7

    background: Rectangle {
        color: Theme.background
        border.width: 1
        border.color: f.activeFocus ? Theme.primary : Theme.border
    }
}
