import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// SPX transfer panel: block map on the left, live stats on the right. Shown
// for any FileOps job that runs longer than a moment; "Hide" sends it to the
// background (the status bar keeps showing the percentage).
SpxDialog {
    id: d

    // Job finished successfully; "completed" waits for the blocks to catch up.
    property bool ended: false
    readonly property bool completed: ended && shown >= 1
    property bool hiddenByUser: false
    property bool cancelling: false
    property string kindLatched: ""
    property real rate: 0
    property var samples: []
    property real startedMs: 0
    property real elapsed: 0
    property bool snap: false
    // Shown progress eases towards the real value so fast jobs still visibly sweep.
    property real shown: ended ? 1 : FileOps.progress
    Behavior on shown {
        enabled: d.visible && !d.snap
        SmoothedAnimation { velocity: 1.25 }
    }
    onCompletedChanged: if (completed) completeHold.restart()

    readonly property bool byteMode: FileOps.totalBytes > 0
    readonly property real done: byteMode ? FileOps.doneBytes : FileOps.doneItems
    readonly property real total: byteMode ? FileOps.totalBytes : FileOps.totalItems
    readonly property bool destructive: ["trash", "delete", "empty_trash"].indexOf(kindLatched) >= 0
    readonly property color tone: completed ? Theme.success : destructive ? Theme.danger : Theme.primary

    function fmtBytes(b) {
        const u = ["B", "kB", "MB", "GB", "TB"]
        let i = 0
        while (b >= 1000 && i < u.length - 1) { b /= 1000; ++i }
        return (i === 0 ? Math.round(b) : b.toFixed(1)) + " " + u[i]
    }
    function fmtAmount(v) { return byteMode ? fmtBytes(v) : Math.round(v).toLocaleString(Qt.locale(), "f", 0) }
    function fmtRate(v) { return v <= 0 ? "--" : byteMode ? fmtBytes(v) + "/s" : Math.round(v) + " items/s" }
    function fmtClock(s) {
        if (s < 0 || !isFinite(s)) return "--:--"
        s = Math.round(s)
        const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = s % 60
        return (h > 0 ? h + ":" + String(m).padStart(2, "0") : m) + ":" + String(sec).padStart(2, "0")
    }

    function reset() {
        kindLatched = FileOps.kind
        snap = true
        ended = false
        snap = false
        cancelling = false
        rate = 0
        samples = []
        startedMs = Date.now()
        elapsed = 0
    }

    function begin() {
        reset()
        if (!hiddenByUser) open()
    }

    function cancelJob() {
        if (ended) { close(); return }
        if (cancelling) return
        cancelling = true
        FileOps.cancel()
    }

    header: ({ copy: "COPY", move: "MOVE", link: "LINK", trash: "MOVE TO TRASH", delete: "DELETE", empty_trash: "EMPTY TRASH", extract: "EXTRACT", compress: "COMPRESS", encrypt: "ENCRYPT", decrypt: "DECRYPT" })[kindLatched] || "TRANSFER"
    tag: completed ? "COMPLETE"
        : kindLatched === "trash" ? "RECOVERABLE"
        : destructive ? "BURN AFTER READING"
        : "IN TRANSIT"
    accent: tone
    danger: destructive && !completed
    cardWidth: 660
    closePolicy: Popup.NoAutoClose
    buttonRow: footer
    defaultIndex: 1
    onEscapePressed: cancelJob()

    Timer {
        id: showDelay
        interval: 350
        onTriggered: if (FileOps.busy) d.begin()
    }
    Timer {
        id: completeHold
        interval: 1000
        onTriggered: d.close()
    }
    Timer {
        interval: 250
        repeat: true
        running: FileOps.busy && d.visible
        onTriggered: {
            const now = Date.now()
            const s = d.samples
            s.push([now, d.done])
            while (s.length > 2 && now - s[0][0] > 3000)
                s.shift()
            const dt = (now - s[0][0]) / 1000
            if (dt >= 0.5)
                d.rate = Math.max(0, (d.done - s[0][1]) / dt)
            d.elapsed = (now - d.startedMs) / 1000
        }
    }

    Connections {
        target: FileOps
        function onBusyChanged() {
            if (FileOps.busy) d.hiddenByUser = false
        }
        function onStarted() {
            completeHold.stop()
            if (d.visible)
                d.reset()
            else if (["trash", "delete", "empty_trash"].indexOf(FileOps.kind) >= 0)
                d.begin()
            else
                showDelay.restart()
        }
        function onFinished(created, cancelled, failed) {
            if (FileOps.queued > 0) return
            showDelay.stop()
            if (!d.visible) return
            if (cancelled || failed) {
                d.close()
            } else {
                d.ended = true
                d.elapsed = (Date.now() - d.startedMs) / 1000
            }
        }
    }

    Item {
        Layout.fillWidth: true
        implicitHeight: 230

        BlockMap {
            id: map
            x: 0
            y: 0
            width: parent.width - side.width - 25
            height: parent.height
            running: d.visible
            progress: d.shown
            accent: d.kindLatched === "decrypt" ? Theme.success : d.kindLatched === "encrypt" ? Theme.cryptRed : d.tone
            frontier: !d.cancelling && !d.completed
        }

        Rectangle {
            x: side.x - 9
            y: -4
            width: 1
            height: parent.height + 8
            color: Theme.border
        }

        ColumnLayout {
            id: side
            x: parent.width - width
            width: 200
            height: parent.height
            spacing: 4

            Text {
                text: {
                    const k = d.kindLatched, done = d.completed
                    switch (k) {
                    case "move": return done ? "MOVED" : "MOVING"
                    case "link": return done ? "LINKED" : "LINKING"
                    case "trash": return done ? "MOVED TO TRASH" : "MOVING TO TRASH"
                    case "delete": return done ? "FILES DELETED" : "DELETING FILES"
                    case "empty_trash": return done ? "TRASH EMPTIED" : "EMPTYING TRASH"
                    case "extract": return done ? "EXTRACTED" : "EXTRACTING"
                    case "compress": return done ? "COMPRESSED" : "COMPRESSING"
                    case "encrypt": return done ? "ENCRYPTED" : "ENCRYPTING"
                    case "decrypt": return done ? "DECRYPTED" : "DECRYPTING"
                    default: return done ? "COMPLETE" : "COPYING"
                    }
                }
                color: d.completed ? d.tone : Theme.text
                font.family: Theme.displayFamily
                font.weight: Font.DemiBold
                font.pixelSize: 15
                font.letterSpacing: 1.5
            }
            Text {
                text: FileOps.title
                color: Theme.textFaint
                font.family: Theme.fontFamily
                font.pixelSize: 10
                elide: Text.ElideRight
                Layout.fillWidth: true
            }
            Text {
                text: Math.floor(d.shown * 100) + "%"
                color: d.tone
                font.family: Theme.displayFamily
                font.weight: Font.DemiBold
                font.pixelSize: 34
                Layout.topMargin: 6
            }
            Text {
                text: d.completed ? "✓ done" : d.cancelling ? "Cancelling…" : (FileOps.currentFile || "Preparing…")
                color: d.completed ? d.tone : Theme.text
                font.family: Theme.fontFamily
                font.pixelSize: 11
                elide: Text.ElideMiddle
                Layout.fillWidth: true
            }
            SpxStat {
                Layout.topMargin: 6
                label: d.byteMode ? "TRANSFERRED" : "ITEMS"
                value: d.fmtAmount(d.done) + " / " + d.fmtAmount(d.total)
            }
            SpxStat {
                label: "SPEED"
                value: d.completed ? "avg " + d.fmtRate(d.done / Math.max(d.elapsed, 0.001)) : d.fmtRate(d.rate)
            }
            SpxStat {
                label: "REMAINING"
                value: d.completed ? "0:00" : d.fmtClock(FileOps.etaSeconds)
            }
            SpxStat {
                label: "ELAPSED"
                value: d.fmtClock(d.elapsed)
            }
            SpxStat {
                visible: FileOps.queued > 0
                label: "QUEUED"
                value: FileOps.queued + " job" + (FileOps.queued === 1 ? "" : "s")
            }
            Item { Layout.fillHeight: true }
        }
    }

    SpxFooter {
        id: footer
        opacity: d.completed ? 0 : 1
        SpxButton {
            text: "Hide"
            onClicked: {
                d.hiddenByUser = true
                d.close()
            }
        }
        SpxButton {
            text: d.cancelling ? "Cancelling…" : "Cancel"
            destructive: d.destructive
            enabled: !d.cancelling
            onClicked: d.cancelJob()
        }
    }
}
