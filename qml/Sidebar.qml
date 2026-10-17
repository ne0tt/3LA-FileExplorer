import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

Rectangle {
    id: side

    property string currentPath: ""
    // Text size for the sidebar, set independently of the file views.
    property int fontSize: Theme.fontSize

    // Like Theme.grow, but for the sidebar's own text size.
    function grow(px) {
        return Math.round(px + Math.max(0, fontSize - 10) * 1.6)
    }
    property var collapsed: ({})
    // Saved order of the Computer entries (paths); unknown entries go last.
    property var placeOrder: []

    readonly property var places: {
        const list = parse(Sys.placesJson)
        const rank = p => { const i = placeOrder.indexOf(p.path); return i < 0 ? 1000 + list.indexOf(p) : i }
        return list.slice().sort((a, b) => rank(a) - rank(b))
    }
    readonly property var devices: parse(Sys.devicesJson).filter(d => !d.network)
    readonly property var network: [{ name: "Browse Network", mountPoint: "network:///", glyph: G.lan, browse: true }]
        .concat(parse(Sys.devicesJson).filter(d => d.network))
    readonly property string currentDisplay: Sys.displayPath(currentPath)
    // One GTK bookmarks file feeds two sections; `gidx` is each entry's line
    // in that file so reordering inside a section maps back correctly.
    readonly property var allBookmarks: parse(Sys.bookmarksJson).map((b, i) => Object.assign({ gidx: i }, b))
    readonly property var bookmarks: allBookmarks.filter(b => !b.remote)
    readonly property var shares: allBookmarks.filter(b => b.remote)

    signal navigateRequested(string path)
    signal newTabRequested(string path)
    signal newWindowRequested(string path)
    signal menuRequested(var item, string kind)
    signal dropRequested(var drop, string path)
    signal placeOrderEdited(var order)

    // --- drag-to-reorder (Computer and Places entries) ---------------------
    property var reorder: null          // { kind, from, to }
    property bool suppressClick: false

    function repeaterFor(kind) {
        return kind === "bookmark" ? bookmarksRep : kind === "share" ? sharesRep : placesRep
    }

    function beginReorder(entry) {
        reorder = { kind: entry.kind, from: entry.idx, to: entry.idx }
        ghost.text = entry.item.name
        ghost.glyph = entry.item.glyph || G.folder
    }

    // `to` is an insertion slot in the original list (0..count): the gap
    // above item `to`, or after the last item when to == count.
    function updateReorder(y) {
        if (!reorder) return
        const rep = repeaterFor(reorder.kind)
        let to = rep.count
        for (let i = 0; i < rep.count; ++i) {
            const it = rep.itemAt(i)
            if (y < it.mapToItem(side, 0, 0).y + it.height / 2) { to = i; break }
        }
        reorder = { kind: reorder.kind, from: reorder.from, to: to }
        ghost.y = y - ghost.height / 2
        const last = rep.itemAt(rep.count - 1)
        indicator.y = to < rep.count
            ? rep.itemAt(to).mapToItem(side, 0, 0).y - 1
            : last.mapToItem(side, 0, 0).y + last.height - 1
    }

    function endReorder() {
        const r = reorder
        reorder = null
        suppressClick = true
        if (!r) return
        // Removing the item first shifts every slot below it up by one.
        const dest = r.to > r.from ? r.to - 1 : r.to
        if (dest === r.from) return
        if (r.kind === "bookmark" || r.kind === "share") {
            const list = r.kind === "share" ? shares : bookmarks
            // Index of the target item in the file, before the moved line is removed.
            Sys.moveBookmark(list[r.from].gidx, list[dest].gidx)
        } else {
            const order = places.map(p => p.path)
            const moved = order.splice(r.from, 1)[0]
            order.splice(dest, 0, moved)
            placeOrderEdited(order)
        }
    }

    function parse(j) {
        try { return JSON.parse(j || "[]") } catch (e) { return [] }
    }

    function toggle(section) {
        const c = Object.assign({}, collapsed)
        c[section] = !c[section]
        collapsed = c
    }

    property string pendingMountDevice: ""
    property bool pendingNewTab: false

    function openDevice(d, newTab) {
        if (d.mounted) {
            newTab ? newTabRequested(d.mountPoint) : navigateRequested(d.mountPoint)
            return
        }
        pendingMountDevice = d.device
        pendingNewTab = newTab
        Sys.mount(d.device)
    }

    Connections {
        target: Sys
        function onMountFinished(device, mountPoint, error) {
            if (device !== side.pendingMountDevice)
                return
            side.pendingMountDevice = ""
            if (mountPoint)
                side.pendingNewTab ? side.newTabRequested(mountPoint) : side.navigateRequested(mountPoint)
        }
    }

    color: Theme.sidebar

    // Each refresh runs lsblk and statvfs, so poll only while the pane is shown
    // and the app has focus; becoming active again refreshes at once.
    Timer {
        interval: 10000
        running: side.visible && Qt.application.state === Qt.ApplicationActive
        repeat: true
        triggeredOnStart: true
        onTriggered: Sys.refreshDevices()
    }

    function bookmarkDrop(d) {
        for (const u of d.urls) {
            const p = Sys.fromUri(String(u))
            if (Sys.isDir(p)) Sys.addBookmark(p)
        }
        d.accept(Qt.LinkAction)
    }

    component SectionHeader: Item {
        id: sh
        required property string title
        required property string key
        width: parent ? parent.width : 0
        height: side.grow(30)
        Row {
            anchors.left: parent.left
            anchors.leftMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 4
            TText {
                text: sh.title.toUpperCase()
                color: Theme.textFaint
                font.pointSize: Math.max(6, side.fontSize - 1)
                font.bold: true
                font.letterSpacing: 1
            }
            Glyph {
                text: side.collapsed[sh.key] ? G.chevronRight : G.chevronDown
                color: Theme.textFaint
                size: 12
                opacity: hdrMa.containsMouse ? 1 : 0
            }
        }
        MouseArea {
            id: hdrMa
            anchors.fill: parent
            hoverEnabled: true
            onClicked: side.toggle(sh.key)
        }
        // Folders dropped on the PLACES heading are bookmarked.
        Rectangle {
            anchors.fill: parent
            radius: Theme.radius
            visible: hdrDrop.containsDrag
            color: Theme.hover
            border.color: Theme.dropTarget
        }
        DropArea {
            id: hdrDrop
            anchors.fill: parent
            enabled: sh.key === "bookmarks"
            onEntered: d => { if (!d.hasUrls) d.accepted = false }
            onDropped: d => side.bookmarkDrop(d)
        }
    }

    component Entry: Rectangle {
        id: e
        required property var item
        required property string kind
        property int idx: -1
        readonly property bool movable: kind === "bookmark" || kind === "share" || kind === "place"
        readonly property bool beingMoved: !!side.reorder && side.reorder.kind === kind && side.reorder.from === idx
        property bool dropOk: (kind !== "device" || item.mounted) && !item.remote && !item.browse
        readonly property string target: kind === "device" || kind === "network" ? (item.mountPoint || "") : item.path
        readonly property bool active: target !== "" && (target === side.currentPath || target === side.currentDisplay)

        width: parent ? parent.width : 0
        height: side.grow(30)
        radius: Theme.radius
        color: active ? Theme.selection : ema.containsMouse || drop.containsDrag ? Theme.hover : "transparent"
        opacity: beingMoved ? 0.35 : 1
        border.width: drop.containsDrag ? 2 : 0
        border.color: Theme.dropTarget

        Glyph {
            id: eg
            x: 12
            anchors.verticalCenter: parent.verticalCenter
            width: 20
            text: e.item.glyph || G.folder
            size: Math.max(12, Math.round(side.fontSize * 1.6))
            color: e.active ? Theme.selectionText : (e.kind === "device" ? Theme.secondary : e.kind === "network" ? Theme.tertiaryContainer : Theme.primaryContainer)
            opacity: e.item.exists === false ? 0.4 : 1
        }
        TText {
            anchors.left: eg.right
            anchors.leftMargin: 10
            anchors.right: ejectBtn.visible ? ejectBtn.left : parent.right
            anchors.rightMargin: 6
            anchors.verticalCenter: parent.verticalCenter
            anchors.verticalCenterOffset: usage.visible ? -3 : 0
            text: e.item.name
            elide: Text.ElideRight
            color: e.active ? Theme.selectionText : Theme.text
            font.pointSize: side.fontSize
            font.italic: e.item.exists === false
        }
        Rectangle {
            id: usage
            visible: e.kind === "device" && e.item.mounted && e.item.usage >= 0
            anchors.left: eg.right
            anchors.leftMargin: 10
            anchors.right: ejectBtn.visible ? ejectBtn.left : parent.right
            anchors.rightMargin: 10
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 5
            height: 3
            radius: 1.5
            color: Theme.surfaceHighest
            Rectangle {
                width: parent.width * Math.max(0, Math.min(1, e.item.usage || 0))
                height: parent.height
                radius: 1.5
                color: (e.item.usage || 0) > 0.9 ? Theme.danger : Theme.primaryContainer
            }
        }
        MouseArea {
            id: ema
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
            preventStealing: true
            property real pressY: 0
            property bool dragging: false
            onPressed: mouse => {
                pressY = mouse.y
                dragging = false
                side.suppressClick = false
            }
            onPositionChanged: mouse => {
                if (!e.movable || !(pressedButtons & Qt.LeftButton))
                    return
                if (!dragging && Math.abs(mouse.y - pressY) > 6) {
                    dragging = true
                    side.beginReorder(e)
                }
                if (dragging)
                    side.updateReorder(mapToItem(side, mouse.x, mouse.y).y)
            }
            onReleased: {
                if (dragging) {
                    dragging = false
                    side.endReorder()
                }
            }
            onCanceled: {
                dragging = false
                side.reorder = null
            }
            onClicked: mouse => {
                if (side.suppressClick) {
                    side.suppressClick = false
                    return
                }
                if (mouse.button === Qt.RightButton) {
                    side.menuRequested(e.item, e.kind)
                } else if (e.kind === "device") {
                    side.openDevice(e.item, mouse.button === Qt.MiddleButton)
                } else if (mouse.button === Qt.MiddleButton) {
                    side.newTabRequested(e.target)
                } else {
                    side.navigateRequested(e.target)
                }
            }
        }
        GlyphButton {
            id: ejectBtn
            visible: (e.kind === "device" && e.item.mounted && e.item.removable) || (e.kind === "network" && !e.item.browse)
            anchors.right: parent.right
            anchors.rightMargin: 4
            anchors.verticalCenter: parent.verticalCenter
            implicitHeight: 24
            implicitWidth: 24
            padding: 2
            glyph: G.eject
            glyphSize: 14
            glyphColor: Theme.textDim
            tip: e.kind === "network" ? "Unmount" : "Eject"
            onClicked: Sys.unmount(e.kind === "network" ? e.item.mountPoint : e.item.device, e.kind === "device")
        }
        DropArea {
            id: drop
            anchors.fill: parent
            enabled: e.dropOk && e.target !== "" && e.target !== "trash:///" && e.target.indexOf("://") < 0
            onEntered: d => { if (!d.hasUrls) d.accepted = false }
            onDropped: d => side.dropRequested(d, e.target)
        }
    }

    ScrollView {
        anchors.fill: parent
        anchors.topMargin: 6
        clip: true
        contentWidth: availableWidth
        ScrollBar.vertical: TScrollBar { parent: side; x: side.width - width - 1; y: 6; height: side.height - 6 }
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

        Column {
            width: side.width - 12
            x: 6
            spacing: 1

            SectionHeader { title: "Computer"; key: "computer" }
            Repeater {
                id: placesRep
                model: side.collapsed.computer ? [] : side.places
                delegate: Entry { required property var modelData; required property int index; item: modelData; kind: "place"; idx: index }
            }

            SectionHeader { title: "Places"; key: "bookmarks"; visible: side.bookmarks.length > 0 }
            Repeater {
                id: bookmarksRep
                model: side.collapsed.bookmarks ? [] : side.bookmarks
                delegate: Entry { required property var modelData; required property int index; item: modelData; kind: "bookmark"; idx: index }
            }

            // Dropping a folder here bookmarks it, like Thunar's side pane.
            Rectangle {
                width: parent.width
                height: bmDrop.containsDrag ? 36 : 8
                radius: Theme.radius
                color: bmDrop.containsDrag ? Theme.hover : "transparent"
                border.width: bmDrop.containsDrag ? 1 : 0
                border.color: Theme.dropTarget
                TText {
                    anchors.centerIn: parent
                    visible: bmDrop.containsDrag
                    text: G.bookmark + "  Add bookmark"
                    color: Theme.textDim
                    font.pointSize: side.fontSize
                }
                DropArea {
                    id: bmDrop
                    anchors.fill: parent
                    onEntered: d => { if (!d.hasUrls) d.accepted = false }
                    onDropped: d => side.bookmarkDrop(d)
                }
            }

            SectionHeader { title: "Shares"; key: "shares"; visible: side.shares.length > 0 }
            Repeater {
                id: sharesRep
                model: side.collapsed.shares ? [] : side.shares
                delegate: Entry { required property var modelData; required property int index; item: modelData; kind: "share"; idx: index }
            }

            SectionHeader { title: "Devices"; key: "devices"; visible: side.devices.length > 0 }
            Repeater {
                model: side.collapsed.devices ? [] : side.devices
                delegate: Entry { required property var modelData; item: modelData; kind: "device" }
            }

            SectionHeader { title: "Network"; key: "network" }
            Repeater {
                model: side.collapsed.network ? [] : side.network
                delegate: Entry { required property var modelData; item: modelData; kind: "network" }
            }

        }
    }

    // Drop position marker and floating label while reordering.
    Rectangle {
        id: indicator
        visible: !!side.reorder
        x: 10
        width: side.width - 20
        height: 2
        color: Theme.primary
        z: 20
    }
    Rectangle {
        id: ghost
        property string text: ""
        property string glyph: ""
        visible: !!side.reorder
        x: 8
        width: side.width - 16
        height: side.grow(28)
        radius: Theme.radius
        color: Qt.alpha(Theme.selection, 0.9)
        border.color: Theme.primary
        z: 21
        Row {
            anchors.verticalCenter: parent.verticalCenter
            x: 10
            spacing: 10
            Glyph { text: ghost.glyph; size: 16; color: Theme.primary }
            TText { text: ghost.text; color: Theme.selectionText; font.pointSize: side.fontSize }
        }
    }

    Rectangle {
        anchors.right: parent.right
        width: 1
        height: parent.height
        color: Theme.divider
    }
}
