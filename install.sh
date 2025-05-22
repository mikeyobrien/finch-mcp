#!/bin/bash
set -e

# finch-mcp installer script
# Usage: curl -sSL https://raw.githubusercontent.com/mikeyobrien/finch-mcp/main/install.sh | bash

REPO="mikeyobrien/finch-mcp"
BINARY_NAME="finch-mcp"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    "Darwin")
        if [ "$ARCH" = "arm64" ]; then
            PLATFORM="macos-aarch64"
            EXTENSION="tar.gz"
        else
            PLATFORM="macos-x86_64"
            EXTENSION="tar.gz"
        fi
        ;;
    "Linux")
        if [ "$ARCH" = "x86_64" ]; then
            PLATFORM="linux-x86_64"
            EXTENSION="tar.gz"
        elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
            PLATFORM="linux-aarch64"
            EXTENSION="tar.gz"
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    "MINGW"*|"MSYS"*|"CYGWIN"*)
        PLATFORM="windows-x86_64.exe"
        EXTENSION="zip"
        BINARY_NAME="finch-mcp.exe"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Get the latest release tag
echo "🔍 Fetching latest release..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "❌ Failed to fetch latest release tag"
    exit 1
fi

echo "📦 Latest version: $LATEST_TAG"

# Construct download URL
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/${BINARY_NAME}-${PLATFORM}.${EXTENSION}"

echo "⬇️  Downloading $DOWNLOAD_URL"

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Download the binary
if command -v curl >/dev/null 2>&1; then
    curl -sL "$DOWNLOAD_URL" -o "finch-mcp.${EXTENSION}"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "finch-mcp.${EXTENSION}"
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Extract the binary
echo "📂 Extracting binary..."
if [ "$EXTENSION" = "tar.gz" ]; then
    tar -xzf "finch-mcp.${EXTENSION}"
elif [ "$EXTENSION" = "zip" ]; then
    unzip -q "finch-mcp.${EXTENSION}"
fi

# Make binary executable (Unix systems)
if [ "$OS" != "MINGW"* ] && [ "$OS" != "MSYS"* ] && [ "$OS" != "CYGWIN"* ]; then
    chmod +x "$BINARY_NAME"
fi

# Install to system PATH
echo "📥 Installing to system..."

if [ "$OS" = "Darwin" ] || [ "$OS" = "Linux" ]; then
    # Try to install to /usr/local/bin (requires sudo)
    if [ -w "/usr/local/bin" ]; then
        mv "$BINARY_NAME" "/usr/local/bin/"
        echo "✅ Installed to /usr/local/bin/$BINARY_NAME"
    elif command -v sudo >/dev/null 2>&1; then
        sudo mv "$BINARY_NAME" "/usr/local/bin/"
        echo "✅ Installed to /usr/local/bin/$BINARY_NAME (with sudo)"
    else
        # Fallback to user bin directory
        mkdir -p "$HOME/.local/bin"
        mv "$BINARY_NAME" "$HOME/.local/bin/"
        echo "✅ Installed to $HOME/.local/bin/$BINARY_NAME"
        echo "⚠️  Make sure $HOME/.local/bin is in your PATH"
        echo "   Add this to your shell profile: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
else
    # Windows: Install to current directory
    mv "$BINARY_NAME" "../$BINARY_NAME"
    echo "✅ Downloaded $BINARY_NAME to current directory"
    echo "   Move it to a directory in your PATH to use globally"
fi

# Cleanup
cd ..
rm -rf "$TMP_DIR"

echo ""
echo "🎉 finch-mcp $LATEST_TAG installed successfully!"
echo ""
echo "Usage examples:"
echo "  finch-mcp uvx mcp-server-time"
echo "  finch-mcp ./my-project"
echo "  finch-mcp https://github.com/user/mcp-repo"
echo ""
echo "For more information, visit: https://github.com/$REPO"