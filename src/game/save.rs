//! Save/load run progress to a small local file next to the game.

use crate::game::script::Biome;

#[derive(Clone, Copy, PartialEq)]
pub enum BiomeId { Candy = 0, Aquarius = 1, Cosma = 2, Home = 3 }

impl BiomeId {
    pub fn from_biome(b: Biome) -> Self {
        match b {
            Biome::Candy => BiomeId::Candy,
            Biome::Aquarius => BiomeId::Aquarius,
            Biome::Cosma => BiomeId::Cosma,
            Biome::Home => BiomeId::Home,
        }
    }
    pub fn to_biome(self) -> Biome {
        match self {
            BiomeId::Candy => Biome::Candy,
            BiomeId::Aquarius => Biome::Aquarius,
            BiomeId::Cosma => Biome::Cosma,
            BiomeId::Home => Biome::Home,
        }
    }
}

pub struct SaveData {
    pub has_horse: bool,
    pub candy_riddles: u32,
    pub aqua_riddles: u32,
    pub cosma_riddles: u32,
    pub gate_solved: bool,
    pub maze_solved: bool,
    pub ingredients: u32,
    pub biome: BiomeId,
    pub player_x: f32,
}

pub fn save_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("save.txt")
}

pub fn save(data: &SaveData) -> bool {
    let txt = format!(
        "horse={}\ncandy={}\naqua={}\ncosma={}\ngate={}\nmaze={}\ningredients={}\nbiome={}\nx={:.0}\n",
        data.has_horse as i32,
        data.candy_riddles,
        data.aqua_riddles,
        data.cosma_riddles,
        data.gate_solved as i32,
        data.maze_solved as i32,
        data.ingredients,
        data.biome as i32,
        data.player_x,
    );
    std::fs::write(save_path(), txt).is_ok()
}

pub fn load() -> Option<SaveData> {
    let txt = std::fs::read_to_string(save_path()).ok()?;
    let mut d = SaveData {
        has_horse: false, candy_riddles: 0, aqua_riddles: 0, cosma_riddles: 0,
        gate_solved: false, maze_solved: false, ingredients: 0,
        biome: BiomeId::Candy, player_x: 140.0,
    };
    for line in txt.lines() {
        let (k, v) = line.split_once('=')?;
        match k {
            "horse" => d.has_horse = v == "1",
            "candy" => d.candy_riddles = v.parse().unwrap_or(0),
            "aqua" => d.aqua_riddles = v.parse().unwrap_or(0),
            "cosma" => d.cosma_riddles = v.parse().unwrap_or(0),
            "gate" => d.gate_solved = v == "1",
            "maze" => d.maze_solved = v == "1",
            "ingredients" => d.ingredients = v.parse().unwrap_or(0),
            "biome" => d.biome = match v.parse::<i32>().unwrap_or(0) {
                1 => BiomeId::Aquarius,
                2 => BiomeId::Cosma,
                3 => BiomeId::Home,
                _ => BiomeId::Candy,
            },
            "x" => d.player_x = v.parse().unwrap_or(140.0),
            _ => {}
        }
    }
    Some(d)
}

pub fn clear() {
    let _ = std::fs::remove_file(save_path());
}
