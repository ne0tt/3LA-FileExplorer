import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// SPX "FILE INFO": general details plus permission editing.
SpxDialog {
    id: d

    property var paths: []
    property var info: ({})
    property string sizeText: ""
    property int mode: 0
    property int page: 0
    property var renameFn: null
    property var openWithFn: null

    readonly property bool multi: paths.length > 1

    // Details are gathered off the GUI thread (stat, child count, gio); the
    // dialog opens at once and fills in when infoReady arrives.
    function show(paths) {
        d.paths = paths
        info = ({ name: Sys.basename(paths[0]), parent: Sys.parent(paths[0]) })
        mode = 0
        nameField.text = multi ? paths.length + " items" : info.name
        sizeText = "Calculating…"
        if (multi)
            Sys.computeSize(paths)
        page = 0
        recursive.checked = false
        permError.text = ""
        Sys.requestInfo(paths[0])
        open()
    }

    function bit(b) { return (mode & b) !== 0 }
    function setBit(b, on) { mode = on ? (mode | b) : (mode & ~b) }
    function applyMode() {
        permError.text = ""
        Sys.applyMode(paths, mode, recursive.checked)
    }

    Connections {
        target: Sys
        enabled: d.visible
        function onSizeComputed(path, text) {
            if (path === d.paths.join("\n")) d.sizeText = text
        }
        function onInfoReady(path, json) {
            if (path !== d.paths[0]) return
            const first = d.info.mode === undefined
            d.info = JSON.parse(json)
            if (!first) return
            d.mode = d.info.mode || 0
            if (!d.multi && d.info.isDir) Sys.computeSize(d.paths)
            else if (!d.multi) d.sizeText = d.info.sizeText || ""
        }
        function onModeApplied(errors) {
            permError.text = errors
            if (!d.multi) Sys.requestInfo(d.paths[0])
        }
    }

    header: "FILE INFO"
    tag: "EYES ONLY"
    cardWidth: 540
    buttonRow: footer

    component InfoRow: RowLayout {
        property string label
        property string value
        Layout.fillWidth: true
        spacing: 12
        SpxLabel { text: parent.label; Layout.preferredWidth: 96; Layout.alignment: Qt.AlignTop; topPadding: 2 }
        SpxText { text: parent.value; color: Theme.text; wrapMode: Text.WrapAnywhere }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 12
        Glyph {
            text: d.multi ? G.file : (d.info.glyph || G.file)
            size: 34
            color: Theme.categoryColor(d.info.category)
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            SpxField {
                id: nameField
                Layout.fillWidth: true
                readOnly: d.multi
                Keys.onReturnPressed: {
                    if (!d.multi && text !== d.info.name && d.renameFn) {
                        const err = d.renameFn(d.info.path, text)
                        if (!err) d.show([Sys.join(d.info.parent, text)])
                    }
                }
            }
            SpxLabel { text: d.multi ? "multiple selection" : (d.info.typeText || "") + " · " + (d.info.mime || "") }
        }
    }

    RowLayout {
        spacing: 8
        SpxButton { text: "General"; selected: d.page === 0; onClicked: d.page = 0; implicitHeight: 24 }
        SpxButton { text: "Permissions"; selected: d.page === 1; onClicked: d.page = 1; implicitHeight: 24 }
    }

    ColumnLayout {
        visible: d.page === 0
        Layout.fillWidth: true
        spacing: 6
        InfoRow { label: "Location"; value: d.info.parent || "" }
        InfoRow { label: "Size"; value: d.sizeText }
        InfoRow { label: "Link target"; value: d.info.linkTarget || ""; visible: !!d.info.isLink }
        InfoRow { label: "Modified"; value: d.info.modified || ""; visible: !d.multi }
        InfoRow { label: "Accessed"; value: d.info.accessed || ""; visible: !d.multi }
        InfoRow { label: "Changed"; value: d.info.changed || ""; visible: !d.multi }
        RowLayout {
            visible: !d.multi && !d.info.isDir
            Layout.fillWidth: true
            spacing: 12
            SpxLabel { text: "Opens with"; Layout.preferredWidth: 96 }
            SpxText { text: d.info.defaultAppName || "None"; color: Theme.text }
            SpxButton { text: "Change"; implicitHeight: 24; onClicked: if (d.openWithFn) d.openWithFn(d.paths, d.info.mime) }
        }
        InfoRow { label: "Free space"; value: d.info.freeSpace || "" }
        Rectangle {
            Layout.fillWidth: true
            Layout.leftMargin: 108
            height: 4
            color: Theme.border
            Rectangle {
                width: parent.width * (d.info.usage || 0)
                height: parent.height
                color: (d.info.usage || 0) > 0.9 ? Theme.danger : Theme.primary
            }
        }
    }

    GridLayout {
        visible: d.page === 1
        Layout.fillWidth: true
        columns: 4
        columnSpacing: 14
        rowSpacing: 6

        SpxLabel { text: "Owner" }
        SpxText { text: d.info.owner || ""; color: Theme.text; Layout.columnSpan: 3 }
        SpxLabel { text: "Group" }
        SpxText { text: d.info.group || ""; color: Theme.text; Layout.columnSpan: 3 }

        Item { width: 1; height: 1 }
        SpxLabel { text: "Read" }
        SpxLabel { text: "Write" }
        SpxLabel { text: d.info.isDir ? "Enter" : "Execute" }

        SpxLabel { text: "Owner" }
        SpxCheck { checked: d.bit(0o400); onToggled: d.setBit(0o400, checked); enabled: !!d.info.isOwner }
        SpxCheck { checked: d.bit(0o200); onToggled: d.setBit(0o200, checked); enabled: !!d.info.isOwner }
        SpxCheck { checked: d.bit(0o100); onToggled: d.setBit(0o100, checked); enabled: !!d.info.isOwner }
        SpxLabel { text: "Group" }
        SpxCheck { checked: d.bit(0o040); onToggled: d.setBit(0o040, checked); enabled: !!d.info.isOwner }
        SpxCheck { checked: d.bit(0o020); onToggled: d.setBit(0o020, checked); enabled: !!d.info.isOwner }
        SpxCheck { checked: d.bit(0o010); onToggled: d.setBit(0o010, checked); enabled: !!d.info.isOwner }
        SpxLabel { text: "Others" }
        SpxCheck { checked: d.bit(0o004); onToggled: d.setBit(0o004, checked); enabled: !!d.info.isOwner }
        SpxCheck { checked: d.bit(0o002); onToggled: d.setBit(0o002, checked); enabled: !!d.info.isOwner }
        SpxCheck { checked: d.bit(0o001); onToggled: d.setBit(0o001, checked); enabled: !!d.info.isOwner }

        SpxText {
            Layout.columnSpan: 4
            text: "Mode " + ("000" + d.mode.toString(8)).slice(-4) + "   " + (d.info.permissions || "")
            color: Theme.textFaint
        }
        SpxCheck { id: recursive; Layout.columnSpan: 4; visible: !!d.info.isDir; text: "Apply recursively to contents" }
        SpxText {
            Layout.columnSpan: 4
            visible: !d.info.isOwner
            text: "You are not the owner, so you cannot change these permissions."
            color: Theme.textFaint
        }
        SpxText { id: permError; Layout.columnSpan: 4; color: Theme.danger; visible: text.length > 0 }
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Apply"; visible: d.page === 1; enabled: !!d.info.isOwner; onClicked: d.applyMode() }
        SpxButton { text: "Close"; onClicked: d.close() }
    }
}
