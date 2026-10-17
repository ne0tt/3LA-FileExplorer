import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

// SPX "NETWORK AUTH": credentials for an SMB server. By default they live in
// memory for this run only; "Remember" stores them in the login keyring.
// They are fed to gio/smbclient/secret-tool via stdin/env, never as arguments.
SpxDialog {
    id: d

    property string uri: ""
    property string host: ""
    property string message: ""
    property var connectFn: null

    function ask(uri, host, message, onConnect) {
        d.uri = uri
        d.host = host
        d.message = message
        connectFn = onConnect
        if (!user.text) user.text = Sys.basename(Sys.homePath)
        if (!domain.text) domain.text = "WORKGROUP"
        password.text = ""
        anonymous.checked = false
        remember.checked = false
        open()
    }

    function connect() {
        Sys.setSmbCredentials(host, user.text, domain.text, password.text, anonymous.checked, remember.checked)
        password.text = ""
        close()
        if (connectFn) connectFn()
    }

    header: "NETWORK AUTH"
    tag: "CREDENTIALS"
    cardWidth: 460
    buttonRow: footer
    onOpened: (anonymous.checked ? user : password).forceActiveFocus()

    SpxTitle { text: "Connect to " + d.host }
    SpxText { text: d.uri; color: Theme.textFaint; wrapMode: Text.WrapAnywhere }
    SpxText {
        visible: d.message.length > 0 && d.message.indexOf("Authentication required") !== 0
        text: d.message
        color: Theme.danger
        font.pixelSize: 11
    }

    GridLayout {
        Layout.fillWidth: true
        columns: 2
        columnSpacing: 12
        rowSpacing: 8
        enabled: !anonymous.checked
        opacity: enabled ? 1 : 0.45

        SpxLabel { text: "User" }
        SpxField { id: user; Layout.fillWidth: true; Keys.onReturnPressed: d.connect() }
        SpxLabel { text: "Domain" }
        SpxField { id: domain; Layout.fillWidth: true; Keys.onReturnPressed: d.connect() }
        SpxLabel { text: "Password" }
        SpxField {
            id: password
            Layout.fillWidth: true
            echoMode: TextInput.Password
            Keys.onReturnPressed: d.connect()
        }
    }
    SpxCheck { id: anonymous; text: "Connect anonymously (guest)"; Layout.fillWidth: true }
    SpxCheck { id: remember; text: "Remember my credentials"; Layout.fillWidth: true }
    SpxText {
        text: remember.checked
            ? "Stored encrypted in your login keyring (Secret Service) and reused next time."
            : "Forgotten when the app closes; shares opened with them are unmounted on exit."
        color: Theme.textFaint
        font.pixelSize: 11
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Cancel"; onClicked: d.close() }
        SpxButton { text: "Connect"; onClicked: d.connect() }
    }
}
