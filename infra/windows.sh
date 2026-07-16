#!/bin/bash
cp .env.example .env

pnpm i
if [[ "$arch" == "arm64" ]]; then
    pnpm i @tauri-apps/cli-win32-arm64-msvc
fi

pnpm run init

# Required for wgpu
cd src-tauri
cargo update

cd ..
pnpm tauri build
