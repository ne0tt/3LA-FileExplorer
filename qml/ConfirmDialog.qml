import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Generic SPX confirmation / notice. ask({ header, tag, title, subject, message, detail,
// acceptText, cancelText ("" hides it), danger, defaultAccept, onAccept }).
SpxDialog {
    id: d

    property string titleText: ""
    property string subject: ""
    property string message: ""
    property string detail: ""
    property string acceptText: "OK"
    property string cancelText: "Cancel"
    property var acceptFn: null

    function ask(opts) {
        header = (opts.header || opts.title || "").toUpperCase()
        tag = opts.tag || ""
        titleText = opts.title || ""
        subject = opts.subject || ""
        message = opts.message || ""
        detail = opts.detail || ""
        acceptText = opts.acceptText || "OK"
        cancelText = opts.cancelText !== undefined ? opts.cancelText : "Cancel"
        danger = !!opts.danger
        acceptFn = opts.onAccept || null
        defaultIndex = opts.defaultAccept === false && cancelText ? 0 : -1
        open()
    }

    cardWidth: 460
    buttonRow: footer

    SpxTitle { text: d.titleText; visible: text.length > 0 }
    SpxSubject { text: d.subject }
    SpxText { text: d.message; visible: text.length > 0 }
    SpxText {
        text: d.detail
        visible: text.length > 0
        color: Theme.textFaint
        font.pixelSize: 11
        maximumLineCount: 14
        elide: Text.ElideRight
    }

    SpxFooter {
        id: footer
        SpxButton {
            text: d.cancelText
            visible: d.cancelText.length > 0
            onClicked: d.close()
        }
        SpxButton {
            text: d.acceptText
            destructive: d.danger
            onClicked: {
                d.close()
                if (d.acceptFn) d.acceptFn()
            }
        }
    }
}
