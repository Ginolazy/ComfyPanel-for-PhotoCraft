#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TURBO_SRC="$ROOT_DIR/src-turbo-bar"
TURBO_TARGET="$ROOT_DIR/src/turbo"

cleanup() {
    if [ -d "$TURBO_TARGET" ]; then
        echo "[build] Cleaning up temporary turbo assets..."
        rm -rf "$TURBO_TARGET"
    fi
}
trap cleanup EXIT

if [ ! -d "$TURBO_TARGET" ] && [ -d "$TURBO_SRC" ]; then
    echo "[build] Injecting temporary turbo assets..."
    cp -r "$TURBO_SRC" "$TURBO_TARGET"
fi

npm run tauri -- "$@"
python3 build_photocraft.py --skip-build
