import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

// Icon view (rows flowing left-to-right) and Thunar's compact view (columns
// flowing top-to-bottom, horizontal scroll) share this implementation.
Item {
    id: root

    required property var pane
    property bool compact: false
    property int zoom: 2
    // Icon view spacing preference: 0 tight, 1 normal, 2 wide.
    // Grid spacing preference, in px added to (or taken from) each cell.
    property int spacing: 0

    readonly property var iconSizes: [24, 32, 48, 64, 96, 128, 192]
    readonly property var compactSizes: [16, 20, 24, 32, 48]
    readonly property int glyphSize: compact
        ? compactSizes[Math.max(0, Math.min(zoom, compactSizes.length - 1))]
        : iconSizes[Math.max(0, Math.min(zoom, iconSizes.length - 1))]
    // Sized from the label font so cells grow with the Font size preference
    // (at 10 pt: 92 px wide, two label lines in 48 px under the icon).
    readonly property real charWidth: textMetrics.averageCharacterWidth
    readonly property real lineHeight: Math.ceil(textMetrics.height)
    readonly property int baseCellWidth: compact
        ? Math.max(Math.ceil(charWidth * 27.5), glyphSize * 11)
        : Math.max(glyphSize + 8, Math.max(Math.ceil(charWidth * 11.5), glyphSize + 40) + spacing)

    FontMetrics {
        id: textMetrics
        font.family: Theme.fontFamily
        font.pointSize: root.compact ? root.pane.listFontSize : Theme.fontSize
    }
    readonly property Flickable flick: grid

    function positionAt(row) {
        if (row >= 0)
            grid.positionViewAtIndex(row, GridView.Contain)
    }

    function perLine() {
        return compact ? Math.max(1, Math.floor(grid.height / grid.cellHeight))
                       : Math.max(1, Math.floor(grid.width / grid.cellWidth))
    }

    function step(row, key) {
        const n = grid.count
        if (n === 0)
            return -1
        if (row < 0)
            return 0
        const k = perLine()
        const page = compact ? k * Math.max(1, Math.floor(grid.width / grid.cellWidth))
                             : k * Math.max(1, Math.floor(grid.height / grid.cellHeight))
        let r = row
        switch (key) {
        case Qt.Key_Left: r = compact ? row - k : row - 1; break
        case Qt.Key_Right: r = compact ? row + k : row + 1; break
        case Qt.Key_Up: r = compact ? row - 1 : row - k; break
        case Qt.Key_Down: r = compact ? row + 1 : row + k; break
        case Qt.Key_PageUp: r = row - page; break
        case Qt.Key_PageDown: r = row + page; break
        case Qt.Key_Home: r = 0; break
        case Qt.Key_End: r = n - 1; break
        }
        return Math.max(0, Math.min(n - 1, r))
    }

    function hitTest(x1, y1, x2, y2) {
        const n = grid.count
        const out = []
        if (n === 0)
            return out
        const cw = grid.cellWidth, ch = grid.cellHeight, k = perLine()
        const c1 = Math.max(0, Math.floor(x1 / cw)), c2 = Math.floor(x2 / cw)
        const r1 = Math.max(0, Math.floor(y1 / ch)), r2 = Math.floor(y2 / ch)
        for (let c = c1; c <= c2; ++c) {
            for (let r = r1; r <= r2; ++r) {
                if (compact && r >= k) continue
                if (!compact && c >= k) continue
                const i = compact ? c * k + r : r * k + c
                if (i < n) out.push(i)
            }
        }
        return out
    }

    GridView {
        id: grid

        anchors.fill: parent
        anchors.leftMargin: root.compact ? 4 : 0
        anchors.topMargin: 4
        model: root.pane.model
        flow: root.compact ? GridView.FlowTopToBottom : GridView.FlowLeftToRight
        cellWidth: root.compact ? root.baseCellWidth
                                : Math.floor(width / Math.max(1, Math.floor(width / root.baseCellWidth)))
        cellHeight: root.compact ? Math.max(root.glyphSize, root.lineHeight) + 10
                                 : Math.max(root.glyphSize + root.lineHeight + 12,
                                            root.glyphSize + 2 * root.lineHeight + 15 + root.spacing)
        boundsBehavior: Flickable.StopAtBounds
        clip: true
        reuseItems: true
        cacheBuffer: 600
        keyNavigationEnabled: false
        currentIndex: -1
        ScrollBar.vertical: TScrollBar { policy: root.compact ? ScrollBar.AlwaysOff : ScrollBar.AsNeeded }
        ScrollBar.horizontal: TScrollBar { policy: root.compact ? ScrollBar.AsNeeded : ScrollBar.AlwaysOff }

        RubberBand {
            pane: root.pane
            flick: grid
            hitTest: root.hitTest
        }

        // One delegate type per layout: building both and hiding one doubled
        // every item's cost (two mouse/drop areas, two sets of bindings).
        delegate: root.compact ? compactDelegate : iconDelegate
    }

    // Icon view: a centred column of icon + wrapped label.
    Component {
        id: iconDelegate

        Item {
            id: del

            required property int index
            required property var model

            readonly property bool selected: root.pane.selection[model.path] === true
            readonly property bool isCurrent: root.pane.currentRow === index && root.pane.activeFocus && root.pane.keyboardCursor
            readonly property bool cut: root.pane.cutSet[model.path] === true
            readonly property bool thumb: root.pane.showThumbnails && root.pane.canThumbnail(model.isImage, model.category) && root.glyphSize >= 32

            width: grid.cellWidth
            height: grid.cellHeight
            z: selected ? 3 : 1

            Item {
                id: box
                anchors.horizontalCenter: parent.horizontalCenter
                y: 1
                width: parent.width - 4
                height: iconHolder.height + nameBg.height + 4

                Rectangle {
                    anchors.centerIn: iconHolder
                    width: root.glyphSize + 14
                    height: root.glyphSize + 10
                    radius: Theme.radius
                    color: del.selected ? Qt.alpha(Theme.selection, 0.6) : mouse.containsMouse || mouse.dropHover ? Theme.hover : "transparent"
                    border.width: mouse.dropHover ? 2 : 0
                    border.color: Theme.dropTarget
                }

                Item {
                    id: iconHolder
                    anchors.horizontalCenter: parent.horizontalCenter
                    y: 4
                    width: root.glyphSize
                    height: root.glyphSize
                    opacity: del.cut ? 0.45 : (del.model.isHidden ? 0.55 : 1)

                    FileIcon {
                        anchors.fill: parent
                        path: del.model.path
                        glyph: del.model.glyph
                        category: del.model.category
                        size: root.glyphSize
                        glyphScale: del.model.isDir ? 1 : 0.8
                        thumbnail: del.thumb
                    }
                    Glyph {
                        anchors.right: parent.right
                        anchors.bottom: parent.bottom
                        anchors.margins: -2
                        visible: del.model.isLink
                        text: "\u{F0C1}"
                        size: Math.max(10, root.glyphSize / 3)
                        color: Theme.folderIcon
                        style: Text.Outline
                        styleColor: Theme.view
                    }
                }

                Rectangle {
                    id: nameBg
                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.top: iconHolder.bottom
                    anchors.topMargin: 4
                    width: Math.min(box.width, nameText.implicitWidth + 6)
                    height: nameText.height + 4
                    radius: Theme.smallRadius
                    color: del.selected ? (root.pane.activeFocus ? Theme.selection : Theme.selectionInactive) : "transparent"
                    border.width: del.isCurrent && !del.selected ? 1 : 0
                    border.color: Theme.focus

                    Text {
                        id: nameText
                        anchors.centerIn: parent
                        width: box.width - 6
                        text: del.model.name
                        color: del.selected ? Theme.selectionText : Theme.text
                        font.family: Theme.fontFamily
                        font.pointSize: Theme.fontSize
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                        maximumLineCount: del.selected ? 8 : 2
                        elide: Text.ElideRight
                        opacity: del.cut ? 0.55 : 1
                    }
                }

                ItemMouse {
                    id: mouse
                    pane: root.pane
                    row: del.index
                    path: del.model.path
                    isDir: del.model.isDir
                }
            }
        }
    }

    // Compact view: a single row of icon + label.
    Component {
        id: compactDelegate

        Item {
            id: del

            required property int index
            required property var model

            readonly property bool selected: root.pane.selection[model.path] === true
            readonly property bool isCurrent: root.pane.currentRow === index && root.pane.activeFocus && root.pane.keyboardCursor
            readonly property bool cut: root.pane.cutSet[model.path] === true

            width: grid.cellWidth
            height: grid.cellHeight
            z: selected ? 3 : 1

            Rectangle {
                anchors.fill: parent
                anchors.margins: 1
                anchors.rightMargin: 6
                radius: Theme.smallRadius
                color: del.selected ? (root.pane.activeFocus ? Theme.selection : Theme.selectionInactive)
                     : cmouse.containsMouse || cmouse.dropHover ? Theme.hover : "transparent"
                border.width: cmouse.dropHover ? 2 : del.isCurrent ? 1 : 0
                border.color: cmouse.dropHover ? Theme.dropTarget : Theme.focus

                Row {
                    id: crow
                    anchors.fill: parent
                    anchors.leftMargin: 6
                    spacing: 8
                    opacity: del.cut ? 0.45 : (del.model.isHidden ? 0.55 : 1)
                    FileIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        width: root.glyphSize + 2
                        height: root.glyphSize
                        path: del.model.path
                        glyph: del.model.glyph
                        category: del.model.category
                        size: root.glyphSize
                        thumbnail: root.pane.listThumbnails && root.pane.canThumbnail(del.model.isImage, del.model.category)
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: crow.width - root.glyphSize - 18
                        text: del.model.name
                        elide: Text.ElideRight
                        color: del.selected ? Theme.selectionText : Theme.text
                        font.family: Theme.fontFamily
                        font.pointSize: root.pane.listFontSize
                        font.italic: del.model.isLink
                    }
                }

                ItemMouse {
                    id: cmouse
                    pane: root.pane
                    row: del.index
                    path: del.model.path
                    isDir: del.model.isDir
                }
            }
        }
    }
}
