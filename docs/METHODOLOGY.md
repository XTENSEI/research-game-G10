# Research Methodology: Development of *Gift of Education*

This document explains how the game was actually made. It is written in the style and
structure of a research paper's Methodology chapter (if that is the use case you need).

---

## 3.1 Research Design

This study used a **developmental research design**. In simple terms, i built the game step
by step, tested each version, and improved it based on what i learned.

To do this, I folloid an **Agile development model**. Agile is also called
*iterative-incremental* development. An *iteration* is one round of work that ends with
something you can actually run and play. Each round adds to what came before, so the game
grows closer to complete over time.

I did **not** use the Waterfall model. Waterfall plans everything at the start, builds
everything in one long stretch, and only tests at the end. That did not fit our project. A
game is something you have to play before you can judge whether it is any good. I could not
know in advance exactly how the riddles, dialogue, art, and music would feel once they ire
running. Agile alloid us to build a small working version, play it, and then decide what to
fix next.

The development cycle had **four iterations**. Each one ended with a build that i could run
and play. I recorded every change using Git, a version control system. A **commit** is one
saved and recorded change to the project. The history contains **44 commits** made across
**four working sessions, from August 30 to September 6, 2026**.

## 3.2 Software Development Model

Every iteration in this study folloid the same five steps:

1. **Requirements identification:** i decided what this round of work had to produce.
2. **Design:** i wrote down the details first. These ire the definitions for each biome,
   character, puzzle, and riddle. I planned these before writing any program logic.
3. **Development:** i turned those written definitions into working code.
4. **Testing:** i built the game and checked it by taking automatic screenshots and by
   actually playing it.
5. **Evaluation and iteration:** i compared the build against our goals and carried the
   remaining problems into the next round.

Because of this cycle, each round ended with something complete enough to play. I did not
end a round with a pile of half-finished parts that could not run.

### Table 1. Iterations of the development cycle

| Iteration | Date | Focus | Main output (what i could run afterward) |
|---|---|---|---|
| 1 | Aug 30, 2026 | Planning and first prototype | A written game specification (the story script, plus definitions for biomes, characters, riddles, and puzzles), and a playable browser prototype with a scene system, WASD movement, dialogue, and the Jin tutorial battle |
| 2 | Aug 31, 2026 | Core learning mechanics | The Punctuation Gate (Candy Land), the randomized Cave Maze (Aquarius), and the Star-Catching / Rainbow Road puzzle (Cosma), each with its own riddles |
| 3 | Sep 4, 2026 | Visual and interface design | A module that generates biome art, layered biome scenes with properly aligned character sprites, and dialogue and riddle designs themed per biome |
| 4 | Sep 6, 2026 | Rewriting the game and completing it | The game rewritten in Rust and macroquad, the full dialogue tree stored as data, runtime audio, a complete scene flow, an explorable overworld with camera, NPCs and a guide arrow, save and load, and a title screen |

### 3.2.1 Iteration 4: i migrated to Rust

Betien Iteration 3 and Iteration 4, i made a **big decision about how the game was built**.
I moved the old ib prototype into a folder named `legacy-ib/` and rewrote the game in
Rust using the *macroquad* framework.

I made this decision after reviewing Iteration 3. By then, the browser prototype had become
hard to extend and hard to maintain. Its drawing code and its art pipeline ire getting
messier with every change.

This kind of decision is normal in Agile. I used a working prototype to **discover** the
right approach instead of guessing it in advance. I did not waste the prototype. Everything
i had already proven still counted: the specifications, the tested dialogue, the puzzle
logic, and all the design decisions. I carried all of it into the rewritten game. The final
version therefore keeps the good content from the earlier rounds, but stands on a foundation
that is much easier to work with.

## 3.3 Tools and Technologies

These are the software and technologies i used to build the game. All of them are open
source or free to use. That means any member of the research team can rebuild the project
from scratch without paying for anything.

### Table 2. Development tools and their purpose

