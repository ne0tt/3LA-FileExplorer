import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

SpxDialog {
    id: d

    header: "KEYBOARD"
    tag: "FIELD MANUAL"
    cardWidth: 680
    buttonRow: footer

    readonly property var keys: [
        ["Ctrl+T / Ctrl+W", "New tab / close tab"],
        ["Ctrl+PgUp / Ctrl+PgDown", "Previous / next tab"],
        ["Ctrl+N", "New window"],
        ["Ctrl+Shift+N", "Create folder"],
        ["Ctrl+L", "Edit location"],
        ["Ctrl+F", "Filter current folder"],
        ["Ctrl+Left / Ctrl+Right / Backspace", "Back / forward / back"],
        ["Alt+Up / Alt+Home", "Parent folder / home"],
        ["F2", "Rename"],
        ["Delete / Shift+Delete", "Move to Trash / delete permanently"],
        ["Ctrl+C / Ctrl+X / Ctrl+V", "Copy / cut / paste"],
        ["Ctrl+Z", "Undo last operation"],
        ["Ctrl+A / Ctrl+Shift+I", "Select all / invert selection"],
        ["Ctrl+S", "Select by pattern"],
        ["Ctrl+H", "Toggle hidden files"],
        ["Ctrl+1 / 2 / 3", "Icon / detailed / compact view"],
        ["Ctrl+= / Ctrl+- / Ctrl+0", "Zoom in / out / reset"],
        ["Ctrl+R / F5", "Reload"],
        ["F9 / Ctrl+M", "Toggle side pane / menubar"],
        ["F4", "Open terminal here"],
        ["Ctrl+D", "Bookmark current folder"],
        ["Alt+Return", "Properties"],
        ["Ctrl+Q", "Close window"],
        ["Type letters", "Jump to matching item"],
        ["Middle click", "Open folder in new tab"]
    ]

    SpxTitle { text: "Keyboard Shortcuts" }
    SpxText { text: "3LA File Explorer — a Thunar-style file manager in Rust + Qt 6." }
    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Math.min(440, (Overlay.overlay ? Overlay.overlay.height : 800) - 220)
        color: "transparent"
        border.color: Theme.border
        ListView {
            anchors { fill: parent; margins: 6 }
            clip: true
            model: d.keys
            ScrollBar.vertical: TScrollBar {}
            delegate: RowLayout {
                required property var modelData
                width: ListView.view.width
                height: 26
                SpxText { text: modelData[0]; color: Theme.primary; Layout.preferredWidth: 280; Layout.fillWidth: false; wrapMode: Text.NoWrap }
                SpxText { text: modelData[1]; elide: Text.ElideRight; wrapMode: Text.NoWrap }
            }
        }
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Close"; onClicked: d.close() }
    }
}
