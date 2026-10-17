import QtQuick
import ThreeLA.FileExplorer

Rectangle {
    id: t

    function show(msg) {
        label.text = msg
        opacity = 1
        hide.restart()
    }

    width: Math.min(parent ? parent.width - 40 : 600, label.implicitWidth + 60)
    height: label.implicitHeight + 20
    radius: Theme.radius + 2
    color: Theme.chrome
    border.color: Theme.border
    opacity: 0
    visible: opacity > 0
    Behavior on opacity { NumberAnimation { duration: 180 } }

    Row {
        anchors.centerIn: parent
        spacing: 10
        Glyph { text: G.info; color: Theme.primary; size: 16; anchors.verticalCenter: parent.verticalCenter }
        TText {
            id: label
            width: Math.min(implicitWidth, t.parent ? t.parent.width - 100 : 540)
            wrapMode: Text.Wrap
            anchors.verticalCenter: parent.verticalCenter
        }
    }
    MouseArea {
        anchors.fill: parent
        onClicked: t.opacity = 0
    }
    Timer {
        id: hide
        interval: 5000
        onTriggered: t.opacity = 0
    }
}
