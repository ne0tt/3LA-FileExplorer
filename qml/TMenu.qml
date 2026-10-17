import QtQuick
import QtQuick.Controls
import ThreeLA.FileExplorer

Menu {
    id: m

    delegate: TMenuItem {}
    padding: 4
    overlap: 2
    font.family: Theme.fontFamily
    font.pointSize: Theme.menuFontSize

    // Separators left at an edge or beside another separator once hidden
    // items are skipped. Reading each item's visibility keeps this live.
    readonly property var looseSeparators: {
        const out = []
        let pending = null
        let seen = false
        for (let i = 0; i < count; ++i) {
            const it = itemAt(i)
            if (!it || !it.visible)
                continue
            if (it.isSeparator === true) {
                if (!seen || pending) out.push(it)
                else pending = it
            } else {
                seen = true
                pending = null
            }
        }
        if (pending) out.push(pending)
        return out
    }
    onLooseSeparatorsChanged: {
        for (let i = 0; i < count; ++i) {
            const it = itemAt(i)
            if (it && it.isSeparator === true)
                it.collapsed = looseSeparators.indexOf(it) >= 0
        }
    }

    background: Rectangle {
        // Wider for larger menu text, so labels and shortcuts still fit.
        implicitWidth: Math.round(320 * Math.max(1, Theme.menuFontSize / 10))
        color: Theme.chrome
        border.color: Theme.border
        radius: Theme.radius
    }
}
