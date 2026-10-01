#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$REPO_ROOT/ui/macos"
DIST_DIR="$REPO_ROOT/dist"
BUNDLE_DIR="$DIST_DIR/Fluyer.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
FRAMEWORKS_DIR="$CONTENTS_DIR/Frameworks"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

CONFIGURATION="debug"
BUILD_DMG=false

for arg in "$@"; do
    case $arg in
        --release)
            CONFIGURATION="release"
            ;;
        --dmg)
            BUILD_DMG=true
            ;;
    esac
done

echo "==> Building fluyer_core in $CONFIGURATION mode..."
if [ "$CONFIGURATION" = "release" ]; then
    cargo build --release --manifest-path "$REPO_ROOT/crates/fluyer_core/Cargo.toml"
    RUST_BIN_DIR="$REPO_ROOT/target/release"
else
    cargo build --manifest-path "$REPO_ROOT/crates/fluyer_core/Cargo.toml"
    RUST_BIN_DIR="$REPO_ROOT/target/debug"
fi

echo "==> Generating UniFFI Swift bindings..."
"$REPO_ROOT/scripts/generate_swift_bindings.sh"

echo "==> Building SwiftUI executable in $CONFIGURATION mode..."
swift build -c "$CONFIGURATION" --package-path "$APP_DIR"

SWIFT_BIN_DIR="$APP_DIR/.build/$CONFIGURATION"
if [ ! -f "$SWIFT_BIN_DIR/FluyerApp" ]; then
    # Try finding in architecture subdirectory
    SWIFT_BIN_DIR="$(find "$APP_DIR/.build" -type f -name "FluyerApp" -path "*/$CONFIGURATION/*" | head -n 1 | xargs dirname)"
fi

echo "==> Packaging into $BUNDLE_DIR..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$FRAMEWORKS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy binary
cp "$SWIFT_BIN_DIR/FluyerApp" "$MACOS_DIR/"

# Copy runtime dynamic libraries
cp "$RUST_BIN_DIR/libfluyer_core.dylib" "$FRAMEWORKS_DIR/"
cp "$REPO_ROOT/libs/macos/libbass.dylib" "$FRAMEWORKS_DIR/"
cp "$REPO_ROOT/libs/macos/libbassmix.dylib" "$FRAMEWORKS_DIR/"

# Generate Info.plist
cat << 'EOF' > "$CONTENTS_DIR/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>FluyerApp</string>
    <key>CFBundleIdentifier</key>
    <string>org.alvindimas05.fluyer</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Fluyer</string>
    <key>CFBundleDisplayName</key>
    <string>Fluyer</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Update dynamic library load paths
install_name_tool -add_rpath "@executable_path/../Frameworks" "$MACOS_DIR/FluyerApp" 2>/dev/null || true

# Ad-hoc sign bundle for local execution
echo "==> Signing app bundle..."
codesign --force --deep --sign - "$BUNDLE_DIR"

echo "==> Fluyer.app successfully created at $BUNDLE_DIR"

# Build DMG if requested
if [ "$BUILD_DMG" = true ]; then
    echo "==> Creating Fluyer.dmg..."
    DMG_PATH="$DIST_DIR/Fluyer.dmg"
    rm -f "$DMG_PATH"
    hdiutil create -volname "Fluyer" -srcfolder "$BUNDLE_DIR" -ov -format UDZO "$DMG_PATH"
    echo "==> Fluyer.dmg successfully created at $DMG_PATH"
fi
