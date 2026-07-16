#!/bin/bash
cp .env.example .env

pnpm i

export ANDROID_NDK_HOME=$NDK_HOME
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export RANLIB=$ANDROID_NDK_ROOT/llvm-ranlib
export AR=$ANDROID_NDK_ROOT/llvm-ar
export STRIP=$ANDROID_NDK_ROOT/llvm-strip
export LD=$ANDROID_NDK_ROOT/ld

cd src-tauri/gen/android
echo "keyAlias=$ANDROID_KEY_ALIAS" > keystore.properties
echo "password=$ANDROID_KEY_PASSWORD" >> keystore.properties
base64 -d <<< "$ANDROID_KEY_BASE64" > $RUNNER_TEMP/keystore.jks
echo "storeFile=$RUNNER_TEMP/keystore.jks" >> keystore.properties
cd ../../../
pnpm android:init -a $ANDROID_ARCH
pnpm tauri android build -v --target $ANDROID_ARCH
