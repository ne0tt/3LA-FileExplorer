import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

// Thunar's location bar: clickable path buttons that remember deeper crumbs
// after going up, or (Ctrl+L) an editable entry with folder completion.
Item {
    id: bar

    property string path: ""
    property string trail: ""
    property bool editing: false

    signal navigateRequested(string path)
    signal newTabRequested(string path)
    signal dropRequested(var drop, string path)
    signal editFinished()

    implicitHeight: Theme.grow(32)

    onPathChanged: {
        if (path.indexOf("://") >= 0 || !(trail === path || trail.startsWith(path === "/" ? "/" : path + "/")))
            trail = path
        crumbsFlick.contentX = Math.max(0, crumbsFlick.contentWidth - crumbsFlick.width)
    }

    readonly property var crumbs: {
        if (path === "trash:///")
            return [{ name: "Trash", path: "trash:///", glyph: G.trash }]
        const net = { name: "Network", path: "network:///", glyph: G.lan }
        const server = h => ({ name: h, path: "smb://" + h + "/", glyph: G.network })
        if (path === "network:///")
            return [net]
        if (path.startsWith("smb://"))
            return [net, server(Sys.basename(path))]
        const t = trail || path
        // Inside a gvfs SMB mount: Network > server > share > subfolders.
        const info = JSON.parse(Sys.smbInfo(t))
        if (info.uri) {
            const parts = info.uri.substring(6).split("/")
            const out = [net, server(parts[0]), { name: parts[1], path: info.root, glyph: "\u{F0870}" }]
            let acc = info.root
            for (const p of t.substring(info.root.length).split("/").filter(s => s.length)) {
                acc += "/" + p
                out.push({ name: p, path: acc, glyph: "" })
            }
            return out
        }
        const parts = t.split("/").filter(s => s.length)
        const out = [{ name: "", path: "/", glyph: G.harddisk }]
        let acc = ""
        const home = Sys.homePath
        for (const p of parts) {
            acc += "/" + p
            out.push({ name: p, path: acc, glyph: acc === home ? G.home : "" })
        }
        return out
    }

    function startEditing(text) {
        editing = true
        field.text = text !== undefined ? text : Sys.displayPath(path)
        field.forceActiveFocus()
        if (text === undefined)
            field.selectAll()
    }

    function stopEditing() {
        editing = false
        completion.close()
        editFinished()
    }

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius
        color: Theme.surfaceLowest
        border.width: 1
        border.color: bar.editing ? Theme.focus : Theme.border
    }

    Flickable {
        id: crumbsFlick
        visible: !bar.editing
        anchors.fill: parent
        anchors.margins: 2
        anchors.rightMargin: 34
        contentWidth: crumbRow.width
        contentHeight: height
        clip: true
        flickableDirection: Flickable.HorizontalFlick
        boundsBehavior: Flickable.StopAtBounds
        onWidthChanged: contentX = Math.max(0, contentWidth - width)

        Row {
            id: crumbRow
            height: parent.height
            spacing: 0
            onWidthChanged: crumbsFlick.contentX = Math.max(0, width - crumbsFlick.width)

            Repeater {
                model: bar.crumbs
                delegate: Row {
                    id: crumb
                    required property var modelData
                    required property int index
                    readonly property bool current: modelData.path === bar.path
                    height: crumbRow.height

                    Glyph {
                        visible: crumb.index > 1
                        anchors.verticalCenter: parent.verticalCenter
                        text: G.chevronRight
                        size: 14
                        color: Theme.textFaint
                    }
                    GlyphButton {
                        anchors.verticalCenter: parent.verticalCenter
                        implicitHeight: Theme.grow(26)
                        glyph: crumb.modelData.glyph
                        text: crumb.modelData.name
                        glyphColor: Theme.primaryContainer
                        font.bold: crumb.current
                        onClicked: bar.navigateRequested(crumb.modelData.path)
                        opacity: bar.path.length && !crumb.current && crumb.modelData.path.length > bar.path.length ? 0.6 : 1
                        MouseArea {
                            anchors.fill: parent
                            acceptedButtons: Qt.MiddleButton
                            onClicked: bar.newTabRequested(crumb.modelData.path)
                        }
                        DropArea {
                            anchors.fill: parent
                            enabled: crumb.modelData.path !== "trash:///"
                            onEntered: d => { if (!d.hasUrls) d.accepted = false }
                            onDropped: d => bar.dropRequested(d, crumb.modelData.path)
                        }
                    }
                }
            }
        }

        MouseArea {
            x: crumbRow.width
            width: Math.max(0, crumbsFlick.width - crumbRow.width)
            height: parent.height
            onClicked: bar.startEditing()
        }
    }

    GlyphButton {
        visible: !bar.editing
        anchors.right: parent.right
        anchors.rightMargin: 2
        anchors.verticalCenter: parent.verticalCenter
        implicitHeight: Theme.grow(28)
        glyph: G.pencil
        glyphSize: 14
        glyphColor: Theme.textDim
        tip: "Edit location (Ctrl+L)"
        onClicked: bar.startEditing()
    }

    TextField {
        id: field
        visible: bar.editing
        anchors.fill: parent
        leftPadding: 10
        color: Theme.text
        selectionColor: Theme.selection
        selectedTextColor: Theme.selectionText
        font.family: Theme.fontFamily
        font.pointSize: Theme.fontSize
        background: Item {}
        selectByMouse: true

        onActiveFocusChanged: if (!activeFocus && !completion.opened) bar.stopEditing()
        // Completion reads a directory; wait for a pause in typing.
        onTextEdited: completeTimer.restart()

        function complete() {
            completeTimer.stop()
            if (!bar.editing) return
            const opts = Sys.complete(text)
            completion.items = opts
            completion.index = -1
            if (opts.length && !(opts.length === 1 && opts[0] === text)) completion.open()
            else completion.close()
        }

        Timer {
            id: completeTimer
            interval: 120
            onTriggered: field.complete()
        }
        Keys.onPressed: event => {
            if (event.key === Qt.Key_Escape) {
                bar.stopEditing()
                event.accepted = true
            } else if (event.key === Qt.Key_Tab) {
                if (completion.items.length) {
                    text = completion.items[Math.max(0, completion.index)]
                    complete()
                }
                event.accepted = true
            } else if (event.key === Qt.Key_Down && completion.opened) {
                completion.index = Math.min(completion.items.length - 1, completion.index + 1)
                event.accepted = true
            } else if (event.key === Qt.Key_Up && completion.opened) {
                completion.index = Math.max(-1, completion.index - 1)
                event.accepted = true
            } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                const t = completion.opened && completion.index >= 0 ? completion.items[completion.index] : text
                bar.stopEditing()
                bar.navigateRequested(t)
                event.accepted = true
            }
        }
    }

    Popup {
        id: completion
        property var items: []
        property int index: -1
        y: bar.height + 2
        width: bar.width
        height: Math.min(260, compList.contentHeight + 8)
        padding: 4
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent
        // Focus may have left the field while this was open; end editing then.
        onClosed: if (!field.activeFocus) bar.stopEditing()
        background: Rectangle {
            color: Theme.surfaceContainer
            border.color: Theme.border
            radius: Theme.radius
        }
        ListView {
            id: compList
            anchors.fill: parent
            clip: true
            model: completion.items
            currentIndex: completion.index
            delegate: Rectangle {
                id: cd
                required property string modelData
                required property int index
                width: compList.width
                height: Theme.grow(26)
                radius: Theme.smallRadius
                color: cd.index === completion.index ? Theme.selection : cma.containsMouse ? Theme.hover : "transparent"
                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    x: 8
                    spacing: 8
                    Glyph { text: G.folder; color: Theme.primaryContainer; size: 14 }
                    TText { text: cd.modelData; color: cd.index === completion.index ? Theme.selectionText : Theme.text }
                }
                MouseArea {
                    id: cma
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        field.text = cd.modelData
                        field.forceActiveFocus()
                        field.complete()
                    }
                }
            }
        }
    }
}
