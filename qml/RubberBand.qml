import QtQuick
import ThreeLA.FileExplorer

// Lives inside a Flickable's contentItem, underneath the delegates, so it only
// sees presses on empty space. Coordinates are content coordinates.
MouseArea {
    id: rb

    required property var pane
    required property Flickable flick
    required property var hitTest

    property point start
    property point cur
    property bool banding: false
    property var base: ({})
    property var lastRows: []

    function sameRows(a, b) {
        if (a.length !== b.length) return false
        for (let i = 0; i < a.length; ++i)
            if (a[i] !== b[i]) return false
        return true
    }

    // Declared inside a GridView/ListView this would be parented to the view
    // itself, beneath the content layer, and never see presses.
    parent: flick.contentItem
    z: -1
    width: Math.max(flick.contentWidth, flick.width)
    height: Math.max(flick.contentHeight, flick.height)
    acceptedButtons: Qt.LeftButton | Qt.RightButton
    preventStealing: true

    function update() {
        const x1 = Math.min(start.x, cur.x), x2 = Math.max(start.x, cur.x)
        const y1 = Math.min(start.y, cur.y), y2 = Math.max(start.y, cur.y)
        band.x = x1
        band.y = y1
        band.width = x2 - x1
        band.height = y2 - y1
        // Most mouse moves don't change which items the band covers; only a
        // changed set is worth a selection commit (which touches every delegate).
        const rows = hitTest(x1, y1, x2, y2)
        if (sameRows(rows, lastRows)) return
        lastRows = rows
        pane.setSelectionRows(rows, base)
    }

    onPressed: mouse => {
        pane.keyboardCursor = false
        pane.forceActiveFocus()
        if (mouse.button === Qt.RightButton) {
            if (!(mouse.modifiers & Qt.ControlModifier))
                pane.clearSelection()
            return
        }
        start = Qt.point(mouse.x, mouse.y)
        cur = start
        banding = false
        lastRows = []
        const additive = mouse.modifiers & (Qt.ControlModifier | Qt.ShiftModifier)
        base = additive ? Object.assign({}, pane.selection) : ({})
        if (!additive)
            pane.clearSelection()
    }

    onPositionChanged: mouse => {
        if (!(pressedButtons & Qt.LeftButton))
            return
        cur = Qt.point(mouse.x, mouse.y)
        if (!banding && Math.abs(cur.x - start.x) + Math.abs(cur.y - start.y) > 4)
            banding = true
        if (banding)
            update()
    }

    onReleased: banding = false
    onCanceled: banding = false

    onClicked: mouse => {
        if (mouse.button === Qt.RightButton)
            pane.backgroundMenuRequested()
    }

    Rectangle {
        id: band
        visible: rb.banding
        color: Qt.alpha(Theme.primaryContainer, 0.18)
        border.color: Theme.primaryContainer
        border.width: 1
        radius: 2
        z: 10
    }

    Timer {
        interval: 30
        repeat: true
        running: rb.banding
        onTriggered: {
            const f = rb.flick
            const vy = rb.cur.y - f.contentY
            const vx = rb.cur.x - f.contentX
            let dy = 0, dx = 0
            if (vy < 0) dy = Math.max(-30, vy)
            else if (vy > f.height) dy = Math.min(30, vy - f.height)
            if (vx < 0) dx = Math.max(-30, vx)
            else if (vx > f.width) dx = Math.min(30, vx - f.width)
            if (!dx && !dy)
                return
            const maxY = Math.max(0, f.contentHeight - f.height)
            const maxX = Math.max(0, f.contentWidth - f.width)
            const ny = Math.max(0, Math.min(maxY, f.contentY + dy))
            const nx = Math.max(0, Math.min(maxX, f.contentX + dx))
            rb.cur = Qt.point(rb.cur.x + (nx - f.contentX), rb.cur.y + (ny - f.contentY))
            f.contentY = ny
            f.contentX = nx
            rb.update()
        }
    }
}
