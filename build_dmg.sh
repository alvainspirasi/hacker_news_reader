#!/bin/bash

# Build DMG script for Hacker News Reader
set -e

APP_NAME="Hacker News Reader"
BINARY_NAME="hacker_news_reader"
VERSION="1.0.0"
DMG_NAME="HackerNewsReader-${VERSION}"

echo "Building DMG for ${APP_NAME}..."

# Build the release binary
echo "Building release binary..."
cargo build --release

# Create temporary directory structure
echo "Creating app bundle structure..."
TEMP_DIR=$(mktemp -d)
APP_DIR="${TEMP_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy the binary
echo "Copying binary..."
cp "target/release/${BINARY_NAME}" "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/${APP_NAME}"

# Create Info.plist
echo "Creating Info.plist..."
cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.haojiang99.hackernewsreader</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
</dict>
</plist>
EOF

# Create and copy icon if it exists
if [ -f "logo/logo.png" ]; then
    echo "Creating app icon..."
    # Create iconset directory
    ICONSET_DIR="${TEMP_DIR}/AppIcon.iconset"
    mkdir -p "${ICONSET_DIR}"
    
    # Create different sizes for the iconset
    sips -z 16 16 "logo/logo.png" --out "${ICONSET_DIR}/icon_16x16.png"
    sips -z 32 32 "logo/logo.png" --out "${ICONSET_DIR}/icon_16x16@2x.png"
    sips -z 32 32 "logo/logo.png" --out "${ICONSET_DIR}/icon_32x32.png"
    sips -z 64 64 "logo/logo.png" --out "${ICONSET_DIR}/icon_32x32@2x.png"
    sips -z 128 128 "logo/logo.png" --out "${ICONSET_DIR}/icon_128x128.png"
    sips -z 256 256 "logo/logo.png" --out "${ICONSET_DIR}/icon_128x128@2x.png"
    sips -z 256 256 "logo/logo.png" --out "${ICONSET_DIR}/icon_256x256.png"
    sips -z 512 512 "logo/logo.png" --out "${ICONSET_DIR}/icon_256x256@2x.png"
    sips -z 512 512 "logo/logo.png" --out "${ICONSET_DIR}/icon_512x512.png"
    sips -z 1024 1024 "logo/logo.png" --out "${ICONSET_DIR}/icon_512x512@2x.png"
    
    # Convert to icns
    iconutil -c icns "${ICONSET_DIR}" -o "${RESOURCES_DIR}/AppIcon.icns"
    
    # Clean up iconset directory
    rm -rf "${ICONSET_DIR}"
fi

# Create the DMG
echo "Creating DMG..."
DMG_PATH="${PWD}/${DMG_NAME}.dmg"

# Remove existing DMG if it exists
if [ -f "${DMG_PATH}" ]; then
    rm "${DMG_PATH}"
fi

create-dmg \
    --volname "${APP_NAME}" \
    --volicon "logo/logo.png" \
    --window-pos 200 120 \
    --window-size 800 600 \
    --icon-size 100 \
    --icon "${APP_NAME}.app" 200 190 \
    --hide-extension "${APP_NAME}.app" \
    --app-drop-link 600 185 \
    "${DMG_PATH}" \
    "${TEMP_DIR}"

# Clean up
echo "Cleaning up..."
rm -rf "${TEMP_DIR}"

echo "DMG created successfully: ${DMG_PATH}"
echo "File size: $(du -h "${DMG_PATH}" | cut -f1)"