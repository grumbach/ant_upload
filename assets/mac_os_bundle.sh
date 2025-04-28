#!/bin/bash

# Script to create a mac os bundle for the ant_upload binary
#
# BINARY_PATH is the path to the binary to be bundled, it can be provided as an environment variable:
#
# BINARY_PATH=./target/release/ant_upload ./mac_os_bundle.sh
#
# If the desired binary path is not provided, the default path will be used

APP_NAME="AntUpload"
ICON_PATH="./assets/ant_up.icns"

if [ -z "$BINARY_PATH" ]; then
    BINARY_PATH="./target/release/ant_upload"  # this one is the default path
fi

# Create the .app structure
mkdir -p "${APP_NAME}.app/Contents/MacOS"
mkdir -p "${APP_NAME}.app/Contents/Resources"

# Copy your binary
cp "$BINARY_PATH" "${APP_NAME}.app/Contents/MacOS/${APP_NAME}"

# Copy icon
cp "$ICON_PATH" "${APP_NAME}.app/Contents/Resources/AppIcon.icns"

# Create Info.plist
cat > "${APP_NAME}.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.${APP_NAME}.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.10</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
</dict>
</plist>
EOF

# Make the binary executable
chmod +x "${APP_NAME}.app/Contents/MacOS/${APP_NAME}"

echo "Created ${APP_NAME}.app"