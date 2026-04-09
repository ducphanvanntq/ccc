#!/bin/bash
set -e

CCC_HOME="$HOME/.ccc"
REPO="ducphanvanntq/ccc"

# Detect OS and arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  TARGET="x86_64-unknown-linux-gnu" ;;
    Darwin)
        case "$ARCH" in
            arm64) TARGET="aarch64-apple-darwin" ;;
            *)     TARGET="x86_64-apple-darwin" ;;
        esac
        ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

ASSET_NAME="ccc-${TARGET}"
CONFIG_ASSET="default-claude-config.zip"

echo "Fetching latest release..."
RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")

DOWNLOAD_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url.*$ASSET_NAME\"" | cut -d '"' -f 4)
CONFIG_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url.*$CONFIG_ASSET\"" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ] || [ -z "$CONFIG_URL" ]; then
    echo "Failed to get download URLs!"
    exit 1
fi

# Create install directory
mkdir -p "$CCC_HOME"

# Download binary
echo "Downloading $ASSET_NAME..."
curl -fsSL "$DOWNLOAD_URL" -o "$CCC_HOME/ccc"
chmod +x "$CCC_HOME/ccc"

# Download and extract default config
echo "Downloading default config..."
TMP_ZIP=$(mktemp)
curl -fsSL "$CONFIG_URL" -o "$TMP_ZIP"
unzip -o "$TMP_ZIP" -d "$CCC_HOME"
rm "$TMP_ZIP"

# Add to PATH
SHELL_NAME="$(basename "$SHELL")"
case "$SHELL_NAME" in
    zsh)  RC_FILE="$HOME/.zshrc" ;;
    bash) RC_FILE="$HOME/.bashrc" ;;
    *)    RC_FILE="$HOME/.profile" ;;
esac

if ! grep -q "$CCC_HOME" "$RC_FILE" 2>/dev/null; then
    echo "" >> "$RC_FILE"
    echo "# ccc - Claude Code Config" >> "$RC_FILE"
    echo "export PATH=\"\$PATH:$CCC_HOME\"" >> "$RC_FILE"
    echo "Added $CCC_HOME to PATH in $RC_FILE"
    echo "Run: source $RC_FILE  or restart your terminal."
else
    echo "$CCC_HOME is already in PATH."
fi

echo ""
echo "Done! ccc installed to $CCC_HOME"
echo "  - $CCC_HOME/ccc"
echo "  - $CCC_HOME/.claude/"
echo ""
echo "Then run: ccc key <your-api-key>"
