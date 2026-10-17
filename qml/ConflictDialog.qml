import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Answers FileOps.conflict(); the worker thread blocks until we resolve.
SpxDialog {
    id: d

    property string source: ""
    property string target: ""
    property var existing: ({})
    property var incoming: ({})
    property bool answered: false

    function answer(single, all) {
        answered = true
        close()
        FileOps.resolveConflict(applyAll.checked ? all : single)
    }

    header: "CONFLICT"
    tag: "ATTENTION"
    cardWidth: 720
    buttonRow: footer
    // Above the progress panel, which may open after this prompt.
    z: 10
    onClosed: if (!answered) FileOps.resolveConflict(FileOps.Cancel)

    Connections {
        target: FileOps
        function onConflict(source, target, sourceInfo, targetInfo) {
            d.source = source
            d.target = target
            d.incoming = JSON.parse(sourceInfo || "{}")
            d.existing = JSON.parse(targetInfo || "{}")
            d.answered = false
            applyAll.checked = false
            d.open()
        }
    }

    component FileCard: Rectangle {
        id: fc
        property string label
        property var info: ({})
        Layout.fillWidth: true
        Layout.preferredWidth: 1
        implicitHeight: col.implicitHeight + 20
        color: "transparent"
        border.color: Theme.border

        ColumnLayout {
            id: col
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 10 }
            spacing: 3
            SpxLabel { text: fc.label; Layout.bottomMargin: 3 }
            RowLayout {
                Layout.fillWidth: true
                spacing: 10
                Glyph {
                    Layout.alignment: Qt.AlignTop
                    text: fc.info.glyph || G.file
                    size: 24
                    color: Theme.categoryColor(fc.info.category)
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2
                    SpxText { text: fc.info.name || ""; font.bold: true; color: Theme.text; wrapMode: Text.WrapAnywhere }
                    SpxText { text: (fc.info.type || "") + " · " + (fc.info.size || ""); color: Theme.textFaint; font.pixelSize: 11; elide: Text.ElideRight; wrapMode: Text.NoWrap }
                    SpxText { text: fc.info.modified || ""; color: Theme.textFaint; font.pixelSize: 11; elide: Text.ElideRight; wrapMode: Text.NoWrap }
                }
            }
        }
    }

    SpxTitle { text: "Name conflict" }
    SpxSubject { text: Sys.basename(d.target) }
    SpxText {
        text: "already exists in \"" + Sys.parent(d.target) + "\"."
        wrapMode: Text.WrapAnywhere
    }
    RowLayout {
        Layout.fillWidth: true
        spacing: 10
        FileCard { label: "On target"; info: d.existing }
        FileCard { label: "Incoming"; info: d.incoming }
    }
    SpxCheck {
        id: applyAll
        Layout.fillWidth: true
        text: "Apply Skip / Keep Both / Replace to all remaining conflicts"
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Cancel"; onClicked: { d.answered = true; d.close(); FileOps.resolveConflict(FileOps.Cancel) } }
        SpxButton { text: "Skip"; onClicked: d.answer(FileOps.Skip, FileOps.SkipAll) }
        SpxButton { text: "Skip All"; onClicked: d.answer(FileOps.SkipAll, FileOps.SkipAll) }
        SpxButton { text: "Keep Both"; onClicked: d.answer(FileOps.KeepBoth, FileOps.KeepBothAll) }
        SpxButton { text: "Replace All"; destructive: true; onClicked: d.answer(FileOps.ReplaceAll, FileOps.ReplaceAll) }
        SpxButton { text: "Replace"; destructive: true; onClicked: d.answer(FileOps.Replace, FileOps.ReplaceAll) }
    }
}
