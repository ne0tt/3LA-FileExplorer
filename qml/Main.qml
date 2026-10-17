import QtCore
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

ApplicationWindow {
    id: win

    width: appPrefs.windowWidth
    height: appPrefs.windowHeight
    minimumWidth: 560
    minimumHeight: 360
    visible: true
    title: (pane ? pane.title : "") + " — 3LA File Explorer"
    color: Theme.window
    font.family: Theme.fontFamily
    font.pointSize: Theme.fontSize

    palette {
        window: Theme.window
        windowText: Theme.text
        base: Theme.surfaceLowest
        alternateBase: Theme.surfaceLow
        text: Theme.text
        button: Theme.surfaceHigh
        buttonText: Theme.text
        highlight: Theme.selection
        highlightedText: Theme.selectionText
        toolTipBase: Theme.surfaceHighest
        toolTipText: Theme.text
        placeholderText: Theme.textFaint
        mid: Theme.outlineVariant
        dark: Theme.surfaceLowest
        light: Theme.surfaceHighest
        accent: Theme.primaryContainer
        brightText: Theme.text
    }

    Settings {
        id: appPrefs
        location: Sys.settingsUrl()
        property int windowWidth: 1180
        property int windowHeight: 740
        property int sidebarWidth: 220
        property bool showSidebar: true
        property bool showStatusbar: true
        property bool showMenubar: true
        property bool showHidden: false
        property bool foldersFirst: true
        property int sortKey: 0
        property bool sortDescending: false
        property int viewMode: 0
        property int iconZoom: 2
        property int listZoom: 1
        property int compactZoom: 1
        property bool singleClick: false
        property bool showThumbnails: true
        property bool listThumbnails: true
        property int gridSpacing: 0
        property int fontSize: 10
        property int sidebarFontSize: 0     // 0 = same as fontSize
        property bool countChildren: true
        property bool rowStripes: false
        property bool alwaysShowTabs: false
        property bool confirmTrash: false
        property bool showDelete: true
        property bool milSpecBorders: true
        property string columns: "[\"size\",\"type\",\"modified\"]"
        property string columnWidths: "{}"
        property string dateFormat: "%d/%m/%Y %H:%M"
        property string placeOrder: "[]"

        onDateFormatChanged: win.applyDateFormat()
    }

    // Settings writes hit the disk; persist the size once a resize settles.
    onWidthChanged: if (visibility === Window.Windowed) sizeSaver.restart()
    onHeightChanged: if (visibility === Window.Windowed) sizeSaver.restart()
    Timer {
        id: sizeSaver
        interval: 500
        onTriggered: {
            appPrefs.windowWidth = win.width
            appPrefs.windowHeight = win.height
        }
    }

    // --- tabs ---------------------------------------------------------------

    property int currentTab: 0
    readonly property var pane: tabRepeater.count > currentTab ? tabRepeater.itemAt(currentTab) : null
    readonly property bool inTrash: !!pane && pane.isTrash
    // Network/server listings: nothing to paste into or create in.
    readonly property bool inVirtual: !!pane && pane.model.isVirtual
    readonly property bool readOnly: inTrash || inVirtual
    property bool filterVisible: false

    ListModel { id: tabsModel }

    function newTab(path, activate) {
        tabsModel.append({ initialPath: path })
        if (activate !== false)
            currentTab = tabsModel.count - 1
    }

    function closeTab(i) {
        if (tabsModel.count <= 1) {
            Qt.quit()
            return
        }
        tabsModel.remove(i)
        if (currentTab >= tabsModel.count)
            currentTab = tabsModel.count - 1
        else if (i < currentTab)
            currentTab -= 1
        focusPane()
    }

    function focusPane() {
        if (pane) pane.forceActiveFocus()
    }

    function busy() {
        return inputDlg.visible || compressDlg.visible || passwordDlg.visible || confirmDlg.visible || conflictDlg.visible || errorDlg.visible
            || symlinkDlg.visible || progressDlg.visible
            || Object.values(lazyDialogs).some(d => d.visible)
    }

    // Occasional dialogs are compiled and created on first use rather than at
    // startup (the previewer alone pulls in QtMultimedia). Reassigned, not
    // mutated, so bindings that call busy() see new entries.
    property var lazyDialogs: ({})

    function dialog(name, props) {
        let d = lazyDialogs[name]
        if (d) return d
        const c = Qt.createComponent(name + ".qml")
        if (c.status !== Component.Ready) {
            console.warn(c.errorString())
            return null
        }
        d = c.createObject(win.contentItem, props || {})
        d.closed.connect(win.focusPane)
        const all = Object.assign({}, lazyDialogs)
        all[name] = d
        lazyDialogs = all
        return d
    }

    function propertiesDialog() {
        return dialog("PropertiesDialog", {
            renameFn: (path, name) => {
                const e = FileOps.rename(path, name)
                win.reportError(e)
                return e
            },
            openWithFn: (paths, mime) => win.openWith(paths, mime)
        })
    }

    // Is `p` a folder? Answered from the model when it is listed (no stat on
    // the GUI thread, which can hang on a stale network mount).
    function isDirPath(p) {
        const r = pane ? pane.model.rowOf(p) : -1
        return r >= 0 ? pane.model.isDirAt(r) : Sys.isDir(p)
    }

    Component.onCompleted: {
        Sys.init()
        if (!Sys.setDateFormat(appPrefs.dateFormat))
            appPrefs.dateFormat = "%d/%m/%Y %H:%M"
        newTab(Sys.startPath)
    }


    // --- operations -----------------------------------------------------------

    function sel() {
        return pane ? pane.selectedPaths() : []
    }

    function applyDateFormat() {
        if (!Sys.setDateFormat(appPrefs.dateFormat))
            return
        for (let i = 0; i < tabRepeater.count; ++i)
            tabRepeater.itemAt(i).model.refreshDisplay()
    }

    function toast(msg) {
        toastItem.show(msg)
    }

    function copyToClipboard(cut) {
        const s = sel()
        if (!s.length) return
        Sys.setClipboard(s, cut)
        toast((cut ? "Cut " : "Copied ") + s.length + " item" + (s.length === 1 ? "" : "s"))
    }

    function paste(target) {
        const paths = Sys.clipboardPaths()
        if (!paths.length) {
            toast("Nothing to paste")
            return
        }
        const cut = Sys.clipboardIsCut()
        if (cut) {
            FileOps.moveTo(paths, target)
            Sys.setClipboard([], false)
        } else {
            FileOps.copy(paths, target)
        }
    }

    function trashSelected() {
        const s = sel()
        if (!s.length || pane.isTrash || pane.model.isVirtual) return
        const go = () => FileOps.trash(s)
        if (appPrefs.confirmTrash)
            confirmDlg.ask({
                header: "MOVE TO TRASH",
                tag: "RECOVERABLE",
                title: s.length === 1 ? "Move this item to Trash?" : "Move " + s.length + " items to Trash?",
                subject: s.length === 1 ? Sys.basename(s[0]) : "",
                message: "Items can be restored from Trash.",
                acceptText: "Move to Trash",
                danger: true,
                defaultAccept: false,
                onAccept: go
            })
        else
            go()
    }

    function deleteSelected() {
        const s = sel()
        if (!s.length) return
        const names = s.slice(0, 10).map(p => "  • " + Sys.basename(p)).join("\n") + (s.length > 10 ? "\n  …" : "")
        confirmDlg.ask({
            header: "PERMANENT DELETE",
            tag: "BURN AFTER READING",
            title: s.length === 1 ? "Permanently delete this item?" : "Permanently delete " + s.length + " items?",
            subject: s.length === 1 ? Sys.basename(s[0]) : "",
            message: "This action cannot be undone.",
            detail: s.length > 1 ? names : "",
            acceptText: "Delete Permanently",
            danger: true,
            defaultAccept: false,
            onAccept: () => pane.isTrash ? pane.model.purgeFromTrash(s) : FileOps.remove(s)
        })
    }

    function restoreSelected() {
        const s = sel()
        if (!s.length || !pane.isTrash || pane.model.isVirtual) return
        const items = s.map(p => {
            const orig = pane.model.origPathOf(p)
            return { path: p, name: Sys.basename(orig || p), folder: orig ? Sys.parent(orig) : "", taken: !!orig && Sys.exists(orig) }
        })
        dialog("RestoreDialog").ask(items, () => pane.model.restoreFromTrash(s))
    }

    // Places and Shares share one bookmarks file; step past entries that
    // belong to the other section.
    function moveBookmarkWithinSection(item, step) {
        const all = JSON.parse(Sys.bookmarksJson)
        const i = all.findIndex(x => x.path === item.path)
        const section = all.filter(x => !!x.remote === !!item.remote)
        const k = section.findIndex(x => x.path === item.path)
        const neighbour = section[k + step]
        if (i < 0 || !neighbour) return
        Sys.moveBookmark(i, all.indexOf(neighbour))
    }

    function reportError(e) {
        if (e) toast(e)
    }

    Binding { target: Theme; property: "fontSize"; value: appPrefs.fontSize }
    Binding { target: Theme; property: "milSpec"; value: appPrefs.milSpecBorders }
    Binding { target: Theme; property: "menuFontSize"; value: appPrefs.sidebarFontSize > 0 ? appPrefs.sidebarFontSize : appPrefs.fontSize }

    Connections {
        target: Sys
        function onLaunchFailed(message) { win.reportError(message) }
    }

    Connections {
        target: win.pane ? win.pane.model : null
        function onOperationFailed(message) { win.reportError(message) }
    }

    function renameSelected() {
        const s = sel()
        if (s.length !== 1 || pane.isTrash || pane.model.isVirtual) return
        const p = s[0]
        inputDlg.ask({
            header: "RENAME",
            title: "Rename",
            label: Sys.basename(p),
            placeholder: "New name",
            text: Sys.basename(p),
            acceptText: "Rename",
            selectStem: !isDirPath(p),
            onAccept: name => {
                const err = FileOps.rename(p, name)
                if (!err) pane.pendingSelect = Sys.join(Sys.parent(p), name)
                return err
            }
        })
    }

    function createFolder() {
        if (!pane || pane.isTrash || pane.model.isVirtual) return
        const dir = pane.path
        inputDlg.ask({
            header: "NEW FOLDER",
            title: "New Folder",
            label: "in " + dir,
            placeholder: "Folder name",
            text: Sys.suggestName(dir, "New Folder"),
            acceptText: "Create",
            onAccept: name => {
                const r = FileOps.createFolder(dir, name)
                if (r.startsWith("!")) return r.substring(1)
                pane.pendingSelect = r
                return ""
            }
        })
    }

    function createFile(template) {
        if (!pane || pane.isTrash || pane.model.isVirtual) return
        const dir = pane.path
        const base = template ? template.fileName : "New Empty File"
        inputDlg.ask({
            header: "NEW FILE",
            title: template ? "New " + template.name : "New File",
            label: "in " + dir,
            placeholder: "File name",
            text: Sys.suggestName(dir, base),
            acceptText: "Create",
            selectStem: true,
            onAccept: name => {
                const r = FileOps.createFile(dir, name, template ? template.path : "")
                if (r.startsWith("!")) return r.substring(1)
                pane.pendingSelect = r
                return ""
            }
        })
    }

    function makeLinks() {
        const s = sel()
        if (s.length) FileOps.link(s, pane.path)
    }

    function duplicate() {
        const s = sel()
        if (s.length) FileOps.copy(s, pane.path)
    }

    function openWith(paths, mime) {
        if (!paths.length) return
        dialog("OpenWithDialog").ask(paths, mime)
    }

    function showProperties(paths) {
        if (!paths.length) return
        propertiesDialog().show(paths)
    }

    function selectByPattern() {
        inputDlg.ask({
            header: "SELECT",
            tag: "FILTER",
            title: "Select by Pattern",
            label: "Wildcards * and ? are supported.",
            text: "*",
            acceptText: "Select",
            onAccept: pat => { pane.selectPattern(pat); return "" }
        })
    }

    function sendTo(target) {
        const s = sel()
        if (s.length) FileOps.copy(s, target)
    }

    function linkToDesktop() {
        const s = sel()
        const desk = Sys.join(Sys.homePath, "Desktop")
        if (s.length) FileOps.link(s, desk)
    }

    function setView(m) {
        if (pane) pane.setViewMode(m)
    }

    // Decrypted files to open once their job finishes.
    property var openAfterDecrypt: []

    function encryptPaths(paths) {
        passwordDlg.ask({
            encrypting: true,
            title: paths.length === 1 ? "Encrypt" : "Encrypt " + paths.length + " items",
            subject: paths.length === 1 ? Sys.basename(paths[0]) : "",
            onAccept: pw => FileOps.encrypt(paths, pw)
        })
    }

    function decryptPaths(paths, openAfter, error) {
        passwordDlg.ask({
            encrypting: false,
            error: error || "",
            title: paths.length === 1 ? "Unlock" : "Unlock " + paths.length + " items",
            subject: paths.length === 1 ? Sys.basename(paths[0]) : "",
            onAccept: pw => {
                win.openAfterDecrypt = openAfter ? paths : []
                FileOps.decrypt(paths, pw)
            }
        })
    }

    function goTo(path) {
        if (pane && pane.navigate(path))
            focusPane()
    }

    Connections {
        target: FileOps
        function onErrorOccurred(message) {
            errorDlg.ask({ header: "ERROR", tag: "ATTENTION", danger: true, cancelText: "", title: "Operation incomplete", message: "Some items could not be processed:", detail: message, acceptText: "Close" })
        }
        function onPasswordRejected(paths) {
            const reopen = win.openAfterDecrypt.length > 0
            win.decryptPaths(paths, reopen, "Wrong password, try again")
        }
        function onFinished(created) {
            if (FileOps.kind === "decrypt" && win.openAfterDecrypt.length) {
                const files = created.filter(p => !win.isDirPath(p))
                win.openAfterDecrypt = []
                if (files.length) win.reportError(Sys.openPaths(files))
            }
            if (win.pane && created.length && created.every(p => Sys.parent(p) === win.pane.path))
                win.pane.pendingSelectPaths = created
            // Show the outcome now rather than after the folder watcher's
            // settle delay (e.g. Empty Trash, deletes).
            for (let i = 0; i < tabRepeater.count; ++i) {
                const t = tabRepeater.itemAt(i)
                if (t === win.pane || t.isTrash) t.reload()
            }
        }
    }

    onActiveChanged: if (active && pane) pane.refreshCutSet()

    // Forget memory-only share credentials (and their mounts) on any exit path.
    Connections {
        target: Qt.application
        function onAboutToQuit() { Sys.shutdown() }
    }

    // Mouse side buttons (button 8/9). Child MouseAreas don't accept these
    // buttons, so the presses fall through to this window-wide handler.
    TapHandler {
        acceptedButtons: Qt.BackButton | Qt.ForwardButton
        gesturePolicy: TapHandler.WithinBounds
        onTapped: (point, button) => {
            if (!win.pane || win.busy()) return
            if (button === Qt.BackButton) win.pane.back()
            else win.pane.forward()
        }
    }

    // --- actions & shortcuts -------------------------------------------------

    // A checkable action whose state mirrors `isOn`. Triggering toggles
    // `checked` and so breaks a plain `checked:` binding; this restores it.
    component ChoiceAction: Action {
        property bool isOn: false
        checkable: true
        checked: isOn
        onTriggered: checked = Qt.binding(() => isOn)
    }


    Action { id: actNewTab; text: "New Tab"; icon.name: G.tabPlus; shortcut: "Ctrl+T"; onTriggered: win.newTab(win.pane ? win.pane.path : Sys.homePath) }
    Action { id: actNewWindow; text: "New Window"; icon.name: G.openNew; shortcut: "Ctrl+N"; onTriggered: Sys.newWindow(win.pane ? win.pane.path : Sys.homePath) }
    Action { id: actNewFolder; text: "Create Folder…"; icon.name: G.folderPlus; shortcut: "Ctrl+Shift+N"; enabled: !!win.pane && !win.readOnly; onTriggered: if (!win.busy()) win.createFolder() }
    Action { id: actNewFile; text: "Empty File…"; icon.name: G.filePlus; enabled: !!win.pane && !win.readOnly; onTriggered: win.createFile(null) }
    Action { id: actTerminal; text: "Open Terminal Here"; icon.name: G.terminal; shortcut: "F4"; enabled: !!win.pane && !win.readOnly; onTriggered: win.reportError(Sys.openTerminal(win.pane.path)) }
    Action { id: actProperties; text: "Properties…"; icon.name: G.info; shortcut: "Alt+Return"; onTriggered: { const s = win.sel(); win.showProperties(s.length ? s : (win.inTrash ? [] : [win.pane.path])) } }
    Action { id: actCloseTab; text: "Close Tab"; icon.name: G.close; shortcut: "Ctrl+W"; onTriggered: win.closeTab(win.currentTab) }
    Action { id: actQuit; text: "Close Window"; shortcut: "Ctrl+Q"; onTriggered: Qt.quit() }

    Action { id: actUndo; text: FileOps.canUndo ? FileOps.undoText : "Undo"; icon.name: G.undo; shortcut: "Ctrl+Z"; enabled: FileOps.canUndo && !FileOps.busy; onTriggered: win.reportError(FileOps.undo()) }
    Action { id: actCut; text: "Cut"; icon.name: G.cut; shortcut: "Ctrl+X"; enabled: !!win.pane && win.pane.selectedCount > 0 && !win.readOnly; onTriggered: win.copyToClipboard(true) }
    Action { id: actCopy; text: "Copy"; icon.name: G.copy; shortcut: "Ctrl+C"; enabled: !!win.pane && win.pane.selectedCount > 0; onTriggered: win.copyToClipboard(false) }
    Action { id: actPaste; text: "Paste"; icon.name: G.paste; shortcut: "Ctrl+V"; enabled: !!win.pane && !win.readOnly; onTriggered: win.paste(win.pane.path) }
    Action { id: actPasteInto; text: "Paste Into Folder"; icon.name: G.paste; enabled: !!win.pane && win.pane.selectedCount === 1 && win.isDirPath(Object.keys(win.pane.selection)[0]); onTriggered: win.paste(win.sel()[0]) }
    Action { id: actTrash; text: "Move to Trash"; icon.name: G.trash; shortcut: "Delete"; enabled: !!win.pane && win.pane.selectedCount > 0 && !win.readOnly; onTriggered: if (!win.busy()) win.trashSelected() }
    Action { id: actDelete; text: "Delete"; icon.name: G.del; shortcut: "Shift+Delete"; enabled: !!win.pane && win.pane.selectedCount > 0; onTriggered: if (!win.busy()) win.deleteSelected() }
    Action { id: actRename; text: "Rename…"; icon.name: G.rename; shortcut: "F2"; enabled: !!win.pane && win.pane.selectedCount === 1 && !win.readOnly; onTriggered: if (!win.busy()) win.renameSelected() }
    Action { id: actDuplicate; text: "Duplicate"; icon.name: G.copy; enabled: !!win.pane && win.pane.selectedCount > 0 && !win.readOnly; onTriggered: win.duplicate() }
    Action { id: actMakeLink; text: "Make Link"; icon.name: G.link; enabled: !!win.pane && win.pane.selectedCount > 0 && !win.readOnly; onTriggered: win.makeLinks() }
    Action { id: actSelectAll; text: "Select All"; icon.name: G.selectAll; shortcut: "Ctrl+A"; onTriggered: win.pane.selectAll() }
    Action { id: actSelectPattern; text: "Select by Pattern…"; shortcut: "Ctrl+S"; onTriggered: if (!win.busy()) win.selectByPattern() }
    Action { id: actInvert; text: "Invert Selection"; shortcut: "Ctrl+Shift+I"; onTriggered: win.pane.invertSelection() }
    Action { id: actPrefs; text: "Preferences…"; icon.name: G.cog; onTriggered: win.dialog("PreferencesDialog", { prefs: appPrefs }).open() }

    Action { id: actReload; text: "Reload"; icon.name: G.reload; shortcut: "Ctrl+R"; onTriggered: win.pane.reload() }
    Action { id: actLocation; text: "Open Location…"; icon.name: G.pencil; shortcut: "Ctrl+L"; onTriggered: pathBar.startEditing() }
    ChoiceAction { id: actFilter; text: "Filter…"; icon.name: G.filter; shortcut: "Ctrl+F"; isOn: win.filterVisible
        onTriggered: {
            win.filterVisible = !win.filterVisible
            if (win.filterVisible) filterField.forceActiveFocus()
            else { win.pane.model.filter = ""; win.focusPane() }
        }
    }
    ChoiceAction { id: actSidebar; text: "Side Pane"; shortcut: "F9"; isOn: appPrefs.showSidebar; onTriggered: appPrefs.showSidebar = !appPrefs.showSidebar }
    ChoiceAction { id: actStatusbar; text: "Statusbar"; isOn: appPrefs.showStatusbar; onTriggered: appPrefs.showStatusbar = !appPrefs.showStatusbar }
    ChoiceAction { id: actMenubar; text: "Menubar"; shortcut: "Ctrl+M"; isOn: appPrefs.showMenubar; onTriggered: appPrefs.showMenubar = !appPrefs.showMenubar }
    ChoiceAction { id: actHidden; text: "Show Hidden Files"; shortcut: "Ctrl+H"; isOn: appPrefs.showHidden; onTriggered: appPrefs.showHidden = !appPrefs.showHidden }
    Action { id: actZoomIn; text: "Zoom In"; shortcut: "Ctrl+="; onTriggered: win.pane.zoomBy(1) }
    Action { id: actZoomOut; text: "Zoom Out"; shortcut: "Ctrl+-"; onTriggered: win.pane.zoomBy(-1) }
    Action { id: actZoomReset; text: "Normal Size"; shortcut: "Ctrl+0"; onTriggered: win.pane.zoomReset() }
    ChoiceAction { id: actViewIcons; text: "Icon View"; icon.name: G.grid; shortcut: "Ctrl+1"; isOn: !!win.pane && win.pane.viewMode === 0; onTriggered: win.setView(0) }
    ChoiceAction { id: actViewList; text: "Detailed List View"; icon.name: G.list; shortcut: "Ctrl+2"; isOn: !!win.pane && win.pane.viewMode === 1; onTriggered: win.setView(1) }
    ChoiceAction { id: actViewCompact; text: "Compact List View"; icon.name: G.compact; shortcut: "Ctrl+3"; isOn: !!win.pane && win.pane.viewMode === 2; onTriggered: win.setView(2) }
    ChoiceAction { id: actSortName; text: "By Name"; isOn: !!win.pane && win.pane.sortKey === 0; onTriggered: win.pane.setSort(0, win.pane.sortDescending) }
    ChoiceAction { id: actSortSize; text: "By Size"; isOn: !!win.pane && win.pane.sortKey === 1; onTriggered: win.pane.setSort(1, win.pane.sortDescending) }
    ChoiceAction { id: actSortType; text: "By Type"; isOn: !!win.pane && win.pane.sortKey === 2; onTriggered: win.pane.setSort(2, win.pane.sortDescending) }
    ChoiceAction { id: actSortDate; text: "By Modification Date"; isOn: !!win.pane && win.pane.sortKey === 3; onTriggered: win.pane.setSort(3, win.pane.sortDescending) }
    ChoiceAction { id: actSortAsc; text: "Ascending"; isOn: !!win.pane && !win.pane.sortDescending; onTriggered: win.pane.setSort(win.pane.sortKey, false) }
    ChoiceAction { id: actSortDesc; text: "Descending"; isOn: !!win.pane && win.pane.sortDescending; onTriggered: win.pane.setSort(win.pane.sortKey, true) }
    ChoiceAction { id: actFoldersFirst; text: "Folders First"; isOn: appPrefs.foldersFirst; onTriggered: appPrefs.foldersFirst = !appPrefs.foldersFirst }

    Action { id: actUp; text: "Open Parent"; icon.name: G.up; shortcut: "Alt+Up"; enabled: !!win.pane && win.pane.path !== "/" && !win.inTrash; onTriggered: win.pane.up() }
    Action { id: actBack; text: "Back"; icon.name: G.back; enabled: !!win.pane && win.pane.canBack; onTriggered: win.pane.back() }
    Action { id: actForward; text: "Forward"; icon.name: G.forward; enabled: !!win.pane && win.pane.canForward; onTriggered: win.pane.forward() }
    Action { id: actHome; text: "Home"; icon.name: G.home; shortcut: "Alt+Home"; onTriggered: win.goTo(Sys.homePath) }
    Action { id: actDesktop; text: "Desktop"; icon.name: "\u{F01C4}"; onTriggered: win.goTo(Sys.join(Sys.homePath, "Desktop")) }
    Action { id: actGoTrash; text: "Trash"; icon.name: G.trash; onTriggered: win.goTo("trash:///") }
    Action { id: actGoRoot; text: "File System"; icon.name: G.harddisk; onTriggered: win.goTo("/") }
    Action { id: actGoTemplates; text: "Templates"; icon.name: "\u{F069D}"; onTriggered: win.goTo(Sys.join(Sys.homePath, "Templates")) }

    Action { id: actAddBookmark; text: "Add Bookmark"; icon.name: G.bookmark; shortcut: "Ctrl+D"; enabled: !!win.pane && !win.inTrash
        onTriggered: {
            const s = win.sel().filter(p => win.isDirPath(p))
            const targets = s.length ? s : [win.pane.path]
            for (const p of targets) Sys.addBookmark(p)
            win.toast("Bookmarked " + targets.map(p => Sys.basename(p)).join(", "))
        }
    }
    Action { id: actShortcuts; text: "Keyboard Shortcuts"; icon.name: G.keyboard; shortcut: "F1"; onTriggered: win.dialog("AboutDialog").open() }

    Action { id: actEmptyTrash; text: "Empty Trash"; icon.name: G.del
        onTriggered: confirmDlg.ask({
            header: "EMPTY TRASH", tag: "BURN AFTER READING", danger: true, defaultAccept: false,
            title: "Empty the Trash?", acceptText: "Empty Trash",
            message: "All items in the Trash will be permanently deleted. This action cannot be undone.",
            onAccept: () => FileOps.emptyTrash()
        })
    }
    Action { id: actRestore; text: "Restore"; icon.name: G.restore; enabled: win.inTrash && win.pane.selectedCount > 0; onTriggered: win.restoreSelected() }

    Shortcut { sequences: ["F5"]; onActivated: win.pane.reload() }
    Shortcut { sequences: ["Ctrl++"]; onActivated: win.pane.zoomBy(1) }
    Shortcut { sequences: ["Backspace"]; enabled: !win.busy(); onActivated: win.pane.back() }
    Shortcut { sequences: ["Ctrl+Left"]; enabled: !!win.pane && !win.busy(); onActivated: win.pane.back() }
    Shortcut { sequences: ["Ctrl+Right"]; enabled: !!win.pane && !win.busy(); onActivated: win.pane.forward() }
    Shortcut { sequences: ["Ctrl+PgDown", "Ctrl+Tab"]; onActivated: win.currentTab = (win.currentTab + 1) % tabsModel.count }
    Shortcut { sequences: ["Ctrl+PgUp", "Ctrl+Shift+Tab"]; onActivated: win.currentTab = (win.currentTab + tabsModel.count - 1) % tabsModel.count }
    Instantiator {
        model: 9
        delegate: Shortcut {
            required property int index
            sequence: "Alt+" + (index + 1)
            onActivated: if (index < tabsModel.count) win.currentTab = index
        }
    }

    // --- menubar ---------------------------------------------------------------

    menuBar: MenuBar {
        id: mbar
        visible: appPrefs.showMenubar
        height: visible ? implicitHeight : 0
        font.family: Theme.fontFamily
        font.pointSize: Theme.menuFontSize

        background: Rectangle { color: Theme.chrome }
        delegate: MenuBarItem {
            id: mbi
            padding: 6
            leftPadding: 10
            rightPadding: 10
            contentItem: TText {
                text: mbi.text.replace("&", "")
                color: mbi.highlighted ? Theme.selectionText : Theme.text
                font.pointSize: Theme.menuFontSize
            }
            background: Rectangle {
                radius: Theme.smallRadius
                color: mbi.highlighted ? Theme.selection : "transparent"
            }
        }

        TMenu {
            title: "&File"
            TMenuItem { action: actNewTab }
            TMenuItem { action: actNewWindow }
            TMenuSeparator {}
            TMenuItem { action: actNewFolder }
            TemplatesMenu { title: "Create Document"; icon.name: G.filePlus; onCreate: t => win.createFile(t) }
            TMenuItem { action: actTerminal }
            TMenuSeparator {}
            TMenuItem { action: actProperties }
            TMenuSeparator {}
            TMenuItem { action: actCloseTab }
            TMenuItem { action: actQuit }
            TMenuSeparator { visible: win.inTrash }
            TMenuItem { action: actEmptyTrash; visible: win.inTrash }
        }
        TMenu {
            title: "&Edit"
            TMenuItem { action: actUndo }
            TMenuSeparator {}
            TMenuItem { action: actCut }
            TMenuItem { action: actCopy }
            TMenuItem { action: actPaste }
            TMenuItem { action: actPasteInto }
            TMenuSeparator {}
            TMenuItem { action: actRename }
            TMenuItem { action: actDuplicate }
            TMenuItem { action: actMakeLink }
            TMenuItem { action: actRestore; visible: win.inTrash }
            TMenuSeparator {}
            TMenuItem { action: actSelectAll }
            TMenuItem { action: actSelectPattern }
            TMenuItem { action: actInvert }
            TMenuSeparator {}
            TMenuItem { action: actPrefs }
            TMenuSeparator {}
            TMenuItem { action: actTrash }
            TMenuItem { action: actDelete; visible: appPrefs.showDelete }
        }
        TMenu {
            title: "&View"
            TMenuItem { action: actReload }
            TMenuItem { action: actFilter }
            TMenuSeparator {}
            TMenuItem { action: actSidebar }
            TMenuItem { action: actStatusbar }
            TMenuItem { action: actMenubar }
            TMenuSeparator {}
            TMenuItem { action: actHidden }
            TMenuSeparator {}
            TMenu {
                title: "Arrange Items"
                icon.name: G.sort
                TMenuItem { action: actSortName }
                TMenuItem { action: actSortSize }
                TMenuItem { action: actSortType }
                TMenuItem { action: actSortDate }
                TMenuSeparator {}
                TMenuItem { action: actSortAsc }
                TMenuItem { action: actSortDesc }
                TMenuSeparator {}
                TMenuItem { action: actFoldersFirst }
            }
            ColumnsMenu { title: "Visible Columns"; icon.name: G.list; prefs: appPrefs }
            TMenuItem { action: actZoomIn }
            TMenuItem { action: actZoomOut }
            TMenuItem { action: actZoomReset }
            TMenuSeparator {}
            TMenuItem { action: actViewIcons }
            TMenuItem { action: actViewList }
            TMenuItem { action: actViewCompact }
        }
        TMenu {
            title: "&Go"
            TMenuItem { action: actUp }
            TMenuItem { action: actBack }
            TMenuItem { action: actForward }
            TMenuSeparator {}
            TMenuItem { action: actHome }
            TMenuItem { action: actDesktop }
            TMenuItem { action: actGoTrash }
            TMenuItem { action: actGoRoot }
            TMenuItem { action: actGoTemplates }
            TMenuSeparator {}
            TMenuItem { action: actLocation }
        }
        TMenu {
            id: bookmarksMenu
            title: "&Bookmarks"
            TMenuItem { action: actAddBookmark }
            TMenuSeparator {}
            Instantiator {
                model: JSON.parse(Sys.bookmarksJson || "[]")
                delegate: TMenuItem {
                    required property var modelData
                    text: modelData.name
                    icon.name: modelData.glyph
                    onTriggered: win.goTo(modelData.path)
                }
                onObjectAdded: (i, o) => bookmarksMenu.insertItem(i + 2, o)
                onObjectRemoved: (i, o) => bookmarksMenu.removeItem(o)
            }
        }
        TMenu {
            title: "&Help"
            TMenuItem { action: actShortcuts }
        }
    }

    // --- toolbar -----------------------------------------------------------------

    header: Rectangle {
        color: Theme.chrome
        height: Theme.grow(44)

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 6
            anchors.rightMargin: 6
            spacing: 2

            GlyphButton {
                id: menuButton
                visible: !appPrefs.showMenubar
                glyph: G.dots
                tip: "Menu"
                onClicked: hamburger.popup(menuButton, 0, menuButton.height)
            }
            GlyphButton { action: actBack; glyph: G.back; tip: "Back (Ctrl+Left)"; text: "" }
            GlyphButton { action: actForward; glyph: G.forward; tip: "Forward (Ctrl+Right)"; text: "" }
            GlyphButton { action: actUp; glyph: G.up; tip: "Parent folder (Alt+Up)"; text: "" }
            GlyphButton { action: actHome; glyph: G.home; tip: "Home (Alt+Home)"; text: "" }

            PathBar {
                id: pathBar
                Layout.fillWidth: true
                Layout.leftMargin: 6
                Layout.rightMargin: 6
                path: win.pane ? win.pane.path : ""
                onNavigateRequested: p => win.goTo(p)
                onNewTabRequested: p => win.newTab(p)
                onDropRequested: (d, p) => win.pane.handleDrop(d, p)
                onEditFinished: win.focusPane()
            }

            TField {
                id: filterField
                visible: win.filterVisible
                Layout.preferredWidth: 220
                placeholderText: G.filter + "  Filter this folder"
                text: win.pane ? win.pane.model.filter : ""
                onTextEdited: win.pane.model.filter = text
                Keys.onEscapePressed: actFilter.trigger()
                Keys.onReturnPressed: win.focusPane()
                Keys.onDownPressed: win.focusPane()
            }

            GlyphButton { action: actReload; glyph: G.reload; tip: "Reload (Ctrl+R)"; text: "" }
            GlyphButton { glyph: G.filter; tip: "Filter (Ctrl+F)"; checked: win.filterVisible; onClicked: actFilter.trigger() }
            Rectangle { width: 1; height: 20; color: Theme.dividerOnChrome; Layout.leftMargin: 4; Layout.rightMargin: 4 }
            GlyphButton { glyph: G.grid; tip: "Icon view (Ctrl+1)"; checked: !!win.pane && win.pane.viewMode === 0; onClicked: win.setView(0) }
            GlyphButton { glyph: G.list; tip: "Detailed list (Ctrl+2)"; checked: !!win.pane && win.pane.viewMode === 1; onClicked: win.setView(1) }
            GlyphButton { glyph: G.compact; tip: "Compact list (Ctrl+3)"; checked: !!win.pane && win.pane.viewMode === 2; onClicked: win.setView(2) }
            Rectangle { width: 1; height: 20; color: Theme.dividerOnChrome; Layout.leftMargin: 4; Layout.rightMargin: 4 }
            GlyphButton { glyph: G.menu; tip: (appPrefs.showMenubar ? "Hide" : "Show") + " menu bar (Ctrl+M)"; checked: appPrefs.showMenubar; onClicked: actMenubar.trigger() }
        }

        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: 1
            color: Theme.divider
        }
    }

    TMenu {
        id: hamburger
        TMenuItem { action: actNewTab }
        TMenuItem { action: actNewWindow }
        TMenuItem { action: actNewFolder }
        TMenuSeparator {}
        TMenuItem { action: actHidden }
        TMenuItem { action: actSidebar }
        TMenuItem { action: actStatusbar }
        TMenuItem { action: actMenubar }
        TMenuSeparator {}
        TMenuItem { action: actAddBookmark }
        TMenuItem { action: actPrefs }
        TMenuItem { action: actShortcuts }
        TMenuSeparator {}
        TMenuItem { action: actQuit }
    }

    // --- body --------------------------------------------------------------------

    SplitView {
        id: split
        anchors.fill: parent
        orientation: Qt.Horizontal

        handle: Rectangle {
            implicitWidth: 4
            color: SplitHandle.pressed || SplitHandle.hovered ? Theme.dividerOnChrome : "transparent"
        }

        Sidebar {
            id: sidebar
            visible: appPrefs.showSidebar
            fontSize: appPrefs.sidebarFontSize > 0 ? appPrefs.sidebarFontSize : appPrefs.fontSize
            SplitView.preferredWidth: appPrefs.sidebarWidth
            SplitView.minimumWidth: 140
            SplitView.maximumWidth: 480
            // Saved once dragging the split handle stops, not on every pixel.
            onWidthChanged: if (visible && width > 0) sidebarSaver.restart()
            Timer {
                id: sidebarSaver
                interval: 500
                onTriggered: appPrefs.sidebarWidth = sidebar.width
            }
            currentPath: win.pane ? win.pane.path : ""
            onNavigateRequested: p => win.goTo(p)
            onNewTabRequested: p => win.newTab(p)
            onNewWindowRequested: p => Sys.newWindow(p)
            onMenuRequested: (item, kind) => sideMenu.show(item, kind)
            onDropRequested: (d, p) => win.pane.handleDrop(d, p)
            placeOrder: { try { return JSON.parse(appPrefs.placeOrder) } catch (e) { return [] } }
            onPlaceOrderEdited: order => appPrefs.placeOrder = JSON.stringify(order)
        }

        ColumnLayout {
            SplitView.fillWidth: true
            spacing: 0

            // Tab strip, shown once there is more than one tab (Thunar default).
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? Theme.grow(34) : 0
                visible: tabsModel.count > 1 || appPrefs.alwaysShowTabs
                color: Theme.chrome

                ListView {
                    id: tabStrip
                    anchors.fill: parent
                    anchors.leftMargin: 4
                    anchors.rightMargin: 40
                    orientation: ListView.Horizontal
                    model: tabsModel
                    clip: true
                    spacing: 2
                    boundsBehavior: Flickable.StopAtBounds
                    currentIndex: win.currentTab
                    delegate: Rectangle {
                        id: tabItem
                        required property int index
                        readonly property var tpane: tabRepeater.count > index ? tabRepeater.itemAt(index) : null
                        readonly property bool current: index === win.currentTab
                        width: Math.min(240, Math.max(120, (tabStrip.width - 4) / tabsModel.count))
                        height: tabStrip.height - 4
                        y: 4
                        radius: Theme.radius
                        color: current ? Theme.view : tma.containsMouse ? Theme.hover : "transparent"
                        Rectangle {
                            visible: tabItem.current
                            anchors.bottom: parent.bottom
                            width: parent.width
                            height: parent.radius
                            color: Theme.view
                        }
                        Rectangle {
                            visible: tabItem.current
                            anchors.top: parent.top
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: parent.width - 16
                            height: 2
                            radius: 1
                            color: Theme.primaryContainer
                        }
                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: 10
                            anchors.right: closeBtn.left
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 8
                            Glyph {
                                text: tabItem.tpane ? tabItem.tpane.glyph : G.folder
                                color: Theme.primaryContainer
                                size: 14
                            }
                            TText {
                                width: parent.width - 30
                                text: tabItem.tpane ? tabItem.tpane.title : ""
                                elide: Text.ElideRight
                                color: tabItem.current ? Theme.text : Theme.textDim
                            }
                        }
                        MouseArea {
                            id: tma
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                            onClicked: mouse => {
                                if (mouse.button === Qt.MiddleButton) win.closeTab(tabItem.index)
                                else { win.currentTab = tabItem.index; win.focusPane() }
                            }
                        }
                        DropArea {
                            anchors.fill: parent
                            onEntered: win.currentTab = tabItem.index
                        }
                        GlyphButton {
                            id: closeBtn
                            anchors.right: parent.right
                            anchors.rightMargin: 4
                            anchors.verticalCenter: parent.verticalCenter
                            implicitWidth: 22
                            implicitHeight: 22
                            padding: 2
                            glyph: G.close
                            glyphSize: 12
                            glyphColor: Theme.textDim
                            onClicked: win.closeTab(tabItem.index)
                        }
                    }
                }
                GlyphButton {
                    anchors.right: parent.right
                    anchors.rightMargin: 6
                    anchors.verticalCenter: parent.verticalCenter
                    glyph: G.plus
                    tip: "New tab (Ctrl+T)"
                    onClicked: actNewTab.trigger()
                }
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: win.currentTab

                Repeater {
                    id: tabRepeater
                    model: tabsModel
                    delegate: BrowserPane {
                        required property string initialPath
                        prefs: appPrefs
                        Component.onCompleted: {
                            navigate(initialPath)
                            refreshCutSet()
                            forceActiveFocus()
                        }
                        onOpenInNewTab: p => win.newTab(p, false)
                        onOpenInNewWindow: p => Sys.newWindow(p)
                        onItemMenuRequested: row => itemMenu.show()
                        onBackgroundMenuRequested: bgMenu.show()
                        onColumnsMenuRequested: columnsPopup.popup()
                        onMessage: text => win.toast(text)
                        onAccessDenied: p => win.dialog("AccessDeniedDialog").show(p, true)
                        onPreviewRequested: p => win.dialog("PreviewDialog").show(win.pane, p)
                        onDecryptRequested: paths => win.decryptPaths(paths, true)
                        onLockedFolderOpened: p => win.decryptPaths([p], false)
                        onAuthRequired: (uri, host, message) => win.dialog("NetworkAuthDialog").ask(uri, host, message, () => navigate(uri))
                    }
                }
            }

            // Status bar.
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? Theme.grow(26) : 0
                visible: appPrefs.showStatusbar
                color: Theme.chrome
                Rectangle { width: parent.width; height: 1; color: Theme.divider }
                TText {
                    anchors.left: parent.left
                    anchors.leftMargin: 12
                    anchors.right: statusRight.left
                    anchors.verticalCenter: parent.verticalCenter
                    text: win.pane ? win.pane.statusText : ""
                    color: Theme.textDim
                    font.pointSize: Theme.smallFontSize
                    elide: Text.ElideRight
                }
                Row {
                    id: statusRight
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 12
                    TText {
                        visible: !!win.pane && win.pane.model.filter.length > 0
                        text: G.filter + " " + (win.pane ? win.pane.model.filter : "")
                        color: Theme.primary
                        font.pointSize: Theme.smallFontSize
                    }
                    TText {
                        visible: appPrefs.showHidden
                        text: G.eye + " hidden"
                        color: Theme.textFaint
                        font.pointSize: Theme.smallFontSize
                    }
                    TText {
                        visible: FileOps.busy
                        text: Math.round(FileOps.progress * 100) + "%"
                        color: Theme.primary
                        font.pointSize: Theme.smallFontSize
                    }
                }
            }
        }
    }

    // --- context menus -----------------------------------------------------------

    TMenu {
        id: itemMenu

        property var paths: []
        property bool single: paths.length === 1
        property bool allDirs: false
        property string mime: ""
        property var apps: []
        // Archive name stems; non-empty only when every selected item is an archive.
        property var stems: []
        // Every selected item is encrypted (.age file or locked folder) / none is.
        property bool allSealed: false
        property bool noneSealed: false

        function show() {
            paths = win.sel()
            if (!paths.length) return
            allDirs = paths.every(p => win.isDirPath(p))
            // The Send To list comes from the device poll, which idles while
            // the side pane is hidden.
            Sys.refreshDevices()
            mime = win.pane.model.mimeAt(win.pane.model.rowOf(paths[0]))
            apps = allDirs || win.inTrash ? [] : JSON.parse(Sys.appsFor(mime))
            const st = paths.map(p => win.isDirPath(p) ? "" : FileOps.archiveStem(p))
            stems = st.every(s => s.length > 0) ? st : []
            const sealed = paths.map(p => win.isDirPath(p) ? FileOps.isLocked(p) : /\.age$/i.test(p))
            allSealed = sealed.every(x => x)
            noneSealed = sealed.every(x => !x)
            popup()
        }

        TMenuItem {
            text: itemMenu.allDirs ? "Open" : (itemMenu.apps.length ? "Open With \"" + itemMenu.apps[0].name + "\"" : "Open")
            icon.name: G.folderOpen
            visible: !win.inTrash
            onTriggered: win.pane.activatePaths(itemMenu.paths)
        }
        TMenuItem {
            text: "Open in New Tab"
            icon.name: G.tabPlus
            visible: itemMenu.allDirs && !win.inTrash
            onTriggered: { for (const p of itemMenu.paths) win.newTab(p, false) }
        }
        TMenuItem {
            text: "Open in New Window"
            icon.name: G.openNew
            visible: itemMenu.allDirs && !win.inTrash
            onTriggered: { for (const p of itemMenu.paths) Sys.newWindow(p) }
        }
        TMenuItem {
            text: "Open Terminal Here"
            icon.name: G.terminal
            visible: itemMenu.single && itemMenu.allDirs && !win.readOnly
            onTriggered: win.reportError(Sys.openTerminal(itemMenu.paths[0]))
        }
        TMenuItem {
            text: "Open With…"
            icon.name: G.app
            enabled: !itemMenu.allDirs && !win.readOnly
            onTriggered: win.openWith(itemMenu.paths, itemMenu.mime)
        }
        TMenuSeparator {}
        TMenu {
            id: sendToMenu
            title: "Send To"
            icon.name: "\u{F048A}"
            enabled: !win.readOnly
            TMenuItem { text: "Desktop (Create Link)"; icon.name: "\u{F01C4}"; onTriggered: win.linkToDesktop() }
            Instantiator {
                model: JSON.parse(Sys.devicesJson || "[]").filter(d => d.mounted && d.removable)
                delegate: TMenuItem {
                    required property var modelData
                    text: modelData.name
                    icon.name: modelData.glyph
                    onTriggered: win.sendTo(modelData.mountPoint)
                }
                onObjectAdded: (i, o) => sendToMenu.insertItem(i + 1, o)
                onObjectRemoved: (i, o) => sendToMenu.removeItem(o)
            }
        }
        TMenuSeparator {}
        TMenuItem {
            text: "Extract Here"
            icon.name: G.extract
            visible: itemMenu.stems.length > 0 && !win.readOnly
            onTriggered: FileOps.extractHere(itemMenu.paths)
        }
        TMenuItem {
            text: itemMenu.stems.length === 1 ? "Extract to \"" + itemMenu.stems[0] + "/\"" : "Extract to Folders"
            icon.name: G.extractFolder
            visible: itemMenu.stems.length > 0 && !win.readOnly
            onTriggered: FileOps.extractToFolder(itemMenu.paths)
        }
        TMenuItem {
            text: "Compress…"
            icon.name: G.compress
            visible: !win.readOnly
            onTriggered: compressDlg.ask(itemMenu.paths, win.pane.path)
        }
        TMenuItem {
            text: "Encrypt…"
            icon.name: G.lock
            visible: itemMenu.noneSealed && !win.readOnly
            onTriggered: win.encryptPaths(itemMenu.paths)
        }
        TMenuItem {
            text: "Decrypt…"
            icon.name: G.unlock
            visible: itemMenu.allSealed && !win.readOnly
            onTriggered: win.decryptPaths(itemMenu.paths, false)
        }
        TMenuSeparator { visible: !win.readOnly }
        TMenuItem { action: actCut }
        TMenuItem { action: actCopy }
        TMenuItem { action: actPasteInto; visible: itemMenu.single && itemMenu.allDirs && !win.readOnly }
        TMenuSeparator {}
        TMenuItem { action: actRestore; visible: win.inTrash }
        TMenuItem { action: actDuplicate; visible: !win.readOnly }
        TMenuItem { action: actMakeLink; visible: !win.readOnly }
        TMenuItem { action: actRename; visible: !win.readOnly }
        TMenuSeparator {}
        TMenuItem {
            text: "Add Bookmark"
            icon.name: G.bookmark
            visible: itemMenu.allDirs && !win.inTrash
            onTriggered: { for (const p of itemMenu.paths) Sys.addBookmark(p) }
        }
        TMenuItem {
            text: "Copy Path"
            icon.name: G.copy
            onTriggered: { clipHelper.text = itemMenu.paths.join("\n"); clipHelper.selectAll(); clipHelper.copy() }
        }
        TMenuItem {
            text: "Properties…"
            icon.name: G.info
            visible: !win.readOnly
            onTriggered: win.showProperties(itemMenu.paths)
        }
        TMenuSeparator {}
        TMenuItem { action: actTrash; visible: !win.readOnly }
        TMenuItem { action: actDelete; visible: appPrefs.showDelete || (win.inTrash) }
    }

    TMenu {
        id: bgMenu

        function show() {
            popup()
        }

        TMenuItem { action: actNewFolder; visible: !win.readOnly }
        TemplatesMenu { title: "Create Document"; icon.name: G.filePlus; enabled: !win.readOnly; onCreate: t => win.createFile(t) }
        TMenuItem { action: actTerminal; visible: !win.readOnly }
        TMenuSeparator {}
        TMenuItem { action: actPaste; visible: !win.readOnly }
        TMenuSeparator {}
        TMenuItem { action: actSelectAll }
        TMenuItem { action: actSelectPattern }
        TMenuItem { action: actInvert }
        TMenuSeparator {}
        TMenu {
            title: "Arrange Items"
            icon.name: G.sort
            TMenuItem { action: actSortName }
            TMenuItem { action: actSortSize }
            TMenuItem { action: actSortType }
            TMenuItem { action: actSortDate }
            TMenuSeparator {}
            TMenuItem { action: actSortAsc }
            TMenuItem { action: actSortDesc }
            TMenuSeparator {}
            TMenuItem { action: actFoldersFirst }
        }
        TMenu {
            title: "View As"
            icon.name: G.grid
            TMenuItem { action: actViewIcons }
            TMenuItem { action: actViewList }
            TMenuItem { action: actViewCompact }
        }
        TMenuItem { action: actHidden }
        TMenuItem { action: actAddBookmark; visible: !win.readOnly }
        TMenuSeparator {}
        TMenuItem {
            text: "Encrypt This Folder…"
            icon.name: G.lock
            visible: !win.readOnly && !!win.pane && !win.pane.model.locked
            onTriggered: win.encryptPaths([win.pane.path])
        }
        TMenuItem {
            text: "Unlock This Folder…"
            icon.name: G.unlock
            visible: !win.readOnly && !!win.pane && win.pane.model.locked
            onTriggered: win.decryptPaths([win.pane.path], false)
        }
        TMenuItem { action: actProperties; visible: !win.readOnly }
        TMenuSeparator { visible: win.inTrash }
        TMenuItem { action: actEmptyTrash; visible: win.inTrash }
    }

    TMenu {
        id: sideMenu

        property var item: ({})
        property string kind: ""
        readonly property string target: kind === "device" || kind === "network" ? (item.mountPoint || "") : (item.path || "")

        function show(it, k) {
            item = it
            kind = k
            popup()
        }

        TMenuItem { text: "Open"; icon.name: G.folderOpen; enabled: sideMenu.target !== "" || sideMenu.kind === "device"
            onTriggered: sideMenu.kind === "device" ? sidebar.openDevice(sideMenu.item, false) : win.goTo(sideMenu.target) }
        TMenuItem { text: "Open in New Tab"; icon.name: G.tabPlus; enabled: sideMenu.target !== "" || sideMenu.kind === "device"
            onTriggered: sideMenu.kind === "device" ? sidebar.openDevice(sideMenu.item, true) : win.newTab(sideMenu.target) }
        TMenuItem { text: "Open in New Window"; icon.name: G.openNew; enabled: sideMenu.target !== ""; onTriggered: Sys.newWindow(sideMenu.target) }
        TMenuSeparator {}
        TMenuItem { text: "Mount"; icon.name: G.usb; visible: sideMenu.kind === "device" && !sideMenu.item.mounted; onTriggered: Sys.mount(sideMenu.item.device) }
        TMenuItem { text: "Unmount"; icon.name: G.eject; visible: (sideMenu.kind === "device" && sideMenu.item.mounted) || (sideMenu.kind === "network" && !sideMenu.item.browse)
            onTriggered: Sys.unmount(sideMenu.kind === "network" ? sideMenu.item.mountPoint : sideMenu.item.device, false) }
        TMenuItem { text: "Eject"; icon.name: G.eject; visible: sideMenu.kind === "device" && !!sideMenu.item.removable
            onTriggered: Sys.unmount(sideMenu.item.device, true) }
        TMenuItem { text: "Rename Bookmark…"; icon.name: G.rename; visible: (sideMenu.kind === "bookmark" || sideMenu.kind === "share")
            onTriggered: inputDlg.ask({
                header: "RENAME", title: "Rename Place", label: sideMenu.item.path, placeholder: "Place name", text: sideMenu.item.name, acceptText: "Save",
                onAccept: name => { Sys.renameBookmark(sideMenu.item.path, name); return "" }
            })
        }
        TMenuItem { text: "Move Up"; icon.name: G.chevronUp; visible: (sideMenu.kind === "bookmark" || sideMenu.kind === "share")
            onTriggered: win.moveBookmarkWithinSection(sideMenu.item, -1) }
        TMenuItem { text: "Move Down"; icon.name: G.chevronDown; visible: (sideMenu.kind === "bookmark" || sideMenu.kind === "share")
            onTriggered: win.moveBookmarkWithinSection(sideMenu.item, 1) }
        TMenuItem { text: "Remove Bookmark"; icon.name: G.del; visible: (sideMenu.kind === "bookmark" || sideMenu.kind === "share"); onTriggered: Sys.removeBookmark(sideMenu.item.path) }
        TMenuSeparator {}
        TMenuItem { text: "Properties…"; icon.name: G.info; enabled: sideMenu.target !== "" && sideMenu.target.indexOf("://") < 0; onTriggered: win.showProperties([sideMenu.target]) }
        TMenuSeparator { visible: sideMenu.item.trash === true }
        TMenuItem { action: actEmptyTrash; visible: sideMenu.item.trash === true }
    }

    ColumnsMenu {
        id: columnsPopup
        prefs: appPrefs
    }

    // Hidden helper for "Copy Path" (plain-text clipboard).
    TextEdit { id: clipHelper; visible: false }

    // --- dialogs & overlays ------------------------------------------------------

    InputDialog { id: inputDlg; onClosed: win.focusPane() }
    CompressDialog { id: compressDlg; onClosed: win.focusPane() }
    PasswordDialog { id: passwordDlg; onClosed: win.focusPane() }
    ConfirmDialog { id: confirmDlg; onClosed: win.focusPane() }
    ConfirmDialog { id: errorDlg }
    ConflictDialog { id: conflictDlg }
    SymlinkDialog { id: symlinkDlg }
    ProgressDialog { id: progressDlg }

    // In the overlay, so a toast raised while a modal dialog is open shows
    // above its scrim.
    Toast {
        id: toastItem
        parent: Overlay.overlay
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 40
        z: 1000
    }
}
