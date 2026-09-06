//! The Grave Cave Maze of Craze. Directional questions with random-to-direction
//! mapping, NPC/environment clues, and silent send-back on failure.

pub struct MazeQ {
    pub question: &'static str,
    /// Correct direction: 0=North 1=East 2=South 3=West
    pub dir: usize,
}

pub struct MazeClue {
    #[allow(dead_code)]
    pub who: &'static str,
    pub text: &'static str,
}

pub const MAZE_CLUES: &[MazeClue] = &[
    MazeClue { who: "Gillbert", text: "hey, do you know where wild sea berries grow?" },
    MazeClue { who: "Scalleon", text: "hmm, I believe it's in the *west*, i think?" },
    MazeClue { who: "Market Lady", text: "do you wanna know where fresh sea berries grow?" },
    MazeClue { who: "Market Lady", text: "real ones grow from the *North*" },
    MazeClue { who: "(Environment)", text: "City hall faces to the *east*" },
    MazeClue { who: "(Environment)", text: "the sun is the light that leaves at night" },
];

/// Questions from the script. The mapping question -> direction is shuffled at runtime;
/// `dir` here is the *correct* direction for the mechanic, display order is shuffled too.
pub const MAZE_QUESTIONS: &[MazeQ] = &[
    MazeQ { question: "Things go wrong, Things go...", dir: 2 },                        // South
    MazeQ { question: "I sound like \"down\", but I'm technically not down", dir: 2 },   // South
    MazeQ { question: "I am a red needle for the path you take", dir: 0 },               // North
    MazeQ { question: "Where the fresh sea berries grow", dir: 0 },                      // North
    MazeQ { question: "It wouldn't be an *occident* to orient yourself to me", dir: 1 }, // East
    MazeQ { question: "Direction where the city hall faces", dir: 1 },                   // East
    MazeQ { question: "Where the locals *claim* the sea berries grow", dir: 3 },         // West
    MazeQ { question: "I am the nest for where it rests ; it is the light that leaves at night", dir: 3 }, // West (sun)
];

pub const DIR_NAMES: &[&str] = &["North", "East", "South", "West"];
