import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

TextField {
    id: f

    implicitHeight: 30
    leftPadding: 10
    rightPadding: 10
    color: Theme.text
    placeholderTextColor: Theme.textFaint
    selectionColor: Theme.selection
    selectedTextColor: Theme.selectionText
    font.family: Theme.fontFamily
    font.pointSize: Theme.fontSize
    selectByMouse: true

    background: Rectangle {
        radius: Theme.radius
        color: Theme.surfaceLowest
        border.width: 1
        border.color: f.activeFocus ? Theme.focus : Theme.border
    }
}
