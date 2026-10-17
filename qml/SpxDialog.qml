import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Modal SPX-style dialog. Children form the body column; point `buttonRow`
// at an SpxFooter so Left/Right move the selected button and Enter fires it.
Popup {
    id: d

    property string header: ""
    property string tag: ""
    property bool danger: false
    property color accent: danger ? Theme.danger : Theme.primary
    property int cardWidth: 440
    property int defaultIndex: -1
    property int selectedIndex: 0
    property Item buttonRow: null
    default property alias body: content.data

    // Fired for Esc when closePolicy doesn't already close on Escape.
    signal escapePressed()

    function buttonList() {
        const out = []
        if (!buttonRow)
            return out
        for (let i = 0; i < buttonRow.children.length; ++i) {
            const c = buttonRow.children[i]
            if (c.isSpxButton === true && c.visible && c.enabled)
                out.push(c)
        }
        return out
    }

    function syncSelection() {
        const b = buttonList()
        for (let i = 0; i < b.length; ++i)
            b[i].selected = i === selectedIndex
    }

    function select(button) {
        const i = buttonList().indexOf(button)
        if (i >= 0) selectedIndex = i
    }

    function activateSelected() {
        const b = buttonList()
        if (b[selectedIndex])
            b[selectedIndex].clicked()
    }

    onSelectedIndexChanged: syncSelection()
    onAboutToShow: {
        const n = buttonList().length
        selectedIndex = defaultIndex < 0 ? Math.max(0, n - 1) : Math.min(defaultIndex, Math.max(0, n - 1))
        syncSelection()
    }
    onOpened: scope.forceActiveFocus()

    parent: Overlay.overlay
    anchors.centerIn: parent
    modal: true
    focus: true
    padding: 0
    width: Math.min(cardWidth, (parent ? parent.width : cardWidth) - 32)
    closePolicy: Popup.CloseOnEscape

    Overlay.modal: Rectangle { color: "#80000000" }

    enter: Transition { NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 110 } }
    exit: Transition { NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 90 } }

    background: SpxCard {
        id: card
        header: d.header
        tag: d.tag
        accent: d.accent
        danger: d.danger
    }

    contentItem: FocusScope {
        id: scope
        focus: true
        implicitWidth: d.cardWidth
        implicitHeight: card.headerHeight + content.implicitHeight + 30

        ColumnLayout {
            id: content
            anchors { top: parent.top; topMargin: card.headerHeight + 12; left: parent.left; right: parent.right; leftMargin: 18; rightMargin: 18 }
            spacing: 12
        }

        Keys.onPressed: event => {
            const n = d.buttonList().length
            if (event.key === Qt.Key_Left) {
                d.selectedIndex = Math.max(0, d.selectedIndex - 1)
            } else if (event.key === Qt.Key_Right) {
                d.selectedIndex = Math.min(n - 1, d.selectedIndex + 1)
            } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                // A button reached with Tab is what Enter means, not the
                // highlighted default.
                const f = scope.Window.activeFocusItem
                if (f && f.isSpxButton === true && f.enabled) f.clicked()
                else d.activateSelected()
            } else if (event.key === Qt.Key_Escape) {
                if (d.closePolicy & Popup.CloseOnEscape) d.close()
                else d.escapePressed()
            } else {
                return
            }
            event.accepted = true
        }
    }
}
