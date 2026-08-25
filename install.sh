#!/usr/bin/env bash

set -euo pipefail

REPO="shadowdara/seg"
BIN_NAME="seg"
INSTALL_DIR="${MYTOOL_INSTALL_DIR:-$HOME/.local/bin}"

info() {
    printf '\033[1;34m%s\033[0m\n' "$1"
}

error() {
    printf '\033[1;31m%s\033[0m\n' "$1" >&2
    exit 1
}

command -v curl >/dev/null 2>&1 || error "curl is required"
command -v tar >/dev/null 2>&1 || error "tar is required"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                ;;
            *)
                error "Unsupported Linux architecture: $ARCH"
                ;;
        esac
        ;;

    Darwin)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64|aarch64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                error "Unsupported macOS architecture: $ARCH"
                ;;
        esac
        ;;

    *)
        error "Unsupported operating system: $OS"
        ;;
esac

# Get latest GitHub release
VERSION="$(
    curl -fsSL \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/${REPO}/releases/latest" |
    sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' |
    head -n 1
)"

[ -n "$VERSION" ] || error "Could not determine latest release"

ARCHIVE="${BIN_NAME}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

info "Installing ${BIN_NAME} ${VERSION}..."
info "Target: ${TARGET}"

curl -fL --progress-bar "$URL" -o "$TMP_DIR/$ARCHIVE"

tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"

if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
    error "Binary '$BIN_NAME' not found in release archive"
fi

install -m 755 "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

info "Installed to $INSTALL_DIR/$BIN_NAME"

# Add install directory to PATH if necessary
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        ;;
    *)
        SHELL_NAME="$(basename "${SHELL:-}")"

        case "$SHELL_NAME" in
            zsh)
                PROFILE="$HOME/.zshrc"
                ;;
            bash)
                PROFILE="$HOME/.bashrc"
                ;;
            fish)
                PROFILE="$HOME/.config/fish/config.fish"
                ;;
            *)
                PROFILE=""
                ;;
        esac

        if [ -n "$PROFILE" ]; then
            mkdir -p "$(dirname "$PROFILE")"

            if ! grep -Fq "$INSTALL_DIR" "$PROFILE" 2>/dev/null; then
                if [ "$SHELL_NAME" = "fish" ]; then
                    printf '\nset -gx PATH %s $PATH\n' "$INSTALL_DIR" >> "$PROFILE"
                else
                    printf '\nexport PATH="%s:$PATH"\n' "$INSTALL_DIR" >> "$PROFILE"
                fi

                info "Added $INSTALL_DIR to $PROFILE"
            fi
        fi
        ;;
esac

printf '\n'
info "Done! 🎉"
printf 'Run: %s --help\n' "$BIN_NAME"
