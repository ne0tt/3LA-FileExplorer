import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

// One tab: a folder model, its history, selection and the active view.
FocusScope {
    id: bp

    required property var prefs

    property string path: ""
    property var history: []
    property int historyIndex: -1
    // Per-folder view state: loaded from folders.json on navigation, falling
    // back to the Preferences defaults, and saved whenever it changes.
    property int viewMode: prefs.viewMode
    property int iconZoom: prefs.iconZoom
    property int listZoom: prefs.listZoom
    property int compactZoom: prefs.compactZoom
    property int sortKey: prefs.sortKey
    property bool sortDescending: prefs.sortDescending

    property var selection: ({})
    property int selectedCount: 0
    property int currentRow: -1
    property int anchorRow: -1
    property var dragUris: []
    property var cutSet: ({})
    property string pendingSelect: ""
    // Items a file operation just created; selected once a listing has them.
    property var pendingSelectPaths: []
    property string typeAhead: ""
    // The current-item outline is a keyboard cursor: shown only after keyboard
    // navigation, hidden again by any mouse press.
    property bool keyboardCursor: false

    readonly property alias model: folder
    readonly property bool isTrash: folder.isTrash
    readonly property bool singleClick: prefs.singleClick
    readonly property bool showThumbnails: prefs.showThumbnails
    readonly property bool listThumbnails: prefs.showThumbnails && prefs.listThumbnails
    // List views (detailed, compact) share the sidebar's text size; the icon
    // view follows the main font size.
    readonly property int listFontSize: prefs.sidebarFontSize > 0 ? prefs.sidebarFontSize : prefs.fontSize

    // Pictures and videos get thumbnails (when the settings allow).
    function canThumbnail(isImage, category) {
        return isImage || category === "video"
    }
    readonly property bool rowStripes: prefs.rowStripes
    readonly property var visibleColumns: parseOr(prefs.columns, [])
    readonly property var columnWidths: parseOr(prefs.columnWidths, {})
    readonly property bool canBack: historyIndex > 0
    readonly property bool canForward: historyIndex < history.length - 1
    readonly property string title: isTrash ? "Trash" : (path === "/" ? "File System" : Sys.basename(path))
    readonly property string glyph: isTrash ? G.trash : G.folder
    readonly property Item view: viewLoader.item
    property string statusText: ""

    signal openInNewTab(string path)
    signal openInNewWindow(string path)
    signal itemMenuRequested(int row)
    signal backgroundMenuRequested()
    signal columnsMenuRequested()
    signal message(string text)
    signal accessDenied(string path)
    signal authRequired(string uri, string host, string message)
    signal previewRequested(string path)
    // Encrypted items: .age files opened, or a locked folder shown.
    signal decryptRequested(var paths)
    signal lockedFolderOpened(string path)

    // Folder already prompted for, so a cancelled prompt isn't repeated on
    // every reload; navigating away and back asks again.
    property string promptedLocked: ""

    function checkLocked() {
        if (!folder.locked) {
            promptedLocked = ""
        } else if (visible && promptedLocked !== path) {
            promptedLocked = path
            lockedFolderOpened(path)
        }
    }

    // smb://host/share/... being mounted; the pane switches to the local gvfs
    // path once Sys reports it ready.
    property string pendingSmb: ""
    property bool pendingSmbHistory: true

    function parseOr(json, fallback) {
        try { return JSON.parse(json) } catch (e) { return fallback }
    }

    // --- navigation -------------------------------------------------------

    function navigate(p, addHistory) {
        // No stat here: a stale network mount can block for seconds. The
        // folder model loads off-thread and reports missing folders itself.
        p = Sys.normalize(p)
        if (p.startsWith("smb://") && Sys.smbHasShare(p)) {
            pendingSmb = p
            pendingSmbHistory = addHistory !== false
            Sys.smbOpen(p)
            return true
        }
        if (p === path) {
            folder.reload()
            return true
        }
        if (addHistory !== false) {
            const h = history.slice(0, historyIndex + 1)
            h.push(p)
            history = h
            historyIndex = h.length - 1
        }
        flushFolderView()
        loadFolderView(p)
        path = p
        pendingSelectPaths = []
        clearSelection()
        currentRow = -1
        anchorRow = -1
        folder.filter = ""
        folder.path = p
        return true
    }

    function back() {
        if (!canBack) return
        historyIndex -= 1
        pendingSelect = path
        navigate(history[historyIndex], false)
    }

    function forward() {
        if (!canForward) return
        historyIndex += 1
        navigate(history[historyIndex], false)
    }

    function up() {
        if (isTrash || path === "/" || path === "network:///") return
        const child = path
        if (navigate(Sys.parent(path)))
            pendingSelect = child
    }

    function reload() {
        folder.reload()
    }

    // --- selection --------------------------------------------------------

    function isSelected(p) {
        return selection[p] === true
    }

    function commit(sel) {
        selection = sel
        selectedCount = Object.keys(sel).length
        statusTimer.restart()
    }

    function clearSelection() {
        commit({})
    }

    function selectOnly(row) {
        const p = folder.pathAt(row)
        const s = {}
        if (p) s[p] = true
        currentRow = row
        anchorRow = row
        commit(s)
    }

    function toggleRow(row) {
        const p = folder.pathAt(row)
        const s = Object.assign({}, selection)
        if (s[p]) delete s[p]
        else s[p] = true
        currentRow = row
        anchorRow = row
        commit(s)
    }

    function selectRange(row, additive) {
        const a = anchorRow >= 0 ? anchorRow : row
        const s = additive ? Object.assign({}, selection) : {}
        for (let i = Math.min(a, row); i <= Math.max(a, row); ++i)
            s[folder.pathAt(i)] = true
        currentRow = row
        commit(s)
    }

    function setSelectionRows(rows, base) {
        const s = Object.assign({}, base)
        for (const r of rows)
            s[folder.pathAt(r)] = true
        commit(s)
    }

    function setSelectionPaths(paths) {
        const s = {}
        let first = -1
        for (const p of paths) {
            const r = folder.rowOf(p)
            if (r >= 0) {
                s[p] = true
                if (first < 0 || r < first) first = r
            }
        }
        commit(s)
        if (first >= 0) {
            currentRow = first
            anchorRow = first
            view?.positionAt(first)
        }
    }

    function selectAll() {
        const s = {}
        for (const p of folder.allPaths())
            s[p] = true
        commit(s)
    }

    function invertSelection() {
        const s = {}
        for (const p of folder.allPaths())
            if (!selection[p]) s[p] = true
        commit(s)
    }

    function selectPattern(pattern) {
        const s = {}
        for (const p of folder.pathsMatching(pattern))
            s[p] = true
        commit(s)
    }

    function selectedPaths() {
        // Keep view order for predictable operations.
        return folder.ordered(Object.keys(selection))
    }

    function selectedUris() {
        return selectedPaths().map(p => Sys.toUri(p))
    }

    // Coalesces the many commits of a rubber-band drag into one summary.
    Timer {
        id: statusTimer
        interval: 50
        onTriggered: bp.statusText = folder.summary(Object.keys(bp.selection))
    }

    function refreshCutSet() {
        const s = {}
        if (Sys.clipboardIsCut())
            for (const p of Sys.clipboardPaths())
                s[p] = true
        cutSet = s
    }

    // --- activation ---------------------------------------------------------

    function activateRow(row) {
        if (row < 0) return
        const sel = selectedPaths()
        if (sel.length > 1 && isSelected(folder.pathAt(row))) {
            activatePaths(sel)
            return
        }
        activatePaths([folder.pathAt(row)])
    }

    function activatePaths(paths) {
        if (isTrash) {
            message("Items in the Trash cannot be opened. Restore them first.")
            return
        }
        // The model already knows which rows are folders; only unlisted
        // paths need a stat.
        const isDir = p => { const r = folder.rowOf(p); return r >= 0 ? folder.isDirAt(r) : Sys.isDir(p) }
        const dirs = paths.filter(isDir)
        const sealed = paths.filter(p => !dirs.includes(p) && /\.age$/i.test(p))
        const files = paths.filter(p => !dirs.includes(p) && !sealed.includes(p))
        if (sealed.length) decryptRequested(sealed)
        if (files.length) {
            const err = Sys.openPaths(files)
            if (err) message(err)
        }
        if (dirs.length === 1 && !files.length) {
            navigate(dirs[0])
        } else {
            for (const d of dirs)
                openInNewTab(d)
        }
    }

    function loadFolderView(p) {
        const v = parseOr(Sys.folderView(p), {})
        viewMode = v.view ?? prefs.viewMode
        iconZoom = v.iconZoom ?? prefs.iconZoom
        listZoom = v.listZoom ?? prefs.listZoom
        compactZoom = v.compactZoom ?? prefs.compactZoom
        sortKey = v.sortKey ?? prefs.sortKey
        sortDescending = v.sortDescending ?? prefs.sortDescending
    }

    // Writes are deferred so a burst of zoom steps saves once; navigating
    // away or closing the tab flushes what is pending for the old folder.
    property string savePath: ""

    function saveFolderView() {
        if (!path) return
        savePath = path
        saveTimer.restart()
    }

    function writeFolderView() {
        Sys.setFolderView(savePath, JSON.stringify({
            view: viewMode, iconZoom: iconZoom, listZoom: listZoom, compactZoom: compactZoom,
            sortKey: sortKey, sortDescending: sortDescending
        }))
    }

    function flushFolderView() {
        if (!saveTimer.running) return
        saveTimer.stop()
        writeFolderView()
    }

    Timer {
        id: saveTimer
        interval: 500
        onTriggered: bp.writeFolderView()
    }

    Component.onDestruction: flushFolderView()

    function setViewMode(m) {
        viewMode = m
        saveFolderView()
    }

    function setSort(key, descending) {
        sortKey = key
        sortDescending = descending
        saveFolderView()
    }

    // Column-header click: same column flips direction, a new one starts ascending.
    function sortBy(key) {
        setSort(key, sortKey === key ? !sortDescending : false)
    }

    function setColumnWidth(key, w) {
        const m = Object.assign({}, columnWidths)
        m[key] = Math.round(w)
        prefs.columnWidths = JSON.stringify(m)
    }

    function zoomBy(delta) {
        if (viewMode === 0) iconZoom = Math.max(0, Math.min(6, iconZoom + delta))
        else if (viewMode === 1) listZoom = Math.max(0, Math.min(4, listZoom + delta))
        else compactZoom = Math.max(0, Math.min(4, compactZoom + delta))
        saveFolderView()
    }

    function zoomReset() {
        iconZoom = 2
        listZoom = 1
        compactZoom = 1
        zoomBy(0)
    }

    // --- drag & drop --------------------------------------------------------

    // Thunar semantics: move within one filesystem, copy across filesystems;
    // Ctrl forces copy and Ctrl+Shift links (the platform reports those as the
    // proposed action on our own drags).
    function handleDrop(drop, target) {
        if (!drop.hasUrls || isTrash)
            return
        const paths = drop.urls.map(u => Sys.fromUri(String(u))).filter(p => p !== target)
        if (!paths.length)
            return
        let action
        if (drop.proposedAction === Qt.LinkAction)
            action = Qt.LinkAction
        else if (drop.proposedAction === Qt.CopyAction && drop.source !== null)
            action = Qt.CopyAction
        else
            action = Sys.sameDevice(paths[0], target) ? Qt.MoveAction : Qt.CopyAction
        if (action === Qt.MoveAction && paths.every(p => Sys.parent(p) === target))
            return
        if (action === Qt.LinkAction) FileOps.link(paths, target)
        else if (action === Qt.CopyAction) FileOps.copy(paths, target)
        else FileOps.moveTo(paths, target)
        drop.accept(action)
    }

    // --- keyboard -----------------------------------------------------------

    function moveCursor(key, mods) {
        if (!view) return
        keyboardCursor = true
        const r = view.step(currentRow, key)
        if (r < 0) return
        if (mods & Qt.ShiftModifier)
            selectRange(r, mods & Qt.ControlModifier)
        else if (mods & Qt.ControlModifier)
            currentRow = r
        else
            selectOnly(r)
        view.positionAt(r)
    }

    Keys.onPressed: event => {
        const nav = [Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down, Qt.Key_PageUp, Qt.Key_PageDown, Qt.Key_Home, Qt.Key_End]
        const histKey = (event.key === Qt.Key_Left || event.key === Qt.Key_Right) && (event.modifiers & Qt.ControlModifier)
        if (nav.indexOf(event.key) >= 0 && !(event.modifiers & Qt.AltModifier) && !histKey) {
            moveCursor(event.key, event.modifiers)
            event.accepted = true
            return
        }
        if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && !(event.modifiers & Qt.AltModifier)) {
            const sel = selectedPaths()
            if (sel.length) activatePaths(sel)
            else if (currentRow >= 0) activateRow(currentRow)
            event.accepted = true
            return
        }
        // Space previews a selected picture or video (Ctrl+Space still toggles).
        if (event.key === Qt.Key_Space && !event.modifiers && selectedCount === 1 && !isTrash) {
            const p = selectedPaths()[0]
            const mime = folder.mimeAt(folder.rowOf(p))
            if (mime.startsWith("image/") || mime.startsWith("video/")) {
                previewRequested(p)
                event.accepted = true
                return
            }
        }
        if (event.key === Qt.Key_Space && currentRow >= 0 && (event.modifiers & Qt.ControlModifier)) {
            toggleRow(currentRow)
            event.accepted = true
            return
        }
        if (event.key === Qt.Key_Escape) {
            if (folder.filter) folder.filter = ""
            else clearSelection()
            event.accepted = true
            return
        }
        // Thunar-style type-ahead: jump to the first name starting with what was typed.
        if (event.text.length === 1 && event.text >= " " && !(event.modifiers & (Qt.ControlModifier | Qt.AltModifier))) {
            typeAhead += event.text
            keyboardCursor = true
            typeAheadTimer.restart()
            const r = folder.findPrefix(typeAhead, typeAhead.length === 1 ? currentRow + 1 : Math.max(0, currentRow))
            if (r >= 0) {
                selectOnly(r)
                view?.positionAt(r)
            }
            event.accepted = true
        }
    }

    Timer {
        id: typeAheadTimer
        interval: 900
        onTriggered: bp.typeAhead = ""
    }

    // --- model & view -------------------------------------------------------

    FolderModel {
        id: folder
        showHidden: bp.prefs.showHidden
        foldersFirst: bp.prefs.foldersFirst
        sortKey: bp.sortKey
        sortDescending: bp.sortDescending
        countChildren: bp.prefs.countChildren

        onLoaded: reset => {
            bp.checkLocked()
            // Listing a server's shares can need credentials too.
            if (folder.error.indexOf("Authentication required") === 0) {
                bp.authRequired(bp.path, Sys.basename(bp.path), folder.error)
                return
            }
            // Thunar/SPX: a folder we may not read gets a dialog, and we step back.
            if (folder.error.indexOf("Permission denied") >= 0) {
                const denied = bp.path
                bp.accessDenied(denied)
                if (bp.canBack) bp.back()
                return
            }
            if (bp.pendingSelectPaths.length) {
                const want = bp.pendingSelectPaths
                bp.setSelectionPaths(want)
                // A listing already in flight may predate the new items.
                if (want.every(p => folder.rowOf(p) >= 0)) bp.pendingSelectPaths = []
                return
            }
            if (bp.pendingSelect) {
                bp.setSelectionPaths([bp.pendingSelect])
                bp.pendingSelect = ""
            } else if (reset) {
                bp.view?.flick && (bp.view.flick.contentY = 0)
            }
            // Drop selections for items that vanished.
            const kept = folder.ordered(Object.keys(bp.selection))
            if (kept.length !== bp.selectedCount) {
                const s = {}
                for (const p of kept) s[p] = true
                bp.commit(s)
            } else {
                statusTimer.restart()
            }
        }
        onCountChanged: statusTimer.restart()
    }

    Connections {
        target: Sys
        function onClipboardSerialChanged() { bp.refreshCutSet() }
        function onSmbReady(uri, localPath) {
            if (uri !== bp.pendingSmb) return
            bp.pendingSmb = ""
            bp.navigate(localPath, bp.pendingSmbHistory)
        }
        function onSmbAuthRequired(uri, host, message) {
            if (uri !== bp.pendingSmb) return
            bp.pendingSmb = ""
            bp.authRequired(uri, host, message)
        }
        function onSmbError(uri, message) {
            if (uri !== bp.pendingSmb) return
            bp.pendingSmb = ""
            bp.message("Could not open " + uri + ": " + message)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.view
    }

    // Background tabs drop their view (delegates, thumbnails) and keep only
    // the model; the scroll position is restored when the tab returns.
    property point savedScroll: Qt.point(0, 0)

    onVisibleChanged: {
        if (!visible && view?.flick)
            savedScroll = Qt.point(view.flick.contentX, view.flick.contentY)
        viewLoader.active = visible
        checkLocked()
    }

    Loader {
        id: viewLoader
        anchors.fill: parent
        sourceComponent: bp.viewMode === 1 ? detailsComp : gridComp
        onLoaded: {
            const s = bp.savedScroll
            bp.savedScroll = Qt.point(0, 0)
            if (s.x || s.y)
                Qt.callLater(() => {
                    if (!item?.flick) return
                    item.flick.contentX = s.x
                    item.flick.contentY = s.y
                })
        }
    }

    Component {
        id: gridComp
        GridBrowser {
            pane: bp
            spacing: bp.prefs.gridSpacing
            compact: bp.viewMode === 2
            zoom: bp.viewMode === 2 ? bp.compactZoom : bp.iconZoom
        }
    }

    Component {
        id: detailsComp
        DetailsView {
            pane: bp
            zoom: bp.listZoom
        }
    }

    // Ctrl+wheel zooms the current view. Plain wheel events are left
    // unaccepted so they fall through to the view underneath and scroll it.
    MouseArea {
        property real zoomAccum: 0

        anchors.fill: parent
        acceptedButtons: Qt.NoButton
        onWheel: wheel => {
            if (!(wheel.modifiers & Qt.ControlModifier)) {
                wheel.accepted = false
                return
            }
            const dy = wheel.angleDelta.y
            if (dy * zoomAccum < 0)
                zoomAccum = 0
            zoomAccum += dy
            while (zoomAccum >= 120) {
                bp.zoomBy(1)
                zoomAccum -= 120
            }
            while (zoomAccum <= -120) {
                bp.zoomBy(-1)
                zoomAccum += 120
            }
            wheel.accepted = true
        }
    }

    // Drops onto empty space go into the current folder.
    DropArea {
        anchors.fill: parent
        z: -1
        enabled: !bp.isTrash
        onEntered: drag => { if (!drag.hasUrls) drag.accepted = false }
        onDropped: drop => bp.handleDrop(drop, bp.path)
    }

    Column {
        anchors.centerIn: parent
        spacing: 8
        visible: folder.count === 0 && !folder.loading && bp.pendingSmb === "" && !folder.locked
        Glyph {
            anchors.horizontalCenter: parent.horizontalCenter
            text: folder.error ? G.alert : (folder.filter ? G.filter : (bp.isTrash ? G.trashOutline : G.folderOpen))
            size: 56
            color: folder.error ? Theme.danger : Theme.outlineVariant
        }
        TText {
            anchors.horizontalCenter: parent.horizontalCenter
            text: folder.error ? folder.error
                : folder.filter ? "No items match \"" + folder.filter + "\""
                : bp.isTrash ? "Trash is empty" : "Folder is empty"
            color: folder.error ? Theme.danger : Theme.textFaint
        }
    }

    // Locked folder: the vault is all that's here until it is unlocked.
    Column {
        anchors.centerIn: parent
        spacing: 10
        visible: folder.locked && !folder.loading
        Glyph {
            anchors.horizontalCenter: parent.horizontalCenter
            text: G.lock
            size: 56
            color: Theme.primary
        }
        TText {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "This folder is encrypted"
            color: Theme.textDim
        }
        SpxButton {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Unlock…"
            onClicked: bp.lockedFolderOpened(bp.path)
        }
    }

    BusyIndicator {
        anchors.centerIn: parent
        running: (folder.loading && folder.count === 0) || bp.pendingSmb !== ""
        visible: running
    }

    TText {
        visible: bp.pendingSmb !== "" || (folder.loading && folder.count === 0 && folder.isVirtual)
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.verticalCenter
        anchors.topMargin: 40
        text: bp.pendingSmb !== "" ? "Connecting to " + bp.pendingSmb + "…"
            : bp.path === "network:///" ? "Searching the network…" : "Listing shares…"
        color: Theme.textFaint
    }

    // Abandons the wait; a mount still in flight is ignored when it finishes
    // (and gives up by itself after 40 s).
    SpxButton {
        visible: bp.pendingSmb !== ""
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.verticalCenter
        anchors.topMargin: 72
        text: "Cancel"
        onClicked: {
            bp.message("Stopped connecting to " + bp.pendingSmb)
            bp.pendingSmb = ""
        }
    }
}
