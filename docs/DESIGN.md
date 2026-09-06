# DESIGN.md - Gift of Education

Single source of truth for look, feel, and sound. Every scene must obey this file.

## Pillars

1. Readable first. Any player parses the screen in one glance.
2. Words are the weapon. Dialogue and riddles are the combat system.
3. World reacts. Every input gets visual and audible feedback.

## Palette

| Role | Hex | Used for |
|---|---|---|
| Candy sky | #FFAACC | Candy Land upper sky |
| Candy cloud | #FFCDE1 | Clouds, bushes |
| Lemon land | #FAE678 | Candy Land ground |
| Aquarius deep | #08416E | Water gradient floor |
| Aquarius light | #145A96 | Water gradient top |
| Cosma void | #06040E | Neon space |
| Dialogue border | #28C8FF | Undertale box frame |
| Dialogue accent | #FF69B4 | Speaker tags, highlights |
| UI white | #FFFFFF | Body text |

Rule: one pink accent per screen. Backgrounds stay soft so white text always wins.

## Typography

In-game text uses the macroquad default bitmap face. Scale:

- Speaker tag: 20px, dark text on pink chip
- Body dialogue: 20px, 24px line height, wrapped at box width minus portrait
- Quiz question: 20px, options at 20px with 44px row spacing
- Headers (puzzle names): 34px pink, centered

## Layout

- Canvas: 1024x720 fixed, letterboxed by the OS window.
- Dialogue box: bottom, 20px margins, 190px tall, 3px pulsing blue frame.
- Portrait: 140x150 inside the box left, speaker chip sits on the top edge.
- Quiz: header at top, question at y 110, options from y 250, feedback at y 470.
- Maze depth meter: right edge, 14x300.
- Rainbow Road: poem preview top left, star field center, paddle y 570, meter bottom left.

## Signature

The Throne Gate passage renders as living stone: the story typeset into the gate
itself, with empty boxes waiting for punctuation. It is the visual thesis of the
game: text is terrain.

## Audio

All synthesized at runtime, no files.

| Sound | Recipe | Feels like |
|---|---|---|
| Biome pads | Layered detuned sines, 4s loop | House warm C major, Candy bright, Aquarius low G, Cosma spacey F |
| Blip | 880Hz 50ms tick | Typewriter |
| Good | 660 to 990Hz two note chime | Correct answer |
| Bad | 110Hz wobble buzz | Wrong answer |
| Page | Noise swoosh 250ms | Intro advance |
| Splash | Swept noise burst 600ms | Trapdoor fall |

Mixing: ambience at 0.12 volume, SFX at 0.5. Never clip: all chunks sum with clamp.

## Interaction map (WASD plus)

- Z or Enter or click: advance dialogue, confirm
- 1-4: pick quiz option or choice
- Arrows or A/D: navigate, move paddle
- Space: mash during QTE

## Anti-slop rules

- No text overlap, ever. Wrapping is greedy and measured.
- No dead click: every click either advances, selects, or is consumed by a hit zone.
- One try per riddle, feedback states the consequence.
- Wrong answers never hard-fail puzzles, only riddles.
