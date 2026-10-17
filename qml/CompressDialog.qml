import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Archive name + format picker for "Compress…".
SpxDialog {
    id: d

    property var sources: []
    property string dir: ""
    property string ext: ".zip"
    property string error: ""

    readonly property var formats: [
        { ext: ".zip", note: "Most portable" },
        { ext: ".tar.gz", note: "Unix tarball, gzip" },
        { ext: ".tar.xz", note: "Smallest tarball, slow" },
        { ext: ".tar.zst", note: "Fast, good ratio" },
        { ext: ".7z", note: "High ratio" },
        { ext: ".tar", note: "No compression" }
    ]

    function ask(paths, folder) {
        sources = paths
        dir = folder
        error = ""
        const base = paths.length === 1 ? Sys.basename(paths[0]) : Sys.basename(folder) || "Archive"
        field.text = base
        open()
        field.forceActiveFocus()
        field.selectAll()
    }

    function submit() {
        const name = field.text.trim()
        if (!name || name.indexOf("/") >= 0) { error = "Invalid file name"; return }
        const target = Sys.join(dir, name + ext)
        if (Sys.exists(target)) { error = "\"" + name + ext + "\" already exists"; return }
        FileOps.compress(sources, target)
        close()
    }

    header: "COMPRESS"
    tag: "PACK"
    cardWidth: 460
    buttonRow: footer
    onOpened: field.forceActiveFocus()

    SpxTitle {
        text: d.sources.length === 1 ? "Compress" : "Compress " + d.sources.length + " items"
    }
    SpxSubject { text: d.sources.length === 1 ? Sys.basename(d.sources[0] || "") : "" }
    SpxText { text: "in " + d.dir; color: Theme.textFaint }
    RowLayout {
        Layout.fillWidth: true
        spacing: 6
        SpxField {
            id: field
            Layout.fillWidth: true
            placeholderText: "Archive name"
            onTextEdited: d.error = ""
            Keys.onReturnPressed: d.activateSelected()
            Keys.onEnterPressed: d.activateSelected()
        }
        SpxText { text: d.ext; color: Theme.primary; wrapMode: Text.NoWrap }
    }
    GridLayout {
        Layout.fillWidth: true
        columns: 2
        columnSpacing: 6
        rowSpacing: 4
        Repeater {
            model: d.formats
            SpxRow {
                id: row
                required property var modelData
                Layout.fillWidth: true
                implicitHeight: 26
                current: d.ext === modelData.ext
                hot: ma.containsMouse
                RowLayout {
                    anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                    SpxText { text: row.modelData.ext; color: row.current ? Theme.primary : Theme.text; wrapMode: Text.NoWrap }
                    SpxLabel { text: row.modelData.note; Layout.fillWidth: true; horizontalAlignment: Text.AlignRight }
                }
                MouseArea {
                    id: ma
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        d.ext = row.modelData.ext
                        d.error = ""
                    }
                }
            }
        }
    }
    SpxText {
        text: d.error
        visible: text.length > 0
        color: Theme.danger
        font.pixelSize: 11
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Cancel"; onClicked: d.close() }
        SpxButton { text: "Compress"; enabled: field.text.trim().length > 0; onClicked: d.submit() }
    }
}
