pub mod script;
pub mod gate;
pub mod maze;
pub mod cosma;
pub mod jin_battle;
pub mod ui;
pub mod run;

/// Shared run state that persists across scenes.
pub struct Progress {
    /// Tutorial battle: how many times River lost to Jin.
    pub jin_losses: u32,
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
    /// Elixir components collected (one per biome boss/puzzle).
    pub ingredients: u32,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            jin_losses: 0,
            has_horse: false,
            candy_riddles_done: 0,
            aqua_riddles_done: 0,
            cosma_riddles_done: 0,
            gate_solved: false,
            maze_solved: false,
            ingredients: 0,
        }
    }
}
