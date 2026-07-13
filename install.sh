#!/usr/bin/env bash
set -euo pipefail

APP_NAME="gltf-linux-preview"
REPO="${REPO:-allanbatista/gltf-linux-preview}"
PREFIX="${PREFIX:-$HOME/.local}"

usage() {
    cat <<'EOF'
Usage: install.sh [--prefix PATH] [--repo OWNER/REPO]

Install gltf-linux-preview from the latest GitHub release.

Environment variables:
  PREFIX  Install prefix (default: ~/.local)
  REPO    GitHub repository (default: allanbatista/gltf-linux-preview)
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            PREFIX="${2:?missing value for --prefix}"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --repo)
            REPO="${2:?missing value for --repo}"
            shift 2
            ;;
        --repo=*)
            REPO="${1#*=}"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

for cmd in curl tar sha256sum cp rm mkdir install uname chmod mktemp; do
    command -v "$cmd" >/dev/null 2>&1 || {
        printf 'missing required command: %s\n' "$cmd" >&2
        exit 1
    }
done

case "$(uname -m)" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        printf 'unsupported architecture: %s\n' "$(uname -m)" >&2
        exit 1
        ;;
esac

ASSET_BASE_URL="https://github.com/${REPO}/releases/latest/download"
ARCHIVE_NAME="${APP_NAME}-linux-${ARCH}.tar.gz"
CHECKSUM_NAME="${ARCHIVE_NAME}.sha256"
TMP_DIR="$(mktemp -d)"

trap 'rm -rf "$TMP_DIR"' EXIT INT TERM HUP

curl -fsSL "${ASSET_BASE_URL}/${ARCHIVE_NAME}" -o "${TMP_DIR}/${ARCHIVE_NAME}"
curl -fsSL "${ASSET_BASE_URL}/${CHECKSUM_NAME}" -o "${TMP_DIR}/${CHECKSUM_NAME}"

(
    cd "$TMP_DIR"
    sha256sum -c "$CHECKSUM_NAME"
)

tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"

SOURCE_DIR="${TMP_DIR}/${APP_NAME}"
APP_ROOT="${PREFIX}/share/${APP_NAME}"
BIN_DIR="${PREFIX}/bin"
APPLICATIONS_DIR="${PREFIX}/share/applications"
THUMBNAILERS_DIR="${PREFIX}/share/thumbnailers"
LAUNCHER_PATH="${BIN_DIR}/${APP_NAME}"
DESKTOP_FILE="${APPLICATIONS_DIR}/${APP_NAME}.desktop"
THUMBNAILER_FILE="${THUMBNAILERS_DIR}/${APP_NAME}.thumbnailer"

if [ ! -x "${SOURCE_DIR}/${APP_NAME}" ]; then
    printf 'missing installed binary: %s\n' "${SOURCE_DIR}/${APP_NAME}" >&2
    exit 1
fi

rm -rf "$APP_ROOT"
install -d "$APP_ROOT" "$BIN_DIR" "$APPLICATIONS_DIR" "$THUMBNAILERS_DIR"
cp -R "${SOURCE_DIR}/." "$APP_ROOT/"

cat > "$LAUNCHER_PATH" <<EOF
#!/bin/sh
set -eu
APP_ROOT="$APP_ROOT"
if [ "\$#" -eq 0 ]; then
    cd -- "\$APP_ROOT"
fi
exec "\$APP_ROOT/$APP_NAME" "\$@"
EOF

chmod 755 "$LAUNCHER_PATH"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=GLTF Preview
Comment=Preview glTF, GLB, and OBJ files
Exec="$LAUNCHER_PATH" %F
TryExec=$LAUNCHER_PATH
Icon=applications-graphics
Terminal=false
Categories=Graphics;3DGraphics;Viewer;
MimeType=model/gltf+json;model/gltf-binary;model/obj;
StartupNotify=false
EOF

cat > "$THUMBNAILER_FILE" <<EOF
[Thumbnailer Entry]
TryExec=$LAUNCHER_PATH
Exec=/usr/bin/env RUST_LOG=error "$LAUNCHER_PATH" --thumbnail %i %o %s
MimeType=model/gltf+json;model/gltf-binary;model/obj;
EOF
chmod 644 "$THUMBNAILER_FILE"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
fi

printf 'Installed %s in %s\n' "$APP_NAME" "$PREFIX"
