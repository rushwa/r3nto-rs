#!/bin/bash
set -e

echo "🎨 Building Tailwind CSS..."

# Detect platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    TAILWIND_BIN="./tools/tailwindcss-linux-x64"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(uname -m) == "arm64" ]]; then
        TAILWIND_BIN="./tools/tailwindcss-macos-arm64"
    else
        TAILWIND_BIN="./tools/tailwindcss-macos-x64"
    fi
else
    TAILWIND_BIN="./tools/tailwindcss-windows-x64.exe"
fi

# Check if binary exists
if [ ! -f "$TAILWIND_BIN" ]; then
    echo "❌ Tailwind binary not found. Download it first."
    exit 1
fi

# Build
$TAILWIND_BIN -i ./styles/input.css -o ./static/tailwind.css --minify

echo "✅ Tailwind CSS built successfully!"
ls -lh ./static/tailwind.css
