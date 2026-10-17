import QtQuick
import QtQuick.Controls
import QtMultimedia
import ThreeLA.FileExplorer

// Space-bar quick look for pictures and videos, framed like the dialogs.
// Left/Right step through the folder's other pictures and videos.
Popup {
    id: d

    property var items: []          // [{ path, name, video }]
    property int index: -1
    readonly property var current: index >= 0 && index < items.length ? items[index] : null
    readonly property bool isVideo: !!current && current.video

    function show(pane, path) {
        // One call for the whole list instead of three per row.
        const list = JSON.parse(pane.model.mediaItems())
        items = list
        index = list.findIndex(i => i.path === path)
        if (index >= 0) open()
    }

    function step(delta) {
        if (!items.length) return
        player.stop()
        index = (index + delta + items.length) % items.length
    }

    function fmt(ms) {
        const s = Math.floor(ms / 1000)
        return Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0")
    }

    onOpened: {
        scope.forceActiveFocus()
        if (isVideo) player.play()
    }
    // Release the decoded picture (and every cached GIF frame) and the
    // player's media while hidden.
    onClosed: {
        player.stop()
        index = -1
        items = []
    }

    parent: Overlay.overlay
    x: 24
    y: 24
    width: parent ? parent.width - 48 : 800
    height: parent ? parent.height - 48 : 600
    modal: true
    focus: true
    padding: 0
    closePolicy: Popup.CloseOnEscape

    Overlay.modal: Rectangle { color: "#d0000000" }

    background: SpxCard {
        id: card
        header: "PREVIEW"
        tag: d.isVideo ? "VIDEO" : "IMAGE"
    }

    contentItem: FocusScope {
        id: scope
        focus: true

        Keys.onPressed: event => {
            switch (event.key) {
            case Qt.Key_Space:
                if (d.isVideo)
                    player.playbackState === MediaPlayer.PlayingState ? player.pause() : player.play()
                else
                    d.close()
                break
            case Qt.Key_Left:
            case Qt.Key_Up:
                d.step(-1)
                break
            case Qt.Key_Right:
            case Qt.Key_Down:
                d.step(1)
                break
            case Qt.Key_Home:
                d.index = 0
                break
            case Qt.Key_End:
                d.index = d.items.length - 1
                break
            default:
                return
            }
            event.accepted = true
        }

        // File name, position and actions under the title bar.
        Row {
            id: info
            x: 18
            y: card.headerHeight + 6
            spacing: 14
            TText {
                text: d.current ? d.current.name : ""
                font.bold: true
                elide: Text.ElideMiddle
                width: Math.min(implicitWidth, scope.width - 360)
            }
            TText {
                text: (d.index + 1) + " / " + d.items.length
                color: Theme.textFaint
            }
        }
        Row {
            anchors.right: parent.right
            anchors.rightMargin: 14
            y: card.headerHeight + 2
            spacing: 4
            GlyphButton { glyph: G.back; tip: "Previous (Left)"; enabled: d.items.length > 1; onClicked: d.step(-1) }
            GlyphButton { glyph: G.forward; tip: "Next (Right)"; enabled: d.items.length > 1; onClicked: d.step(1) }
            GlyphButton {
                glyph: G.openNew
                tip: "Open with default application"
                onClicked: {
                    Sys.openPaths([d.current.path])
                    d.close()
                }
            }
            GlyphButton { glyph: G.close; tip: "Close (Esc)"; onClicked: d.close() }
        }

        Item {
            id: stage
            anchors.fill: parent
            anchors.topMargin: card.headerHeight + 40
            anchors.leftMargin: 18
            anchors.rightMargin: 18
            anchors.bottomMargin: d.isVideo ? 64 : 18
            clip: true

            AnimatedImage {
                id: picture
                anchors.fill: parent
                visible: !d.isVideo
                source: d.current && !d.isVideo ? Sys.toUri(d.current.path) : ""
                // Never upscale small pictures past their own size.
                fillMode: implicitWidth <= stage.width && implicitHeight <= stage.height ? Image.Pad : Image.PreserveAspectFit
                asynchronous: true
                autoTransform: true
                smooth: true
                mipmap: true
                cache: false
                sourceSize.width: Math.max(1, stage.width * 2)
            }
            TText {
                anchors.centerIn: parent
                visible: !d.isVideo && picture.status === Image.Error
                text: "Cannot display this image"
                color: Theme.danger
            }

            VideoOutput {
                id: video
                anchors.fill: parent
                visible: d.isVideo
            }
        }

        MediaPlayer {
            id: player
            source: d.current && d.isVideo ? Sys.toUri(d.current.path) : ""
            videoOutput: video
            audioOutput: AudioOutput { id: audio }
            // Start each video once its source is in place (stepping to it
            // with Left/Right changes the source after `current`).
            onSourceChanged: if (source.toString() !== "" && d.opened) play()
        }

        // Video transport: play/pause, seek bar, time, mute.
        Row {
            visible: d.isVideo
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 18
            height: 32
            spacing: 10

            GlyphButton {
                glyph: player.playbackState === MediaPlayer.PlayingState ? "\u{F03E4}" : "\u{F040A}"
                tip: "Play/Pause (Space)"
                onClicked: player.playbackState === MediaPlayer.PlayingState ? player.pause() : player.play()
            }
            Item {
                id: seek
                width: parent.width - 32 - 130 - 32 - 30
                height: parent.height
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width
                    height: 4
                    color: Theme.border
                    Rectangle {
                        width: player.duration > 0 ? parent.width * player.position / player.duration : 0
                        height: parent.height
                        color: Theme.primary
                    }
                }
                MouseArea {
                    anchors.fill: parent
                    onPressed: mouse => player.setPosition(player.duration * mouse.x / width)
                    onPositionChanged: mouse => { if (pressed) player.setPosition(player.duration * Math.max(0, Math.min(1, mouse.x / width))) }
                }
            }
            TText {
                anchors.verticalCenter: parent.verticalCenter
                width: 130
                horizontalAlignment: Text.AlignRight
                text: d.fmt(player.position) + " / " + d.fmt(player.duration)
                color: Theme.textDim
            }
            GlyphButton {
                glyph: audio.muted ? "\u{F0581}" : "\u{F057E}"
                tip: "Mute"
                onClicked: audio.muted = !audio.muted
            }
        }
    }
}
