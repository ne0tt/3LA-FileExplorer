import QtQuick
import QtQuick.Layouts

// Buttons always sit bottom-right: the filler pushes them over.
RowLayout {
    Layout.fillWidth: true
    Layout.topMargin: 4
    spacing: 8
    Item { Layout.fillWidth: true }
}
