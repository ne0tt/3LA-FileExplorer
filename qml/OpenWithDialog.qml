import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

SpxDialog {
    id: d

    property var paths: []
    property string mime: ""
    property var apps: []
    property var recommended: []
    property string chosen: ""
    property string error: ""

    readonly property string chosenName: {
        const a = recommended.concat(apps).find(x => x.id === chosen)
        return a ? a.name : "this application"
    }
    // ".txt" when the files share an extension, otherwise the MIME type.
    readonly property string kindLabel: {
        const exts = paths.map(p => {
            const n = Sys.basename(p)
            const i = n.lastIndexOf(".")
            return i > 0 ? n.substring(i).toLowerCase() : ""
        })
        return exts.length && exts[0] && exts.every(e => e === exts[0]) ? exts[0] : mime
    }

    readonly property var filtered: {
        const q = search.text.toLowerCase()
        const rec = recommended.map(a => a.id)
        const match = a => !q || a.name.toLowerCase().includes(q) || a.id.toLowerCase().includes(q)
        return recommended.filter(match).map(a => Object.assign({ rec: true }, a))
            .concat(apps.filter(a => rec.indexOf(a.id) < 0 && match(a)).map(a => Object.assign({ rec: false }, a)))
    }

    function ask(paths, mime) {
        d.paths = paths
        d.mime = mime
        recommended = JSON.parse(Sys.appsFor(mime))
        apps = JSON.parse(Sys.allApps())
        chosen = recommended.length ? recommended[0].id : ""
        search.text = ""
        command.text = ""
        makeDefault.checked = false
        error = ""
        open()
    }

    function launch() {
        let err = ""
        if (command.text.trim()) {
            err = Sys.openWithCommand(command.text.trim(), paths)
        } else if (chosen) {
            if (makeDefault.checked) Sys.setDefaultApp(mime, chosen)
            err = Sys.openWith(chosen, paths)
        }
        if (err) error = err
        else close()
    }

    header: "OPEN WITH"
    tag: "RESTRICTED"
    cardWidth: 560
    buttonRow: footer
    onOpened: search.forceActiveFocus()

    SpxTitle {
        text: d.paths.length === 1 ? "Open with" : "Open " + d.paths.length + " files with"
    }
    SpxSubject { text: d.paths.length === 1 ? Sys.basename(d.paths[0] || "") : "" }
    SpxLabel { text: d.mime }
    SpxField {
        id: search
        Layout.fillWidth: true
        placeholderText: "Search applications…"
        Keys.onDownPressed: {
            const i = d.filtered.findIndex(a => a.id === d.chosen)
            if (i + 1 < d.filtered.length) d.chosen = d.filtered[i + 1].id
            list.positionViewAtIndex(i + 1, ListView.Contain)
        }
        Keys.onUpPressed: {
            const i = d.filtered.findIndex(a => a.id === d.chosen)
            if (i > 0) d.chosen = d.filtered[i - 1].id
            list.positionViewAtIndex(i - 1, ListView.Contain)
        }
        Keys.onReturnPressed: d.launch()
        Keys.onEnterPressed: d.launch()
    }
    Rectangle {
        Layout.fillWidth: true
        implicitHeight: 280
        color: "transparent"
        border.color: Theme.border
        ListView {
            id: list
            anchors { fill: parent; margins: 1 }
            clip: true
            model: d.filtered
            ScrollBar.vertical: TScrollBar {}
            delegate: SpxRow {
                id: row
                required property var modelData
                width: list.width
                height: 34
                current: d.chosen === modelData.id
                hot: ma.containsMouse
                RowLayout {
                    anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                    spacing: 10
                    Image {
                        Layout.preferredWidth: 20
                        Layout.preferredHeight: 20
                        sourceSize: Qt.size(40, 40)
                        source: "image://icon/" + (row.modelData.icon || "application-x-executable")
                        asynchronous: true
                    }
                    SpxText { text: row.modelData.name; color: row.current ? Theme.primary : Theme.text; elide: Text.ElideRight; wrapMode: Text.NoWrap }
                    SpxLabel {
                        text: row.modelData.default ? "Current" : row.modelData.rec ? "Suggested" : ""
                        color: row.modelData.default ? Theme.primary : Theme.textFaint
                    }
                }
                MouseArea {
                    id: ma
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: d.chosen = row.modelData.id
                    onDoubleClicked: { d.chosen = row.modelData.id; d.launch() }
                }
            }
        }
    }
    SpxField {
        id: command
        Layout.fillWidth: true
        placeholderText: "Or a custom command (e.g. nvim %f)"
        Keys.onReturnPressed: d.launch()
        Keys.onEnterPressed: d.launch()
    }
    SpxCheck {
        id: makeDefault
        Layout.fillWidth: true
        enabled: command.text.trim().length === 0
        text: "Always open " + d.kindLabel + " files with " + d.chosenName
    }
    SpxText { text: d.error; visible: text.length > 0; color: Theme.danger; font.pixelSize: 11 }

    SpxFooter {
        id: footer
        SpxButton { text: "Cancel"; onClicked: d.close() }
        SpxButton { text: "Open"; enabled: d.chosen.length > 0 || command.text.trim().length > 0; onClicked: d.launch() }
    }
}
