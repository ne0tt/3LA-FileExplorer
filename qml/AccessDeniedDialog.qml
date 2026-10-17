import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

SpxDialog {
    id: d

    property string path: ""
    property bool isDir: true

    function show(path, isDir) {
        d.path = path
        d.isDir = isDir
        open()
    }

    header: "ACCESS DENIED"
    tag: "RESTRICTED"
    danger: true
    cardWidth: 420
    buttonRow: footer

    SpxTitle { text: "Permission denied" }
    SpxText {
        text: d.isDir ? "You do not have permission to view this folder."
                      : "You do not have permission to view this file."
    }
    SpxText { text: d.path; color: Theme.textFaint; wrapMode: Text.WrapAnywhere }

    SpxFooter {
        id: footer
        SpxButton { text: "Close"; destructive: true; onClicked: d.close() }
    }
}
