#!/bin/bash
cp .env.example .env
grep -v '^DISCORD_APPLICATION_ID=' .env > .env.tmp && mv .env.tmp .env
echo "DISCORD_APPLICATION_ID=${DISCORD_APPLICATION_ID:-}" >> .env

pnpm i
if [[ "$arch" == "arm64" ]]; then
    export ARCH="arm64"
    pnpm i @tauri-apps/cli-win32-arm64-msvc
fi

pnpm run init

# Required for wgpu
# cd src-tauri
# cargo update

# cd ..
if [[ "$arch" == "arm64" ]]; then
    pnpm tauri build --target aarch64-pc-windows-msvc
else
    pnpm tauri build
fi
