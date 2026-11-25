//! Aquarius Puzzle: The Grave Cave Maze of Craze (SCRIPT RES.pdf pp. 16-17).
//! Cardinal N/E/S/W randomized chambers. Right answers advance depth toward exit;
//! wrong answers silently return the traveler one step back with NO visual cue.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    pub fn label(self) -> &'static str {
        match self {
            Dir::North => "NORTH",
            Dir::East => "EAST",
            Dir::South => "SOUTH",
            Dir::West => "WEST",
        }
    }
}

#[derive(Clone, Copy)]
pub struct MazeQuestion {
    pub dir: Dir,
    pub prompt: &'static str,
    pub choices: [&'static str; 4],
    pub correct: usize,
}

/// Verbatim 8-question cardinal bank from SCRIPT RES.pdf pp. 16-17.
pub const QUESTIONS: &[MazeQuestion] = &[
    // North
    MazeQuestion {
        dir: Dir::North,
        prompt: "Which sentence demonstrates correct subject-verb agreement?",
        choices: [
            "The reef dolphins swims beside Tow-Tou.",
            "Neither Gillbert nor the guards was late.",
            "A school of glowing lanterns drifts above the trench.",
            "The twin pearls on the pedestal sparkles loudly.",
        ],
        correct: 2,
    },
    MazeQuestion {
        dir: Dir::North,
        prompt: "What tense is the verb in: \"The cat sits on the mat\"?",
        choices: [
            "Past simple",
            "Present simple",
            "Future progressive",
            "Present perfect",
        ],
        correct: 1,
    },
    // East
    MazeQuestion {
        dir: Dir::East,
        prompt: "\"This is the most delicious meal I have ever tasted.\"\nWhat degree of comparison is 'most delicious'?",
        choices: [
            "Positive degree",
            "Comparative degree",
            "Superlative degree",
            "Participial modifier",
        ],
        correct: 2,
    },
    MazeQuestion {
        dir: Dir::East,
        prompt: "\"In order to succeed, you must study hard every day.\"\nWhat role does 'In order to succeed' play?",
        choices: [
            "Independent main clause",
            "Infinitive phrase of purpose",
            "Appositive noun phrase",
            "Coordinating conjunction",
        ],
        correct: 1,
    },
    // South
    MazeQuestion {
        dir: Dir::South,
        prompt: "\"While she was cooking dinner, the lantern flickered.\"\nWhat type of clause is 'While she was cooking dinner'?",
        choices: [
            "Dependent adverbial clause",
            "Independent sentence",
            "Relative adjective clause",
            "Prepositional fragment",
        ],
        correct: 0,
    },
    MazeQuestion {
        dir: Dir::South,
        prompt: "Fill in the blank for logical cause and effect:\n\"She was tired, ____ she went to bed early.\"",
        choices: [
            "yet",
            "so",
            "unless",
            "nor",
        ],
        correct: 1,
    },
    // West
    MazeQuestion {
        dir: Dir::West,
        prompt: "\"Although it was raining, the travelers pressed onward.\"\nWhat grammatical function does 'Although' serve?",
        choices: [
            "Subordinating conjunction showing contrast",
            "Modal auxiliary verb",
            "Possessive determiner",
            "Correlative pronoun",
        ],
        correct: 0,
    },
    MazeQuestion {
        dir: Dir::West,
        prompt: "Choose the proper subjunctive mood:\n\"If I ___ you, I would bring Tow-Tou's spare wrench.\"",
        choices: [
            "was",
            "am",
            "were",
            "will be",
        ],
        correct: 2,
    },
];

pub const MAZE_TARGET_DEPTH: u8 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maze_covers_every_direction() {
        for d in [Dir::North, Dir::East, Dir::South, Dir::West] {
            assert_eq!(QUESTIONS.iter().filter(|q| q.dir == d).count(), 2);
        }
    }
}
