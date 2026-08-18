#!/bin/bash
cp .env.example .env
grep -v '^DISCORD_APPLICATION_ID=' .env > .env.tmp && mv .env.tmp .env
echo "DISCORD_APPLICATION_ID=${DISCORD_APPLICATION_ID:-}" >> .env

pnpm i
pnpm run init

/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install lld

pnpm tauri build
