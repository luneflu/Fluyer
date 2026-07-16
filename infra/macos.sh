#!/bin/bash
cp .env.example .env

pnpm i
pnpm run init

/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install lld

pnpm tauri build
