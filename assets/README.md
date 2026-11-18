# assets/

All art and audio the game loads at runtime. Files in `game/` are generated, so do not
hand-edit them: change `tools/build.py` or drop replacement art in with the same names.

## Layout

- `vendor/`: third-party CC0 / OFL source packs, plus `vendor/README.md` with the
  licence and source link for each.
- `fonts/`: pixel TTFs (Press Start 2P, VT323) and their OFL licences.
- `tools/build.py`: the whole art pipeline. Reads `vendor/` and writes `game/`.
- `tools/devserver.py`: tiny no-cache HTTP server for browser testing.
- `game/`: generated runtime assets.

## Regenerating

```bash
python3 assets/tools/build.py
```

It needs Python 3 and Pillow, and nothing else. Output is deterministic, so a rerun
with unchanged inputs produces byte-identical files.

## game/ file map

| Path | Size | Contents |
|---|---|---|
| `game/char/<cast>.png` | 96x144 | Standing frame. `<cast>_w1.png` and `_w2.png` are the walk cycle, `<cast>_shadow.png` is the baked contact shadow |
| `game/tiles/*.png` | 16x16 | Ground, ground variants, dirt, flowers, bushes, stone, home furniture |
| `game/aqua/*.png` | varies | Fish, seaweed, rocks, background kelp, tight-cropped |
| `game/cosma/*.png` | varies | Floor blocks, lit rim, neon nodes, crystals, boulders, meteors, pebbles, planets |
| `game/fx/*.png` | 128x128 | Particle sprites: smoke, spark, star, magic, fire, flame, circle, trace, twirl, light, flare, dirt |
| `game/audio/*.ogg` | varies | Mastered SFX and footsteps |
| `game/sheet_candy.png` | full sheet | Palette-remapped roguelike sheet kept for reference and future crops |

## Cast ids

`river`, `jin`, `kandy`, `matt`, `guard_a`, `guard_b`, `nadia`, `grandparent`,
`gill`, `scal`, `lady`. Each id needs a stand frame, two walk frames and a shadow.
The engine loads exactly these names, so real art can replace a single character
without touching the others.

## Replacing placeholder art with final art

1. Export the character at any resolution into the 2:3 canvas used by the pack
   (96x144 works, larger is fine as long as the aspect stays 2:3 and the feet sit on
   the canvas bottom).
2. Keep the file names above, or add the character to `assets/tools/build.py` if it
   belongs in the generated set.
3. If final art changes the tint, colour or silhouette, update the token table in
   `docs/DESIGN.md` so the rest of the scenes stay consistent.
4. `cargo build --release` and `cargo run -- --shot` to check the new sprite in the
   Candy Land screen.
