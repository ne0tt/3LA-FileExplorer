import QtQuick
import ThreeLA.FileExplorer

// A file's glyph, replaced by its picture/video thumbnail once one has loaded.
Item {
    id: icon

    property string path: ""
    property string glyph: ""
    property string category: ""
    property int size: 16
    // Glyph size relative to `size` (thumbnails always fill it).
    property real glyphScale: 1
    // Whether to try for a thumbnail at all (setting on, picture or video).
    property bool thumbnail: false

    implicitWidth: size
    implicitHeight: size

    Glyph {
        anchors.centerIn: parent
        visible: !(thumb.item && thumb.item.status === Image.Ready)
        text: icon.glyph
        size: Math.round(icon.size * icon.glyphScale)
        color: Theme.categoryColor(icon.category)
    }

    // Only thumbnailable files get an Image; sourceSize is 2x for crispness,
    // so no mipmaps are needed.
    Loader {
        id: thumb
        anchors.fill: parent
        active: icon.thumbnail && icon.path !== ""
        sourceComponent: Image {
            source: "image://thumb/" + encodeURIComponent(icon.path)
            sourceSize: Qt.size(icon.size * 2, icon.size * 2)
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            smooth: true
            visible: status === Image.Ready
        }
    }
}
