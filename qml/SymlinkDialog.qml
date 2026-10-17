import QtQuick
import QtQuick.Layouts
import ThreeLA.FileExplorer

// The destination filesystem refused a symlink (SMB, FAT…): skip it or abort.
SpxDialog {
    id: d

    property string name: ""
    property string destination: ""
    property bool answered: false

    function answer(choice) {
        answered = true
        close()
        FileOps.resolveConflict(choice)
    }

    header: "SYMLINK"
    tag: "ATTENTION"
    cardWidth: 520
    defaultIndex: 1
    buttonRow: footer
    // Above the progress panel, which may open after this prompt.
    z: 10
    onClosed: if (!answered) FileOps.resolveConflict(FileOps.Cancel)

    Connections {
        target: FileOps
        function onSymlinkConflict(name, destination) {
            d.name = name
            d.destination = destination
            d.answered = false
            d.open()
        }
    }

    SpxTitle { text: "Symlinks not supported" }
    SpxSubject { text: d.name }
    SpxText { text: "The destination cannot store symbolic links. Skip it?" }
    SpxText { text: d.destination; color: Theme.textFaint; wrapMode: Text.WrapAnywhere }

    SpxFooter {
        id: footer
        SpxButton { text: "Cancel"; onClicked: d.answer(FileOps.Cancel) }
        SpxButton { text: "Skip"; onClicked: d.answer(FileOps.Skip) }
        SpxButton { text: "Skip All"; onClicked: d.answer(FileOps.SkipAll) }
    }
}
