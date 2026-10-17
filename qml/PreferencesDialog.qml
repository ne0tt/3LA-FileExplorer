import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ThreeLA.FileExplorer

// Preferences split into sections picked from a tab column on the left. The
// page area is as tall as the tallest section, so switching tabs never
// resizes the card; it only scrolls when the window is too short for that.
SpxDialog {
    id: d

    required property var prefs
    property int tab: 0

    readonly property var tabs: [
        { name: "View", glyph: "\u{F11D9}" },
        { name: "Appearance", glyph: "\u{F0E0C}" },
        { name: "Dates", glyph: "\u{F00F0}" },
        { name: "Behaviour", glyph: "\u{F0CFE}" }
    ]

    header: "PREFERENCES"
    tag: "CONFIG"
    cardWidth: 680
    buttonRow: footer

    Shortcut {
        sequences: ["Ctrl+Tab", "Ctrl+PgDown"]
        enabled: d.visible
        onActivated: d.tab = (d.tab + 1) % d.tabs.length
    }
    Shortcut {
        sequences: ["Ctrl+Shift+Tab", "Ctrl+PgUp"]
        enabled: d.visible
        onActivated: d.tab = (d.tab + d.tabs.length - 1) % d.tabs.length
    }

    component Heading: SpxLabel {
        Layout.topMargin: 6
        color: Theme.primary
    }

    component Check: SpxCheck {
        Layout.fillWidth: true
    }

    // Label, −/value/+ and an optional reset button on one line.
    component Stepper: RowLayout {
        id: st
        property string label
        property string value
        property bool canDec: true
        property bool canInc: true
        property string resetText: "Reset"
        property bool resettable: false
        signal dec()
        signal inc()
        signal reset()

        spacing: 8
        SpxLabel { text: st.label; Layout.preferredWidth: 170 }
        SpxButton { text: "−"; implicitWidth: 32; implicitHeight: 24; enabled: st.canDec; onClicked: st.dec() }
        SpxText {
            text: st.value
            color: Theme.text
            horizontalAlignment: Text.AlignHCenter
            Layout.preferredWidth: 56
        }
        SpxButton { text: "+"; implicitWidth: 32; implicitHeight: 24; enabled: st.canInc; onClicked: st.inc() }
        SpxButton { text: st.resetText; implicitHeight: 24; visible: st.resettable; onClicked: st.reset() }
        Item { Layout.fillWidth: true }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 0

        Column {
            Layout.alignment: Qt.AlignTop
            Layout.preferredWidth: 150
            spacing: 2

            Repeater {
                model: d.tabs
                SpxRow {
                    id: tabRow
                    required property var modelData
                    required property int index
                    width: parent.width
                    height: 32
                    current: d.tab === index
                    hot: tabMa.containsMouse

                    Rectangle {
                        width: 2
                        height: parent.height
                        color: Theme.primary
                        visible: tabRow.current
                    }
                    Row {
                        anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                        spacing: 10
                        Text {
                            text: tabRow.modelData.glyph
                            color: tabRow.current ? Theme.primary : Theme.textDim
                            font.family: Theme.glyphFamily
                            font.pixelSize: 15
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        Text {
                            text: tabRow.modelData.name
                            color: tabRow.current ? Theme.primary : Theme.text
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    MouseArea {
                        id: tabMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: d.tab = tabRow.index
                    }
                }
            }
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.leftMargin: 12
            Layout.rightMargin: 18
            implicitWidth: 1
            color: Theme.border
        }

        Flickable {
            id: flick
            // Header, footer and margins take roughly this much of the window.
            readonly property real room: (d.parent ? d.parent.height : 800) - 200
            readonly property bool scrolls: contentHeight > height + 1

            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(stack.implicitHeight, Math.max(160, room))
            contentWidth: width
            contentHeight: stack.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            clip: true
            ScrollBar.vertical: TScrollBar { policy: flick.scrolls ? ScrollBar.AlwaysOn : ScrollBar.AlwaysOff }

            StackLayout {
                id: stack
                width: flick.width - (flick.scrolls ? 12 : 0)
                currentIndex: d.tab

                // ── View ──────────────────────────────────────────────
                ColumnLayout {
                    spacing: 6
                    SpxTitle { text: "View" }
                    RowLayout {
                        spacing: 8
                        SpxLabel { text: "Default view"; Layout.preferredWidth: 170 }
                        Repeater {
                            model: ["Icons", "Details", "Compact"]
                            SpxButton {
                                required property string modelData
                                required property int index
                                text: modelData
                                implicitHeight: 24
                                selected: d.prefs.viewMode === index
                                onClicked: d.prefs.viewMode = index
                            }
                        }
                    }
                    Stepper {
                        label: "Grid spacing"
                        value: (d.prefs.gridSpacing > 0 ? "+" : "") + d.prefs.gridSpacing + " px"
                        canDec: d.prefs.gridSpacing > -16
                        canInc: d.prefs.gridSpacing < 48
                        resettable: d.prefs.gridSpacing !== 0
                        onDec: d.prefs.gridSpacing = Math.max(-16, d.prefs.gridSpacing - 2)
                        onInc: d.prefs.gridSpacing = Math.min(48, d.prefs.gridSpacing + 2)
                        onReset: d.prefs.gridSpacing = 0
                    }
                    Heading { text: "Items" }
                    Check { text: "Sort folders before files"; checked: d.prefs.foldersFirst; onToggled: d.prefs.foldersFirst = checked }
                    Check { text: "Show hidden files"; checked: d.prefs.showHidden; onToggled: d.prefs.showHidden = checked }
                    Check { text: "Show number of items in folder size column"; checked: d.prefs.countChildren; onToggled: d.prefs.countChildren = checked }
                    Heading { text: "Thumbnails" }
                    Check { text: "Show thumbnails for pictures and videos"; checked: d.prefs.showThumbnails; onToggled: d.prefs.showThumbnails = checked }
                    Check {
                        text: "Use thumbnails as icons in list views"
                        enabled: d.prefs.showThumbnails
                        leftPadding: 22
                        checked: d.prefs.listThumbnails
                        onToggled: d.prefs.listThumbnails = checked
                    }
                    Item { Layout.fillHeight: true }
                }

                // ── Appearance ────────────────────────────────────────
                ColumnLayout {
                    spacing: 6
                    SpxTitle { text: "Appearance" }
                    Heading { text: "Text" }
                    Stepper {
                        label: "Font size"
                        value: d.prefs.fontSize + " pt"
                        canDec: d.prefs.fontSize > 7
                        canInc: d.prefs.fontSize < 20
                        resettable: d.prefs.fontSize !== 10
                        onDec: d.prefs.fontSize = Math.max(7, d.prefs.fontSize - 1)
                        onInc: d.prefs.fontSize = Math.min(20, d.prefs.fontSize + 1)
                        onReset: d.prefs.fontSize = 10
                    }
                    // 0 means "same as the main font size"; +/- start from that.
                    Stepper {
                        readonly property int current: d.prefs.sidebarFontSize > 0 ? d.prefs.sidebarFontSize : d.prefs.fontSize
                        label: "Sidebar, lists & menus"
                        value: d.prefs.sidebarFontSize > 0 ? d.prefs.sidebarFontSize + " pt" : "= main"
                        canDec: current > 7
                        canInc: current < 20
                        resetText: "Same as main"
                        resettable: d.prefs.sidebarFontSize > 0
                        onDec: d.prefs.sidebarFontSize = Math.max(7, current - 1)
                        onInc: d.prefs.sidebarFontSize = Math.min(20, current + 1)
                        onReset: d.prefs.sidebarFontSize = 0
                    }
                    Heading { text: "Lists" }
                    Check { text: "Alternate row colours in detailed list"; checked: d.prefs.rowStripes; onToggled: d.prefs.rowStripes = checked }
                    Heading { text: "Dialogs" }
                    Check { text: "3LA-MilSpec borders (chamfered corners)"; checked: d.prefs.milSpecBorders; onToggled: d.prefs.milSpecBorders = checked }
                    Heading { text: "Theme" }
                    SpxText {
                        Layout.fillWidth: true
                        color: Theme.textFaint
                        text: "Colours follow matugen live via ~/.config/3la-fileexplorer/colors.json. "
                            + "Run `matugen` (or change wallpaper) to regenerate."
                    }
                    Item { Layout.fillHeight: true }
                }

                // ── Dates ─────────────────────────────────────────────
                ColumnLayout {
                    spacing: 6
                    SpxTitle { text: "Dates" }
                    Heading { text: "Date format" }
                    Repeater {
                        model: [
                            { label: "dd/mm/yyyy HH:mm", fmt: "%d/%m/%Y %H:%M" },
                            { label: "yyyy-mm-dd HH:mm", fmt: "%Y-%m-%d %H:%M" },
                            { label: "mm/dd/yyyy hh:mm AM", fmt: "%m/%d/%Y %I:%M %p" },
                            { label: "dd Mon yyyy HH:mm", fmt: "%d %b %Y %H:%M" },
                            { label: "Relative (Today, Yesterday…)", fmt: "relative" }
                        ]
                        SpxRow {
                            id: fmtRow
                            required property var modelData
                            Layout.fillWidth: true
                            implicitHeight: 26
                            current: d.prefs.dateFormat === modelData.fmt
                            hot: fmtMa.containsMouse
                            RowLayout {
                                anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                                SpxText { text: fmtRow.modelData.label; color: fmtRow.current ? Theme.primary : Theme.text; wrapMode: Text.NoWrap }
                                SpxLabel { text: Sys.previewDate(fmtRow.modelData.fmt) }
                            }
                            MouseArea {
                                id: fmtMa
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: {
                                    d.prefs.dateFormat = fmtRow.modelData.fmt
                                    customFmt.text = fmtRow.modelData.fmt
                                }
                            }
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.topMargin: 4
                        spacing: 10
                        SpxLabel { text: "Custom" }
                        SpxField {
                            id: customFmt
                            Layout.fillWidth: true
                            text: d.prefs.dateFormat
                            placeholderText: "strftime pattern, e.g. %d/%m/%Y %H:%M"
                            onTextEdited: if (Sys.previewDate(text)) d.prefs.dateFormat = text
                        }
                    }
                    SpxText {
                        readonly property string sample: Sys.previewDate(customFmt.text)
                        text: sample ? "Preview: " + sample : "Invalid pattern (see `man strftime`)"
                        color: sample ? Theme.textFaint : Theme.danger
                        font.pixelSize: 11
                    }
                    Item { Layout.fillHeight: true }
                }

                // ── Behaviour ─────────────────────────────────────────
                ColumnLayout {
                    spacing: 6
                    SpxTitle { text: "Behaviour" }
                    Heading { text: "Navigation" }
                    Check { text: "Single click to activate items"; checked: d.prefs.singleClick; onToggled: d.prefs.singleClick = checked }
                    Check { text: "Always show the tab bar"; checked: d.prefs.alwaysShowTabs; onToggled: d.prefs.alwaysShowTabs = checked }
                    Heading { text: "Trash & delete" }
                    Check { text: "Ask before moving files to the Trash"; checked: d.prefs.confirmTrash; onToggled: d.prefs.confirmTrash = checked }
                    Check { text: "Show a 'Delete' action that bypasses the Trash in menus"; checked: d.prefs.showDelete; onToggled: d.prefs.showDelete = checked }
                    Item { Layout.fillHeight: true }
                }
            }
        }
    }

    SpxFooter {
        id: footer
        SpxButton { text: "Close"; onClicked: d.close() }
    }
}
