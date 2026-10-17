import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Single-field SPX prompt (new folder/file, rename, bookmark alias, pattern).
SpxDialog {
    id: d

    property string titleText: ""
    property string label: ""
    property string acceptText: "OK"
    property string error: ""
    property var acceptFn: null
    property var validate: null

    function ask(opts) {
        header = (opts.header || opts.title || "").toUpperCase()
        tag = opts.tag || "RESTRICTED"
        titleText = opts.title || ""
        label = opts.label || ""
        acceptText = opts.acceptText || "OK"
        acceptFn = opts.onAccept || null
        validate = opts.validate || null
        error = ""
        field.text = opts.text || ""
        field.placeholderText = opts.placeholder || ""
        open()
        field.forceActiveFocus()
        if (opts.selectStem) {
            const t = field.text
            const multi = [".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst"].find(x => t.toLowerCase().endsWith(x))
            const dot = multi ? t.length - multi.length : t.lastIndexOf(".")
            field.select(0, dot > 0 ? dot : t.length)
        } else {
            field.selectAll()
        }
    }

    function submit() {
        const t = field.text
        if (validate) {
            const e = validate(t)
            if (e) { error = e; return }
        }
        if (acceptFn) {
            const e = acceptFn(t)
            if (e) { error = e; return }
        }
        close()
    }

    cardWidth: 440
    buttonRow: footer
    onOpened: field.forceActiveFocus()

    SpxTitle { text: d.titleText }
    SpxText { text: d.label; visible: text.length > 0; color: Theme.textFaint }
    SpxField {
        id: field
        Layout.fillWidth: true
        onTextEdited: d.error = ""
        Keys.onReturnPressed: d.activateSelected()
        Keys.onEnterPressed: d.activateSelected()
        Keys.onUpPressed: d.selectedIndex = 0
        Keys.onDownPressed: d.selectedIndex = 1
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
        SpxButton { text: d.acceptText; enabled: field.text.length > 0; onClicked: d.submit() }
    }
}
