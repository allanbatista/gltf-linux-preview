#!/bin/sh
set -eu

APP_NAME="gltf-linux-preview"

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT_DIR"

PREFIX="${PREFIX:-$HOME/.local}"
APP_ROOT="$PREFIX/share/$APP_NAME"
BIN_DIR="$PREFIX/bin"
APPLICATIONS_DIR="$PREFIX/share/applications"
THUMBNAILERS_DIR="$PREFIX/share/thumbnailers"
DESKTOP_FILE="$APPLICATIONS_DIR/$APP_NAME.desktop"
THUMBNAILER_FILE="$THUMBNAILERS_DIR/$APP_NAME.thumbnailer"
BIN_PATH="$APP_ROOT/$APP_NAME"
LAUNCHER_PATH="$BIN_DIR/$APP_NAME"
TARGET_BIN="$ROOT_DIR/target/release/$APP_NAME"

if [ "$(id -u)" -ne 0 ]; then
    cargo build --release
fi

if [ ! -x "$TARGET_BIN" ]; then
    printf 'missing build output: %s\n' "$TARGET_BIN" >&2
    if [ "$(id -u)" -eq 0 ]; then
        printf 'run cargo build --release without sudo first\n' >&2
    fi
    exit 1
fi

if [ ! -d "$ROOT_DIR/assets" ]; then
    printf 'missing assets dir: %s\n' "$ROOT_DIR/assets" >&2
    exit 1
fi

mkdir -p "$APP_ROOT" "$BIN_DIR" "$APPLICATIONS_DIR" "$THUMBNAILERS_DIR"
install -m 755 "$TARGET_BIN" "$BIN_PATH"
rm -rf "$APP_ROOT/assets"
cp -R "$ROOT_DIR/assets" "$APP_ROOT/"

cat > "$LAUNCHER_PATH" <<EOF
#!/bin/sh
set -eu
APP_ROOT="$APP_ROOT"
if [ "$#" -eq 0 ]; then
    cd -- "$APP_ROOT"
fi
exec "$APP_ROOT/$APP_NAME" "\$@"
EOF

chmod 755 "$LAUNCHER_PATH"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=GLTF Preview
Comment=Preview glTF, GLB, and OBJ files
Exec=$LAUNCHER_PATH %F
TryExec=$LAUNCHER_PATH
Icon=applications-graphics
Terminal=false
Categories=Graphics;3DGraphics;Viewer;
MimeType=model/gltf+json;model/gltf-binary;model/obj;
StartupNotify=false
EOF

case "$PREFIX" in
    /usr|/usr/*)
        cat > "$THUMBNAILER_FILE" <<EOF
[Thumbnailer Entry]
TryExec=$LAUNCHER_PATH
Exec=/usr/bin/env RUST_LOG=error "$LAUNCHER_PATH" --thumbnail %i %o %s
MimeType=model/gltf+json;model/gltf-binary;model/obj;
EOF
        chmod 644 "$THUMBNAILER_FILE"

        if [ "$(id -u)" -eq 0 ] && [ -n "${SUDO_USER:-}" ] && command -v getent >/dev/null 2>&1; then
            USER_HOME=$(getent passwd "$SUDO_USER" 2>/dev/null | cut -d: -f6)
            LEGACY_THUMBNAILER="$USER_HOME/.local/share/thumbnailers/$APP_NAME.thumbnailer"
            if [ -n "$USER_HOME" ] && [ -f "$LEGACY_THUMBNAILER" ]; then
                rm -f "$LEGACY_THUMBNAILER"
            fi
        fi
        ;;
    *)
        rm -f "$THUMBNAILER_FILE"
        ;;
esac

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
fi

printf 'Installed %s in %s\n' "$APP_NAME" "$PREFIX"
