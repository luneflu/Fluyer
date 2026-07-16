#!/bin/bash
cp .env.example .env

sudo curl --output-dir /etc/apt/trusted.gpg.d -O https://apt.fruit.je/fruit.gpg
echo "deb http://apt.fruit.je/debian trixie mpv" | sudo tee /etc/apt/sources.list.d/fruit.list
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf librust-alsa-sys-dev libmpv-dev lld

pnpm i
pnpm run init

export NO_STRIP=true
pnpm tauri build
