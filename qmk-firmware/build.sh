#!/usr/bin/env bash
# Build the Eight-Switch QMK macropad firmware.
#
# Usage:
#   ./build.sh [keymap ...]          # default: "default via"
#
# Environment:
#   QMK_HOME   path to a qmk_firmware checkout (with submodules).
#              Defaults to ./.qmk_firmware, cloned on first run.
#   QMK_REF    qmk_firmware tag/branch to clone (default below).
#
# Output: build/eight-switch-qmk-<keymap>.uf2
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QMK_REF="${QMK_REF:-0.34.8}"
QMK_HOME="${QMK_HOME:-$HERE/.qmk_firmware}"
KEYMAPS=("$@")
[ ${#KEYMAPS[@]} -eq 0 ] && KEYMAPS=(default via)

if [ ! -d "$QMK_HOME/quantum" ]; then
    echo "==> Cloning qmk_firmware $QMK_REF into $QMK_HOME"
    git clone --depth 1 --branch "$QMK_REF" https://github.com/qmk/qmk_firmware.git "$QMK_HOME"
fi

echo "==> Updating qmk_firmware submodules"
(cd "$QMK_HOME" && git submodule update --init --recursive --depth 1)

# QMK only builds keyboards that live inside its own tree, so copy ours in.
echo "==> Syncing keyboards/n8ech into $QMK_HOME"
rm -rf "$QMK_HOME/keyboards/n8ech"
cp -R "$HERE/keyboards/n8ech" "$QMK_HOME/keyboards/n8ech"

mkdir -p "$HERE/build"
for km in "${KEYMAPS[@]}"; do
    echo "==> Compiling keymap: $km"
    make -C "$QMK_HOME" "n8ech/eight_switch:$km"
    cp "$QMK_HOME/n8ech_eight_switch_${km}.uf2" "$HERE/build/eight-switch-qmk-${km}.uf2"
done

echo "==> Done:"
ls -1 "$HERE/build"/*.uf2