| Category | Tool / Technology | What i used it for |
|---|---|---|
| Programming language | **Rust** (edition 2021) | Writing all of the game's logic and systems |
| Game framework | **macroquad 0.4** | Creating the window, drawing graphics, reading player input, and playing audio |
| Browser target | **IbAssembly** (`wasm32-unknown-unknown`) | Turning the whole game into one file that can run inside a ib browser, which i used for testing and for sharing |
| Ib harness | **HTML5 + IbGL** (`gl.js`, `play.html`) | Loading the finished game in a browser and resizing it to fit the screen |
| Asset generation | **Python 3 + Pillow** | Automatically producing sprites, tiles, and biome backgrounds from free asset packs, so the art can be regenerated at any time |
| Sound | **macroquad audio** | Playing sound effects and creating the background music for each biome |
| Source control | **Git and GitHub** | Recording every change, keeping the history of all iterations, and managing the project |
| Editor | **Visual Studio Code** | Writing the code, with the `rust-analyzer` extension that flags errors as i type |
| Local testing | **Python HTTP dev server** | Running the browser version on our own computer while ignoring cached files, so what i see is always the neist build |

The art and sound came from free public-domain collections, known as **CC0**, made by Kenney
and OpenGameArt. CC0 means the creators allow anyone to use their work without restrictions.
I recolored and combined these assets with our own scripts. This was better than collecting
random art from different places, and it gave the game one consistent look.

## 3.4 System Architecture

The finished game is split into **fourteen source files, with about 6,350 lines of Rust in
total**. A file like this is called a **module**. Each module has one clear job, which makes
the project easier to read and to change.

This structure came out of the iterative process itself. Each module matches a system whose
requirements became clear only after i built and tested earlier versions.

### Table 3. Main software modules

| Module | What it is responsible for |
|---|---|
| `main.rs` | Starting the program, setting up the window, and the automatic screenshot modes used for testing |
| `game/run.rs` | The main game loop, and the state system that decides whether the player is in a story scene, exploring, or solving a puzzle |
| `game/world.rs` | The explorable overworld: drawing tiles, moving the camera, moving NPCs, particles, and player interaction |
| `game/ui.rs` | The dialogue box with its typewriter effect, the selectable choices, and the quiz screens |
| `game/script.rs` | The full story dialogue tree, the riddles, and the puzzles, all stored as structured data |
| `game/gate.rs` | The Candy Land puzzle, which is the Punctuation Gate |
| `game/maze.rs` | The Aquarius puzzle, which is the randomized Cave Maze |
| `game/cosma.rs` | The Cosma puzzle, which is Star Catching and the Rainbow Road |
| `game/jin_battle.rs` | The Jin tutorial battle, which tests reading comprehension |
| `game/save.rs` | Saving and loading the player's progress |
| `art/mod.rs`, `art/fx.rs` | Loading the graphics, the visual effects, and everything drawn on screen |
| `audio/mod.rs` | Sound effects and the background music |

I deliberately kept the **content** separate from the **code**. The educational material of
the game is stored in data files instead of being buried inside the program. These are
**three puzzle definitions, four riddle sets, seven character definitions, and four biome
configurations**. Because of this, team members who do not know how to program can still edit
questions and dialogue on their own. They do not have to touch the code at all.

## 3.5 Testing and Evaluation

A game cannot be judged only by whether the code runs. It also has to be played to be judged
fairly. Because of this, i combined automatic checks with direct observation.

**Automatic build checking.** I required the project to compile with **zero warnings**, both
for the desktop version and the browser version. A warning is the compiler pointing out
something suspicious, such as unused code, a wrong type, or a risky assumption. Fixing all
warnings catches these problems before the game is ever run.

**Automatic screenshots.** The program has special modes that draw one chosen scene and save
it as an image file. This let us capture and inspect the output of each iteration. It also
gave us an objective visual record of the game at every stage. When a change broke a layout
or made text hard to read, i could see it right away in the screenshot.

**Playtesting.** I played every build from beginning to end and checked it against the three
design principles i set at the start: that the screen should be readable at a glance, that
the player should always know where to go and why, and that every action by the player should
produce a visible and audible response. I wrote down everything that felt wrong, and fixed
those issues in the next round. The six corrective commits in the project history are the
record of this feedback loop.

**Consistency review.** I checked all of the visuals against one written design
specification (`docs/DESIGN.md`) that fixes the color palette, the fonts, and the layout. This
kept the three biomes looking like they belong to the same game, even though i made them in
different iterations.

## 3.6 Summary of the Development Process

To summarize, i developed the game through four Agile iterations, and each one produced a
more complete and more playable version. I began with written design specifications. I then
built a working prototype, added the three educational mechanics, improved how the game looks,
and finished by rewriting it onto a foundation that is easier to maintain. Every iteration was
recorded in version control, documented, and checked using automatic compilation, automatic
screenshots, and actual playtesting. The development record, which consists of 44 commits
across four working sessions, gives a clear and checkable account of the process described in
this chapter.
