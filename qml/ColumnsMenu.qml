import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

TMenu {
    id: m

    required property var prefs
    readonly property var cols: { try { return JSON.parse(prefs.columns) } catch (e) { return [] } }

    function toggle(key) {
        const c = cols.slice()
        const i = c.indexOf(key)
        if (i >= 0) c.splice(i, 1)
        else c.push(key)
        const order = ["size", "type", "modified", "permissions", "owner", "group", "mime"]
        c.sort((a, b) => order.indexOf(a) - order.indexOf(b))
        prefs.columns = JSON.stringify(c)
    }

    Instantiator {
        model: [
            { k: "size", t: "Size" },
            { k: "type", t: "Type" },
            { k: "modified", t: "Date Modified" },
            { k: "permissions", t: "Permissions" },
            { k: "owner", t: "Owner" },
            { k: "group", t: "Group" },
            { k: "mime", t: "MIME Type" }
        ]
        delegate: TMenuItem {
            required property var modelData
            text: modelData.t
            checkable: true
            checked: m.cols.indexOf(modelData.k) >= 0
            // Triggering toggles `checked` and breaks the binding; restore it.
            onTriggered: {
                m.toggle(modelData.k)
                checked = Qt.binding(() => m.cols.indexOf(modelData.k) >= 0)
            }
        }
        onObjectAdded: (i, o) => m.insertItem(i, o)
        onObjectRemoved: (i, o) => m.removeItem(o)
    }
}
