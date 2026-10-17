import QtQuick
import ThreeLA.FileExplorer

// Shared press/drag/drop behaviour for a file delegate. Mirrors Thunar:
// click selects, Ctrl toggles, Shift extends, drag carries the selection,
// folders accept drops.
MouseArea {
    id: ma

    required property var pane
    required property int row
    required property string path
    required property bool isDir
    readonly property bool dropHover: dropArea.containsDrag && dropArea.accepting

    property bool deferSelect: false

    anchors.fill: parent
    acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
    hoverEnabled: true
    preventStealing: true
    drag.target: proxy
    drag.threshold: 8
    cursorShape: pane.singleClick && containsMouse ? Qt.PointingHandCursor : Qt.ArrowCursor

    onPressed: mouse => {
        pane.keyboardCursor = false
        pane.forceActiveFocus()
        deferSelect = false
        if (mouse.button === Qt.RightButton) {
            if (!pane.isSelected(path))
                pane.selectOnly(row)
            return
        }
        if (mouse.button === Qt.MiddleButton)
            return
        if (mouse.modifiers & Qt.ShiftModifier)
            pane.selectRange(row, mouse.modifiers & Qt.ControlModifier)
        else if (mouse.modifiers & Qt.ControlModifier)
            pane.toggleRow(row)
        else if (!pane.isSelected(path))
            pane.selectOnly(row)
        else
            deferSelect = true
    }

    // The URI list is built once, when a drag actually starts, rather than on
    // every press (and re-evaluated by bindings in every delegate).
    Connections {
        target: ma.drag
        function onActiveChanged() {
            if (ma.drag.active) {
                const uris = ma.pane.selectedUris()
                ma.pane.dragUris = uris
                proxy.Drag.mimeData = { "text/uri-list": uris.join("\r\n") }
                // A theme icon, not grabToImage(): an async grab of a delegate
                // that is destroyed mid-render (e.g. double-click navigates
                // away) crashes the scene-graph render thread.
                proxy.Drag.imageSource = "image://icon/" + (uris.length > 1 ? "edit-copy" : ma.isDir ? "folder" : "text-x-generic")
            }
            proxy.Drag.active = ma.drag.active
        }
    }

    onReleased: mouse => {
        proxy.x = 0
        proxy.y = 0
        if (deferSelect && !drag.active)
            pane.selectOnly(row)
        deferSelect = false
    }

    onClicked: mouse => {
        if (mouse.button === Qt.RightButton) {
            pane.itemMenuRequested(row)
        } else if (mouse.button === Qt.MiddleButton) {
            if (isDir)
                pane.openInNewTab(path)
            else
                pane.activateRow(row)
        } else if (pane.singleClick && !(mouse.modifiers & (Qt.ControlModifier | Qt.ShiftModifier))) {
            pane.activateRow(row)
        }
    }

    onDoubleClicked: mouse => {
        if (mouse.button === Qt.LeftButton && !pane.singleClick)
            pane.activateRow(row)
    }

    Item {
        id: proxy
        width: 1
        height: 1
        Drag.dragType: Drag.Automatic
        Drag.supportedActions: Qt.CopyAction | Qt.MoveAction | Qt.LinkAction
        Drag.proposedAction: Qt.MoveAction
        Drag.hotSpot.x: 16
        Drag.hotSpot.y: 16
        Drag.onDragFinished: ma.pane.dragUris = []
    }

    DropArea {
        id: dropArea

        // Decided once per entry rather than kept as a binding over dragUris.
        property bool accepting: false

        anchors.fill: parent
        enabled: ma.isDir
        onEntered: drag => {
            accepting = drag.hasUrls && !ma.pane.isTrash && !ma.pane.dragUris.includes(Sys.toUri(ma.path))
            if (!accepting)
                drag.accepted = false
        }
        onExited: accepting = false
        onDropped: drop => {
            accepting = false
            ma.pane.handleDrop(drop, ma.path)
        }
    }
}
