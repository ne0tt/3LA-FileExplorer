import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

Item {
    id: root

    required property var pane
    // Shares the sidebar's text size (see BrowserPane.listFontSize).
    readonly property int fontSize: pane.listFontSize
    property int zoom: 1

    readonly property var glyphSizes: [14, 16, 20, 24, 32]
    readonly property int glyphSize: glyphSizes[Math.max(0, Math.min(zoom, glyphSizes.length - 1))]
    readonly property int rowHeight: Math.max(glyphSize, Math.ceil(textMetrics.height)) + 10

    FontMetrics {
        id: textMetrics
        font.family: Theme.fontFamily
        font.pointSize: root.fontSize
    }
    readonly property Flickable flick: list

    // key, title, model role, sort key (-1 = not sortable), default width
    readonly property var allColumns: [
        { key: "name", title: "Name", role: "name", sort: 0, width: 320 },
        { key: "origPath", title: "Original Location", role: "origPath", sort: -1, width: 260 },
        { key: "deleted", title: "Deleted", role: "deletedText", sort: -1, width: 160 },
        { key: "size", title: "Size", role: "sizeText", sort: 1, width: 100 },
        { key: "type", title: "Type", role: "typeText", sort: 2, width: 170 },
        { key: "modified", title: "Date Modified", role: "modifiedText", sort: 3, width: 170 },
        { key: "permissions", title: "Permissions", role: "permissions", sort: -1, width: 110 },
        { key: "owner", title: "Owner", role: "owner", sort: -1, width: 90 },
        { key: "group", title: "Group", role: "group", sort: -1, width: 90 },
        { key: "mime", title: "MIME Type", role: "mime", sort: -1, width: 180 }
    ]

    // Which columns show; widths live apart (widthOf) so resizing a column
    // updates cell widths instead of rebuilding every row's cells.
    readonly property var columns: {
        const vis = pane.visibleColumns
        return allColumns.filter(c => {
            if (c.key === "name") return true
            if (c.key === "origPath" || c.key === "deleted") return pane.isTrash
            return vis.indexOf(c.key) >= 0
        })
    }
    // Name is always first and drawn outside the per-cell Repeater.
    readonly property var extraColumns: columns.slice(1)
    readonly property var defaultWidths: {
        const m = {}
        for (const c of allColumns) m[c.key] = c.width
        return m
    }
    // Width being dragged; written to the preferences once on release.
    property var liveWidths: ({})

    function widthOf(key) {
        return liveWidths[key] ?? pane.columnWidths[key] ?? defaultWidths[key]
    }

    readonly property int totalWidth: columns.reduce((a, c) => a + widthOf(c.key), 0) + 12

    function positionAt(row) {
        if (row >= 0)
            list.positionViewAtIndex(row, ListView.Contain)
    }

    function step(row, key) {
        const n = list.count
        if (n === 0)
            return -1
        if (row < 0)
            return 0
        const page = Math.max(1, Math.floor(list.height / rowHeight) - 1)
        switch (key) {
        case Qt.Key_Up: return Math.max(0, row - 1)
        case Qt.Key_Down: return Math.min(n - 1, row + 1)
        case Qt.Key_PageUp: return Math.max(0, row - page)
        case Qt.Key_PageDown: return Math.min(n - 1, row + page)
        case Qt.Key_Home: return 0
        case Qt.Key_End: return n - 1
        }
        // Left/Right do nothing here; returning the row would collapse a
        // multi-selection to it.
        return -1
    }

    function hitTest(x1, y1, x2, y2) {
        const out = []
        const r1 = Math.max(0, Math.floor(y1 / rowHeight))
        const r2 = Math.min(list.count - 1, Math.floor(y2 / rowHeight))
        for (let r = r1; r <= r2; ++r)
            out.push(r)
        return out
    }

    // Column header, kept in sync with the list's horizontal scroll.
    Rectangle {
        id: header
        width: parent.width
        height: Math.round(28 + Math.max(0, root.fontSize - 10) * 1.6)
        color: Theme.chrome
        clip: true
        z: 2

        Row {
            x: -list.contentX
            height: parent.height
            Repeater {
                model: root.columns
                delegate: Item {
                    id: hcell
                    required property var modelData
                    required property int index
                    width: root.widthOf(modelData.key)
                    height: header.height

                    Rectangle {
                        anchors.fill: parent
                        color: hma.containsMouse && hcell.modelData.sort >= 0 ? Theme.hover : "transparent"
                    }
                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: 10
                        anchors.right: sortGlyph.left
                        anchors.verticalCenter: parent.verticalCenter
                        text: hcell.modelData.title
                        color: Theme.textDim
                        font.family: Theme.fontFamily
                        font.pointSize: Math.max(6, root.fontSize - 1)
                        font.bold: root.pane.model.sortKey === hcell.modelData.sort
                        elide: Text.ElideRight
                    }
                    Glyph {
                        id: sortGlyph
                        anchors.right: parent.right
                        anchors.rightMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        visible: root.pane.model.sortKey === hcell.modelData.sort
                        text: root.pane.model.sortDescending ? G.chevronDown : G.chevronUp
                        color: Theme.primary
                        size: 14
                    }
                    MouseArea {
                        id: hma
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        onClicked: mouse => {
                            if (mouse.button === Qt.RightButton)
                                root.pane.columnsMenuRequested()
                            else if (hcell.modelData.sort >= 0)
                                root.pane.sortBy(hcell.modelData.sort)
                        }
                    }
                    Rectangle {
                        anchors.right: parent.right
                        width: 1
                        height: parent.height - 10
                        anchors.verticalCenter: parent.verticalCenter
                        color: Theme.dividerOnChrome
                    }
                    MouseArea {
                        anchors.right: parent.right
                        anchors.rightMargin: -4
                        width: 8
                        height: parent.height
                        cursorShape: Qt.SplitHCursor
                        property real startX
                        property int startW
                        onPressed: mouse => {
                            startX = mapToItem(header, mouse.x, 0).x
                            startW = root.widthOf(hcell.modelData.key)
                        }
                        onPositionChanged: mouse => {
                            if (!pressed)
                                return
                            const dx = mapToItem(header, mouse.x, 0).x - startX
                            const w = {}
                            w[hcell.modelData.key] = Math.round(Math.max(50, startW + dx))
                            root.liveWidths = w
                        }
                        onReleased: {
                            const key = hcell.modelData.key
                            if (root.liveWidths[key] !== undefined)
                                root.pane.setColumnWidth(key, root.liveWidths[key])
                            root.liveWidths = {}
                        }
                    }
                }
            }
        }
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: 1
            color: Theme.divider
        }
    }

    ListView {
        id: list

        anchors.fill: parent
        anchors.topMargin: header.height
        model: root.pane.model
        clip: true
        reuseItems: true
        cacheBuffer: 800
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.AutoFlickIfNeeded
        contentWidth: Math.max(width, root.totalWidth)
        keyNavigationEnabled: false
        currentIndex: -1
        ScrollBar.vertical: TScrollBar {}
        ScrollBar.horizontal: TScrollBar {}

        RubberBand {
            pane: root.pane
            flick: list
            hitTest: root.hitTest
        }

        delegate: Rectangle {
            id: row

            required property int index
            required property var model

            readonly property bool selected: root.pane.selection[model.path] === true
            readonly property bool isCurrent: root.pane.currentRow === index && root.pane.activeFocus && root.pane.keyboardCursor
            readonly property bool cut: root.pane.cutSet[model.path] === true

            width: Math.max(list.width, root.totalWidth)
            height: root.rowHeight
            color: selected ? (root.pane.activeFocus ? Theme.selection : Theme.selectionInactive)
                 : rmouse.containsMouse || rmouse.dropHover ? Theme.hover
                 : (root.pane.rowStripes && index % 2 ? Qt.alpha(Theme.ink, 0.025) : "transparent")
            border.width: rmouse.dropHover ? 2 : isCurrent ? 1 : 0
            border.color: rmouse.dropHover ? Theme.dropTarget : Theme.focus
            radius: 3

            Row {
                id: cells
                height: parent.height
                opacity: row.cut ? 0.45 : (row.model.isHidden ? 0.55 : 1)

                Item {
                    width: root.widthOf("name")
                    height: row.height

                    FileIcon {
                        id: nameGlyph
                        x: 8
                        anchors.verticalCenter: parent.verticalCenter
                        width: root.glyphSize + 2
                        height: root.glyphSize
                        path: row.model.path
                        glyph: row.model.glyph
                        category: row.model.category
                        size: root.glyphSize
                        thumbnail: root.pane.listThumbnails && root.pane.canThumbnail(row.model.isImage, row.model.category)
                    }
                    Text {
                        anchors.left: nameGlyph.right
                        anchors.leftMargin: 8
                        anchors.right: parent.right
                        anchors.rightMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        text: row.model.name
                        elide: Text.ElideMiddle
                        color: row.selected ? Theme.selectionText : Theme.text
                        font.family: Theme.fontFamily
                        font.pointSize: root.fontSize
                        font.italic: row.model.isLink
                    }
                }

                Repeater {
                    model: root.extraColumns
                    delegate: Text {
                        id: cell
                        required property var modelData
                        width: root.widthOf(modelData.key)
                        height: row.height
                        leftPadding: 10
                        rightPadding: 8
                        verticalAlignment: Text.AlignVCenter
                        text: row.model[modelData.role] ?? ""
                        elide: Text.ElideRight
                        horizontalAlignment: modelData.key === "size" ? Text.AlignRight : Text.AlignLeft
                        color: row.selected ? Theme.selectionText : Theme.textDim
                        font.family: Theme.fontFamily
                        font.pointSize: root.fontSize
                    }
                }
            }

            ItemMouse {
                id: rmouse
                pane: root.pane
                row: row.index
                path: row.model.path
                isDir: row.model.isDir
            }
        }
    }
}
