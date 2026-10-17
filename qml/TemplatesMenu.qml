import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

// Thunar's "Create Document" submenu, fed from ~/Templates.
TMenu {
    id: m

    property var templates: []

    signal create(var tpl)

    onAboutToShow: templates = JSON.parse(Sys.templates())

    TMenuItem { text: "Empty File"; icon.name: G.filePlus; onTriggered: m.create(null) }
    TMenuSeparator {}
    Instantiator {
        model: m.templates
        delegate: TMenuItem {
            required property var modelData
            text: modelData.name
            icon.name: modelData.glyph
            onTriggered: m.create(modelData)
        }
        onObjectAdded: (i, o) => m.insertItem(i + 2, o)
        onObjectRemoved: (i, o) => m.removeItem(o)
    }
    TMenuItem {
        text: "No templates installed"
        enabled: false
        visible: m.templates.length === 0
    }
}
