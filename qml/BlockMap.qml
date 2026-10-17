import QtQuick
import ThreeLA.FileExplorer

// Block map that fills in a scattered left-to-right order, with a flickering
// write frontier ahead of the filled cells (from SPX-FileExplorer).
Canvas {
    id: map

    property bool running: false
    property real progress: 0
    property color accent: Theme.primary
    property bool frontier: true
    readonly property int cell: 12
    readonly property int cols: Math.max(1, Math.floor(width / cell))
    readonly property int rows: Math.max(1, Math.floor(height / cell))
    property var rank: []
    property real tick: 0

    function rgba(value, a) {
        const c = Qt.color(value)
        return "rgba(" + Math.round(c.r * 255) + "," + Math.round(c.g * 255) + "," + Math.round(c.b * 255) + "," + a + ")"
    }

    function shuffle() {
        const n = cols * rows
        const order = []
        for (let i = 0; i < n; i++)
            order.push([i, i + Math.random() * cols * 1.2])
        order.sort((a, b) => a[1] - b[1])
        const r = new Array(n)
        order.forEach((pair, idx) => r[pair[0]] = idx)
        rank = r
    }

    onColsChanged: shuffle()
    onRowsChanged: shuffle()
    onRunningChanged: if (running) shuffle()
    onProgressChanged: requestPaint()
    Component.onCompleted: shuffle()

    // Each tick repaints the whole canvas on the GUI thread; ~7 fps is plenty.
    Timer {
        interval: 150
        repeat: true
        running: map.running && map.visible
        onTriggered: {
            map.tick += 1
            map.requestPaint()
        }
    }

    onPaint: {
        const g = getContext("2d")
        g.reset()
        const n = cols * rows
        if (rank.length !== n)
            return
        const filled = Math.floor(Math.max(0, Math.min(1, progress)) * n)
        const front = frontier ? Math.ceil(cols * 0.7) : 0
        const fill = rgba(accent, 0.85)
        const hot = rgba(accent, 1)
        const warm = rgba(accent, 0.35)
        const idle = rgba(Theme.border, 1)
        g.lineWidth = 1
        for (let c = 0; c < n; c++) {
            const x = (c % cols) * cell
            const y = Math.floor(c / cols) * cell
            const r = rank[c]
            if (r < filled) {
                g.fillStyle = fill
                g.fillRect(x, y, cell - 3, cell - 3)
            } else if (r < filled + front) {
                g.strokeStyle = ((tick + c * 7) % 5 < 2) ? hot : warm
                g.strokeRect(x + 0.5, y + 0.5, cell - 4, cell - 4)
            } else {
                g.strokeStyle = idle
                g.strokeRect(x + 0.5, y + 0.5, cell - 4, cell - 4)
            }
        }
    }
}
