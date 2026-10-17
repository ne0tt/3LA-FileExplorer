#!/bin/sh
# Build a release binary and install it plus the desktop entry for the current user.
set -eu

cd "$(dirname "$0")"
cargo build --release

bin_dir="${HOME}/.local/bin"
app_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
tpl_dir="${XDG_CONFIG_HOME:-$HOME/.config}/matugen/templates"

mkdir -p "$bin_dir" "$app_dir"
install -m 755 target/release/3la-fileexplorer "$bin_dir/3la-fileexplorer"
sed "s|@BIN@|$bin_dir/3la-fileexplorer|g" data/3LA-FileExplorer.desktop > "$app_dir/3LA-FileExplorer.desktop"
if [ -d "$tpl_dir" ]; then
    install -m 644 matugen/3la-fileexplorer-colors.json "$tpl_dir/3la-fileexplorer-colors.json"
fi
command -v update-desktop-database >/dev/null && update-desktop-database "$app_dir" || true

echo "Installed to $bin_dir/3la-fileexplorer"
case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) echo "Note: $bin_dir is not on PATH" ;;
esac
echo "Optional: make it the default folder handler with"
echo "  xdg-mime default 3LA-FileExplorer.desktop inode/directory"
