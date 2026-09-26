#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$REPO_ROOT/crates/fluyer_core"
OUTPUT_DIR="$REPO_ROOT/bindings/swift"

echo "==> Building fluyer_core dylib..."
cargo build --manifest-path "$CRATE_DIR/Cargo.toml"

echo "==> Generating UniFFI Swift bindings..."
cargo run --manifest-path "$CRATE_DIR/Cargo.toml" \
    --bin uniffi-bindgen generate \
    --library "$REPO_ROOT/target/debug/libfluyer_core.dylib" \
    --language swift \
    --out-dir "$OUTPUT_DIR"

echo "==> Structuring Swift Package..."
mkdir -p "$OUTPUT_DIR/Sources/FluyerCore"
mkdir -p "$OUTPUT_DIR/Sources/fluyer_coreFFI"

mv "$OUTPUT_DIR/fluyer_core.swift" "$OUTPUT_DIR/Sources/FluyerCore/"
mv "$OUTPUT_DIR/fluyer_coreFFI.h" "$OUTPUT_DIR/Sources/fluyer_coreFFI/"
mv "$OUTPUT_DIR/fluyer_coreFFI.modulemap" "$OUTPUT_DIR/Sources/fluyer_coreFFI/module.modulemap"

cat << 'EOF' > "$OUTPUT_DIR/Package.swift"
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FluyerCore",
    platforms: [
        .macOS(.v13),
        .iOS(.v16)
    ],
    products: [
        .library(
            name: "FluyerCore",
            targets: ["FluyerCore"]
        ),
    ],
    targets: [
        .target(
            name: "fluyer_coreFFI",
            path: "Sources/fluyer_coreFFI",
            publicHeadersPath: "."
        ),
        .target(
            name: "FluyerCore",
            dependencies: ["fluyer_coreFFI"],
            path: "Sources/FluyerCore"
        ),
    ]
)
EOF

echo "==> Swift bindings successfully generated in $OUTPUT_DIR!"
