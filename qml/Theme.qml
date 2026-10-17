pragma Singleton

import QtQuick
import ThreeLA.FileExplorer

// Colours come from ~/.config/3la-fileexplorer/colors.json, written by the
// matugen template in matugen/ and live-reloaded by Sys on every wallpaper change.
QtObject {
    id: theme

    readonly property var c: {
        try {
            return JSON.parse(Sys.themeJson || "{}")
        } catch (e) {
            return {}
        }
    }

    function role(name, fallback) {
        const v = c[name]
        return (typeof v === "string" && v.length > 0) ? v : fallback
    }

    readonly property bool dark: role("mode", "dark") !== "light"

    readonly property color background: role("background", "#0d1515")
    readonly property color surface: role("surface", "#0d1515")
    readonly property color surfaceLowest: role("surface_container_lowest", "#080f0f")
    readonly property color surfaceLow: role("surface_container_low", "#151d1d")
    readonly property color surfaceContainer: role("surface_container", "#192121")
    readonly property color surfaceHigh: role("surface_container_high", "#242b2b")
    readonly property color surfaceHighest: role("surface_container_highest", "#2e3636")
    readonly property color ink: role("on_surface", "#dce4e3")
    readonly property color inkVariant: role("on_surface_variant", "#bacac9")
    readonly property color primary: role("primary", "#b4fffe")
    readonly property color primaryContainer: role("primary_container", "#2decec")
    readonly property color secondary: role("secondary", "#98d1d0")
    readonly property color tertiary: role("tertiary", "#f4efff")
    readonly property color tertiaryContainer: role("tertiary_container", "#d5d0ff")
    readonly property color error: role("error", "#ffb4ab")
    readonly property color outline: role("outline", "#849493")
    readonly property color outlineVariant: role("outline_variant", "#3b4949")
    readonly property color scrim: role("scrim", "#000000")
    readonly property color main: role("main", "#0c3a3a")
    readonly property color accent: role("accent", "#690005")
    readonly property color accent2: role("accent2", "#b38000")

    // Semantic roles used by the UI.
    readonly property color window: surface
    readonly property color chrome: main
    readonly property color sidebar: background
    readonly property color view: background
    readonly property color text: ink
    readonly property color textDim: inkVariant
    readonly property color textFaint: outline
    readonly property color border: outlineVariant
    // Dividers share the toolbar colour; lines drawn on chrome-coloured
    // surfaces (menus, toolbar, column headers) use a lighter tint of it.
    readonly property color divider: chrome
    readonly property color dividerOnChrome: Qt.lighter(chrome, 1.6)
    readonly property color hover: Qt.alpha(ink, 0.07)
    readonly property color pressed: Qt.alpha(ink, 0.12)
    // Selection uses the matugen accent; text on it is black or white,
    // whichever contrasts (as on the dialog title bars).
    readonly property color selection: accent
    readonly property color selectionText: (0.299 * accent.r + 0.587 * accent.g + 0.114 * accent.b) > 0.55 ? "#000000" : "#ffffff"
    readonly property color selectionInactive: Qt.alpha(accent, 0.55)
    readonly property color focus: primary
    readonly property color dropTarget: accent2
    readonly property color danger: error

    readonly property color success: "#9ef0a8"
    readonly property color cryptRed: "#ff5c5c"

    // File icon colours, as in SPX-FileExplorer: folders in the wallpaper's
    // source colour, everything else in a lightened primary.
    readonly property color folderIcon: role("source_color", primaryContainer)
    readonly property color fileIcon: role("primary_light", primary)

    function categoryColor(cat) {
        return cat === "folder" ? folderIcon : fileIcon
    }

    readonly property string fontFamily: "JetBrainsMono Nerd Font"
    // Propo has true advance widths for icons, so glyphs centre exactly.
    readonly property string glyphFamily: "JetBrainsMono Nerd Font Propo"
    // Embedded in the binary (fonts/ChakraPetch-SemiBold.ttf).
    readonly property string displayFamily: "Chakra Petch"
    // Set from the Font size preference (Main.qml binds it).
    property int fontSize: 10
    readonly property int smallFontSize: Math.max(6, fontSize - 1)
    // Menus follow the "Sidebar & lists" size (also bound from Main.qml).
    property int menuFontSize: fontSize
    // Chamfered 3LA-MilSpec dialog frames; square when off (bound from Main.qml).
    property bool milSpec: true

    // A height designed for 10 pt text, grown to fit larger text (never shrunk).
    function grow(px, size) {
        return Math.round(px + Math.max(0, (size === undefined ? fontSize : size) - 10) * 1.6)
    }
    readonly property int radius: 6
    readonly property int smallRadius: 4
    readonly property int animMs: 120
}
