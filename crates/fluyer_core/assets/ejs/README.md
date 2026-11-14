# yt-dlp EJS solver (vendored)

`jsc.min.js` is a generated artifact of <https://github.com/yt-dlp/ejs> (SPDX: Unlicense,
bundles `meriyah` ISC and `astring` MIT), rebuilt from `main` with:

```sh
esbuild src/yt/solver/main.ts --bundle --format=iife --global-name=jsc --minify
```

It runs inside QuickJS (`rquickjs`) in `crates/fluyer_core/src/services/ytmusic/cipher.rs`:

- `jsc.default({type: "player", player: baseJs, requests: [], output_preprocessed: true})`
  → `preprocessed_player`, cached on disk per player version.
- `jsc.default({type: "preprocessed", preprocessed_player, requests: [{type: "n" | "sig", challenges: [...]}]})`
  → `responses[0].data[challenge]`.

Rebuild this file when YouTube changes its player and extraction stops matching.
