//! Aquarius Puzzle: The Grave Cave Maze of Craze (SCRIPT RES.pdf pp. 16-17).
//! Cardinal N/E/S/W randomized chambers & shuffled direction choices.
//! Right answers advance depth toward the Somber Slums; wrong answers silently
//! step you back one chamber with ZERO visual cue.

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
            Dir::North => "North",
            Dir::East => "East",
            Dir::South => "South",
            Dir::West => "West",
        }
    }
}

#[derive(Clone, Copy)]
pub struct MazeQuestion {
    pub dir: Dir,
    pub prompt: &'static str,
}

/// All 8 cardinal direction & NPC-clue riddles from SCRIPT RES.pdf pp. 16-17.
pub const QUESTIONS: &[MazeQuestion] = &[
    // North
    MazeQuestion {
        dir: Dir::North,
        prompt: "Chamber Inscription:\n\"I am a red needle pointing the way you must walk.\"",
    },
    MazeQuestion {
        dir: Dir::North,
        prompt: "Market Memory:\n\"Take the path where real, sweet True Seaberries actually grow.\"",
    },
    // East
    MazeQuestion {
        dir: Dir::East,
        prompt: "Ancient Pun:\n\"It wouldn't be an occident to orient yourself toward me.\"",
    },
    MazeQuestion {
        dir: Dir::East,
        prompt: "Harbor Memory:\n\"Walk the direction that Neptune's Statue at City Hall faces.\"",
    },
    // South
    MazeQuestion {
        dir: Dir::South,
        prompt: "Abyss Proverb:\n\"Things go wrong... things go ___.\"",
    },
    MazeQuestion {
        dir: Dir::South,
        prompt: "Phonetic Riddle:\n\"I sound like a soft 'down', yet I am only a compass direction.\"",
    },
    // West
    MazeQuestion {
        dir: Dir::West,
        prompt: "Town Rumor:\n\"Follow where Scalleon and the market locals first claimed sea berries grew.\"",
    },
    MazeQuestion {
        dir: Dir::West,
        prompt: "Twilight Verse:\n\"I am the nest where light rests; I am where daylight leaves at night.\"",
    },
];

pub const MAZE_TARGET_DEPTH: u8 = 4;

/// Generates a shuffled 8-question cycle plus a shuffled [N/E/S/W] layout.
pub fn shuffled_cycle() -> ([usize; 8], [Dir; 4]) {
    let mut order = [0usize, 1, 2, 3, 4, 5, 6, 7];
    for i in (1..8).rev() {
        let j = macroquad::rand::gen_range(0, i + 1);
        order.swap(i, j);
    }
    (order, shuffled_dirs())
}

pub fn shuffled_dirs() -> [Dir; 4] {
    let mut dirs = [Dir::North, Dir::East, Dir::South, Dir::West];
    for i in (1..4).rev() {
        let j = macroquad::rand::gen_range(0, i + 1);
        dirs.swap(i, j);
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maze_covers_every_direction() {
        for d in [Dir::North, Dir::East, Dir::South, Dir::West] {
            assert_eq!(QUESTIONS.iter().filter(|q| q.dir == d).count(), 2);
        }
        let (order, dirs) = shuffled_cycle();
        let mut sorted = order;
        sorted.sort();
        assert_eq!(sorted, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert!(dirs.contains(&Dir::North) && dirs.contains(&Dir::South));
        assert!(dirs.contains(&Dir::East) && dirs.contains(&Dir::West));
    }
}
