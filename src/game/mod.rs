pub mod script;
pub mod gate;
pub mod maze;
pub mod cosma;
pub mod kenny_battle;
pub mod ui;
pub mod world;
pub mod run;
pub mod save;

use std::sync::atomic::{AtomicBool, Ordering};

/// Shared run state across biomes, dialogue, puzzles, and save files.
pub struct Progress {
    /// True if Kenny's tutorial battle was cleared with zero mistakes.
    pub kenny_flawless: bool,
    /// One-try riddle outcomes per biome (right count, total seen).
    pub kelp_riddles_right: u32,
    pub kelp_riddles_seen: u32,
    pub coral_riddles_right: u32,
    pub coral_riddles_seen: u32,
    pub slums_riddles_right: u32,
    pub slums_riddles_seen: u32,
    /// Major biome puzzle flags.
    pub gate_solved: bool,
    pub maze_solved: bool,
    pub algae_solved: bool,
    pub rainbow_solved: bool,
    /// Coral City fetch items: [Blue Seashroom, Tentacle Anemone, True Seaberry].
    pub items: [bool; 3],
    /// Clues logged into [Tab] Field Notes from Aquarium NPCs.
    pub clues: Vec<&'static str>,
    /// Elixir progress (0..=3 components / complete vial).
    pub elixir: u32,
    /// Completed one-shot encounters / locked riddles (by id).
    pub done: std::collections::HashSet<&'static str>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            kenny_flawless: false,
            kelp_riddles_right: 0,
            kelp_riddles_seen: 0,
            coral_riddles_right: 0,
            coral_riddles_seen: 0,
            slums_riddles_right: 0,
            slums_riddles_seen: 0,
            gate_solved: false,
            maze_solved: false,
            algae_solved: false,
            rainbow_solved: false,
            items: [false; 3],
            clues: Vec::new(),
            elixir: 0,
            done: std::collections::HashSet::new(),
        }
    }

    pub fn add_clue(&mut self, clue: &'static str) {
        if !self.clues.contains(&clue) {
            self.clues.push(clue);
        }
    }

    pub fn total_riddles_right(&self) -> u32 {
        self.kelp_riddles_right + self.coral_riddles_right + self.slums_riddles_right
    }

    pub fn total_riddles_seen(&self) -> u32 {
        self.kelp_riddles_seen + self.coral_riddles_seen + self.slums_riddles_seen
    }

    pub fn all_coral_items(&self) -> bool {
        self.items[0] && self.items[1] && self.items[2]
    }
}

static SWIFT_SWIM: AtomicBool = AtomicBool::new(false);

pub fn set_swift_swim(on: bool) {
    SWIFT_SWIM.store(on, Ordering::Relaxed);
}

pub fn swift_swim_unlocked() -> bool {
    SWIFT_SWIM.load(Ordering::Relaxed)
}

/// Drive a batch of async asset futures concurrently so web builds issue parallel HTTP fetches.
pub async fn join_all<T, F>(futs: Vec<F>) -> Vec<T>
where
    F: std::future::Future<Output = T>,
{
    let mut slots: Vec<Option<std::pin::Pin<Box<F>>>> = futs.into_iter().map(|f| Some(Box::pin(f))).collect();
    let mut out: Vec<Option<T>> = (0..slots.len()).map(|_| None).collect();
    std::future::poll_fn(move |cx| {
        let mut left = 0usize;
        for (i, slot) in slots.iter_mut().enumerate() {
            if let Some(fut) = slot {
                match fut.as_mut().poll(cx) {
                    std::task::Poll::Ready(val) => {
                        out[i] = Some(val);
                        *slot = None;
                    }
                    std::task::Poll::Pending => left += 1,
                }
            }
        }
        if left == 0 {
            std::task::Poll::Ready(out.iter_mut().map(|o| o.take().unwrap()).collect())
        } else {
            std::task::Poll::Pending
        }
    })
    .await
}
