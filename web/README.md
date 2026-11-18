# web/

Browser build output and the two glue scripts `play.html` loads.

- `gl.js`: miniquad's WebGL loader, copied from the miniquad sources macroquad ships
  with (MIT / Apache-2.0). Not edited.
- `macroquad_audio.js`: quad-snd WebAudio backend registered with miniquad, copied from
  the macroquad sources (MIT / Apache-2.0). Must load after `gl.js`.
- `literacy-rpg.wasm`: build output, git-ignored.

Rebuild the wasm after any game change:

```bash
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/literacy-rpg.wasm web/literacy-rpg.wasm
python3 assets/tools/devserver.py 5014
```

Then open `http://127.0.0.1:5014/play.html`. The server sends no-cache headers so a
reload always shows the build currently on disk.
