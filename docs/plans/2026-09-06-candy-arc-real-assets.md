# Candy Land Arc Polish, real-asset pass

Date: 2026-09-06. Status: active. This is the plan for the pass the operator approved:
real sprite pack + polish one full arc (House intro to trapdoor plunge). Aquarius and
Cosma stay reachable behind a "next episode" card after the plunge.

## Design direction (locked)

Soft-storybook confection: a candy dreamland drawn in honest pixel art, deep-pastel
biome palettes, chunky sprites, and an Undertale-style cream dialogue panel. One
aesthetic risk: recolor generic grass/dungeon art into a lemon-pink candy world via
palette remap instead of trying to draw candy objects, so the world reads coherent and
art-directed, not asset-soup.

Pillars (game-designer): the player always knows where to go and why; every answer
beat gives instant feedback (sound, flash, bounce); the world is never empty of motion.

### Tokens (frontend-design)
- Sky pink #F7B8D0, deep candy pink #E26A9C, lemon ground #F6DE7E, licorice red #C43D5E
- Text ink plum #2E1A47 on cream panel #FFF6E9, panel border #4A2A63
- Aqua accent (Aquarius tease) #54B8C9, genie flame #FF9F45
- Type: OFL pixel font (Pixelify Sans or Press Start 2P), sizes 16/24/40
- Grid: 16px tiles rendered x3 (48px); characters ~x2 scale

### Signature
A bouncing pink guide arrow plus chunky squash-stretch walk is the remembered motion;
the palette-remapped candy world is the remembered look.

## Asset plan

Vendored in assets/vendor/kenney (all CC0, License.txt per pack):
- roguelike pack: 16px ground/water/tree/dungeon tiles, props. Source for candy palette remap.
- urban tilemap: street/plaza tiles (guards/castle gate area).
- top-down shooter characters: Man Brown (River), Man Blue + Hitman (castle guards),
  Woman Green (Kandy Kween, recolored pink with crown overlay), Man Old (grandparent).
- Font: OFL TTF from Google Fonts, with license file.

Build step (python): slice the roguelike sheet by its 1px-margin grid, palette-remap a
chosen subset into candy colors per biome, and write plain PNGs + a frame index JSON so
Rust loads simple files. All generated art is committed so the game runs without python.

## Code plan (Rust, existing base kept)

1. art/mod.rs: load textures from assets/game/*.png; keep particle/camera code; tint-free.
2. world.rs: ground rendered from a JSON tile strip; players/NPCs draw their sprite
   texture with bob and facing flip.
3. ui.rs: load TTF pixel font; restyle dialogue panel to tokens; keep typewriter/choices.
4. run.rs: no phase rework; add --shot N to render frame N to shot.png and exit so we can
   actually see the game and iterate (the missing feedback loop).
5. After the plunge: black, then "AQUARIUS / next episode" title card, stop. Aquarius and
   Cosma phases stay compiled behind a debug flag.
6. Intro panels restyled to tokens, script lines verbatim.

## Verification

- python build script runs clean and regenerates committed art deterministically.
- cargo build with zero warnings.
- --shot frames for intro, candy overworld, dialogue, battle, gate, guards, queen, plunge.
  Inspect each on screen and fix what looks wrong before reporting to the operator.
- Operator runs cargo run and gives one piece of feedback; we fix forward.

## Scope cut

No new systems. No engine change. Not touching Aquarius/Cosma mechanics this pass.
One logical change per commit, signed, scoped: art, ui, tools, docs.
