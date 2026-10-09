#!/bin/bash
# Cross-builds fluyer_core for Android, stages it + BASS into the app's jniLibs and
# regenerates the Kotlin bindings. Run before `./gradlew` in ui/android.
#   ./scripts/build_android.sh            # debug
#   ./scripts/build_android.sh --release  # release
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$REPO_ROOT/crates/fluyer_core"
APP_DIR="$REPO_ROOT/ui/android"
# ponytail: arm64 only (every phone since ~2017 + the arm64 emulator image). Add
# x86_64 / armeabi-v7a here and in app/build.gradle.kts `abiFilters` if needed.
TARGET="aarch64-linux-android"
ABI="arm64-v8a"
API=26

PROFILE="debug"
CARGO_FLAGS=()
if [ "$1" = "--release" ]; then
    PROFILE="release"
    CARGO_FLAGS=(--release)
fi

SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
NDK="${ANDROID_NDK_HOME:-$(ls -d "$SDK"/ndk/* 2>/dev/null | sort -V | tail -1)}"
[ -d "$NDK" ] || { echo "Android NDK not found; set ANDROID_NDK_HOME" >&2; exit 1; }
TOOLCHAIN="$(ls -d "$NDK"/toolchains/llvm/prebuilt/* | head -1)/bin"

[ -f "$REPO_ROOT/libs/android/$ABI/libbass.so" ] || {
    echo "libs/android/$ABI/libbass*.so missing (gitignored; copy them in)" >&2; exit 1; }

export CC_aarch64_linux_android="$TOOLCHAIN/${TARGET}${API}-clang"
export AR_aarch64_linux_android="$TOOLCHAIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/${TARGET}${API}-clang"

echo "==> Building fluyer_core ($PROFILE, $TARGET)..."
rustup target add "$TARGET" >/dev/null 2>&1 || true
cargo build --manifest-path "$CRATE_DIR/Cargo.toml" --lib --target "$TARGET" "${CARGO_FLAGS[@]}"
SO="$REPO_ROOT/target/$TARGET/$PROFILE/libfluyer_core.so"

echo "==> Staging native libs into jniLibs/$ABI..."
JNI_DIR="$APP_DIR/app/src/main/jniLibs/$ABI"
mkdir -p "$JNI_DIR"
cp "$SO" "$JNI_DIR/"
cp "$REPO_ROOT/libs/android/$ABI/"*.so "$JNI_DIR/"

echo "==> Generating UniFFI Kotlin bindings..."
cargo run --manifest-path "$CRATE_DIR/Cargo.toml" --bin uniffi-bindgen generate \
    --library "$SO" --language kotlin --out-dir "$REPO_ROOT/bindings/kotlin" --no-format

echo "==> Done. Next: (cd ui/android && ./gradlew installDebug)"
