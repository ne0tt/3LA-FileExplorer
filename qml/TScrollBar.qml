import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

ScrollBar {
    id: s

    minimumSize: 0.05
    padding: 2
    contentItem: Rectangle {
        implicitWidth: 6
        implicitHeight: 6
        radius: 3
        color: s.pressed ? Theme.primary : s.hovered ? Theme.outline : Qt.alpha(Theme.outline, 0.55)
        opacity: s.policy === ScrollBar.AlwaysOn || s.active || s.hovered ? 1 : 0.0
        Behavior on opacity { NumberAnimation { duration: 200 } }
    }
}
