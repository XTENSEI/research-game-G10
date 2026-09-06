pub mod script;
pub mod gate;
pub mod maze;
pub mod cosma;
pub mod jin_battle;
pub mod ui;
pub mod world;
pub mod run;
pub mod save;

/// Shared run state that persists across scenes.
pub struct Progress {
    /// True when the Jin battle was cleared with zero losses.
    pub has_horse: bool,
    /// Riddle results per biome (one-try only).
    pub candy_riddles_done: u32,
    pub aqua_riddles_done: u32,
    pub cosma_riddles_done: u32,
    /// Gate puzzle solved.
    pub gate_solved: bool,
    /// Maze puzzle solved.
    pub maze_solved: bool,
    /// Elixir components collected (one per biome).
    pub ingredients: u32,
    /// Encounters already completed this run (by action id).
    pub done: std::collections::HashSet<&'static str>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            has_horse: false,
            candy_riddles_done: 0,
            aqua_riddles_done: 0,
            cosma_riddles_done: 0,
            gate_solved: false,
            maze_solved: false,
            ingredients: 0,
            done: std::collections::HashSet::new(),
        }
    }
}

/// Global flag so the world knows travel speed. Set when Jin's horse is earned.
use std::sync::atomic::{AtomicBool, Ordering};
static HORSE: AtomicBool = AtomicBool::new(false);

pub fn set_horse(on: bool) {
    HORSE.store(on, Ordering::Relaxed);
}
pub fn ride_unlocked() -> bool {
    HORSE.load(Ordering::Relaxed)
}
