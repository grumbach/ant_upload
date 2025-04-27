#!/bin/bash

# Set your app name and binary path
APP_NAME="AntUpload"
BINARY_PATH="./target/release/ant_upload"  # adjust this path to your binary
ICON_PATH="./assets/ant_up.icns"

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
    <string>com.maidsafe.${APP_NAME}</string>
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