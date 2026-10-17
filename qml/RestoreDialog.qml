import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Confirms restoring Trash items, listing where each one goes back to.
SpxDialog {
    id: d

    property var items: []      // [{ path, name, folder, exists }]
    property var acceptFn: null

    function ask(items, onAccept) {
        d.items = items
        acceptFn = onAccept
        open()
    }

    header: "RESTORE"
    tag: "RECOVERY"
    cardWidth: 520
    buttonRow: footer

    SpxTitle {
        text: d.items.length === 1 ? "Restore this item?" : "Restore " + d.items.length + " items?"
    }
    SpxSubject { text: d.items.length === 1 ? d.items[0].name : "" }
    SpxText { text: "Items go back to the folder they were deleted from." }
    SpxLabel { text: "Restore to"; Layout.topMargin: 2 }

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Math.min(list.contentHeight, 168) + 2
        color: "transparent"
        border.color: Theme.border

        ListView {
            id: list
            anchors { fill: parent; margins: 1 }
            clip: true
            interactive: contentHeight > height
            model: d.items
            ScrollBar.vertical: TScrollBar {}
            delegate: Item {
                required property var modelData
                required property int index
                width: list.width
                height: rowCol.implicitHeight + 12
                Rectangle {
                    visible: index > 0
                    anchors { left: parent.left; right: parent.right; top: parent.top }
                    height: 1
                    color: Theme.border
                }
                ColumnLayout {
                    id: rowCol
                    anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; leftMargin: 10; rightMargin: 10 }
                    spacing: 1
                    SpxText { text: modelData.name + (modelData.taken ? "  (name taken)" : ""); color: Theme.text; elide: Text.ElideMiddle; wrapMode: Text.NoWrap }
                    SpxText {
                        text: modelData.folder || "Original location unknown"
                        color: modelData.folder ? Theme.textFaint : Theme.danger
                        font.pixelSize: 11
                        elide: Text.ElideMiddle
                        wrapMode: Text.NoWrap
                    }
                }
            }
        }
    }

    SpxText {
        visible: d.items.some(i => i.taken)
        text: "Items whose original name is taken again cannot be restored until that item is moved."
        color: Theme.textFaint
        font.pixelSize: 11
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Cancel"; onClicked: d.close() }
        SpxButton {
            text: "Restore"
            onClicked: {
                d.close()
                if (d.acceptFn) d.acceptFn()
            }
        }
    }
}
