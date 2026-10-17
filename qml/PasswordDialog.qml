import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Password prompt for encrypting (password + confirmation, strength hint) and
// decrypting. Fields are cleared whenever the dialog closes.
SpxDialog {
    id: d

    property bool encrypting: false
    property string titleText: ""
    property string subject: ""
    property string error: ""
    property var acceptFn: null

    readonly property int minLength: 8
    readonly property int strength: {
        const p = pw.text
        let classes = 0
        for (const re of [/[a-z]/, /[A-Z]/, /[0-9]/, /[^A-Za-z0-9]/])
            if (re.test(p)) classes++
        const score = (p.length >= 8) + (p.length >= 12) + (p.length >= 16) + (classes >= 2) + (classes >= 3)
        return p.length < d.minLength ? 0 : Math.min(3, Math.floor(score * 3 / 5) + 1)
    }

    function ask(opts) {
        encrypting = opts.encrypting === true
        header = opts.header || (encrypting ? "ENCRYPT" : "DECRYPT")
        tag = opts.tag || (encrypting ? "SEALED" : "LOCKED")
        titleText = opts.title || ""
        subject = opts.subject || ""
        acceptFn = opts.onAccept || null
        error = opts.error || ""
        wipe()
        reveal.checked = false
        open()
        pw.forceActiveFocus()
    }

    function wipe() {
        pw.clear()
        pw2.clear()
    }

    function submit() {
        const p = pw.text
        if (!p) return
        if (encrypting) {
            if (p.length < minLength) { error = "Use at least " + minLength + " characters"; return }
            if (p !== pw2.text) { error = "The passwords don't match"; return }
        }
        const f = acceptFn
        close()
        if (f) f(p)
    }

    cardWidth: 460
    buttonRow: footer
    danger: encrypting
    onOpened: pw.forceActiveFocus()
    onClosed: { wipe(); acceptFn = null }

    SpxTitle { text: d.titleText }
    SpxSubject { text: d.subject }
    SpxText {
        Layout.fillWidth: true
        color: Theme.textFaint
        text: d.encrypting
            ? "The originals are deleted once the encrypted copy is verified. Without this password the data cannot be recovered."
            : "Enter the password used to encrypt it."
    }
    SpxField {
        id: pw
        Layout.fillWidth: true
        placeholderText: "Password"
        echoMode: reveal.checked ? TextInput.Normal : TextInput.Password
        inputMethodHints: Qt.ImhSensitiveData | Qt.ImhNoPredictiveText
        onTextEdited: d.error = ""
        Keys.onReturnPressed: d.encrypting ? pw2.forceActiveFocus() : d.submit()
        Keys.onEnterPressed: d.encrypting ? pw2.forceActiveFocus() : d.submit()
    }
    SpxField {
        id: pw2
        visible: d.encrypting
        Layout.fillWidth: true
        placeholderText: "Confirm password"
        echoMode: reveal.checked ? TextInput.Normal : TextInput.Password
        inputMethodHints: Qt.ImhSensitiveData | Qt.ImhNoPredictiveText
        onTextEdited: d.error = ""
        Keys.onReturnPressed: d.submit()
        Keys.onEnterPressed: d.submit()
    }
    RowLayout {
        Layout.fillWidth: true
        SpxCheck { id: reveal; text: "Show password" }
        Item { Layout.fillWidth: true }
        SpxLabel {
            visible: d.encrypting && pw.text.length > 0
            text: ["Too short", "Weak", "Fair", "Strong"][d.strength]
            color: [Theme.danger, Theme.danger, Theme.accent2, Theme.success][d.strength]
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
        SpxButton {
            text: d.encrypting ? "Encrypt" : "Decrypt"
            destructive: d.encrypting
            enabled: pw.text.length > 0
            onClicked: d.submit()
        }
    }
}
