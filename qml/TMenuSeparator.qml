import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

MenuSeparator {
    id: s

    readonly property bool isSeparator: true
    // Set by TMenu when hidden items would leave this at an edge or next to
    // another separator.
    property bool collapsed: false

    height: visible && !collapsed ? implicitHeight : 0
    padding: 0
    topPadding: collapsed ? 0 : 4
    bottomPadding: collapsed ? 0 : 4
    contentItem: Rectangle {
        implicitWidth: 200
        implicitHeight: s.collapsed ? 0 : 1
        visible: !s.collapsed
        color: Theme.dividerOnChrome
    }
}
