//! Save/load run progress to a small local file next to the executable (no-op on wasm).

use crate::game::script::Biome;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BiomeId {
    Kelp = 0,
    Coral = 1,
    Slums = 2,
    Home = 3,
}

impl BiomeId {
    pub fn from_biome(b: Biome) -> Self {
        match b {
            Biome::Kelp => BiomeId::Kelp,
            Biome::Coral => BiomeId::Coral,
            Biome::Slums => BiomeId::Slums,
            Biome::Home => BiomeId::Home,
        }
    }

    pub fn to_biome(self) -> Biome {
        match self {
            BiomeId::Kelp => Biome::Kelp,
            BiomeId::Coral => Biome::Coral,
            BiomeId::Slums => Biome::Slums,
            BiomeId::Home => Biome::Home,
        }
    }
}

pub struct SaveData {
    pub swift_swim: bool,
    pub kelp_riddles: u32,
    pub coral_riddles: u32,
    pub slums_riddles: u32,
    pub gate_solved: bool,
    pub maze_solved: bool,
    pub elixir: u32,
    pub biome: BiomeId,
    pub player_x: f32,
}

pub fn save_path() -> std::path::PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("assets/game").is_dir() {
            return cwd.join("save.txt");
        }
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(|d| std::path::PathBuf::from(d).join("GiftOfEducation"));
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(|d| std::path::PathBuf::from(d).join("gift-of-education"))
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share/gift-of-education"))
        });

    if let Some(dir) = base {
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("save.txt");
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("save.txt")))
        .unwrap_or_else(|| std::path::PathBuf::from("save.txt"))
}

pub fn save(data: &SaveData) -> bool {
    let txt = format!(
        "swift={}\nkelp={}\ncoral={}\nslums={}\ngate={}\nmaze={}\nelixir={}\nbiome={}\nx={:.0}\n",
        data.swift_swim as i32,
        data.kelp_riddles,
        data.coral_riddles,
        data.slums_riddles,
        data.gate_solved as i32,
        data.maze_solved as i32,
        data.elixir,
        data.biome as i32,
        data.player_x,
    );
    std::fs::write(save_path(), txt).is_ok()
}

pub fn load() -> Option<SaveData> {
    let txt = std::fs::read_to_string(save_path()).ok()?;
    let mut d = SaveData {
        swift_swim: false,
        kelp_riddles: 0,
        coral_riddles: 0,
        slums_riddles: 0,
        gate_solved: false,
        maze_solved: false,
        elixir: 0,
        biome: BiomeId::Kelp,
        player_x: 140.0,
    };
    for line in txt.lines() {
        let (k, v) = line.split_once('=')?;
        match k {
            "swift" => d.swift_swim = v == "1",
            "kelp" => d.kelp_riddles = v.parse().unwrap_or(0),
            "coral" => d.coral_riddles = v.parse().unwrap_or(0),
            "slums" => d.slums_riddles = v.parse().unwrap_or(0),
            "gate" => d.gate_solved = v == "1",
            "maze" => d.maze_solved = v == "1",
            "elixir" => d.elixir = v.parse().unwrap_or(0),
            "biome" => {
                d.biome = match v.parse::<i32>().unwrap_or(0) {
                    1 => BiomeId::Coral,
                    2 => BiomeId::Slums,
                    3 => BiomeId::Home,
                    _ => BiomeId::Kelp,
                }
            }
            "x" => d.player_x = v.parse().unwrap_or(140.0),
            _ => {}
        }
    }
    Some(d)
}

pub fn clear() {
    let _ = std::fs::remove_file(save_path());
}
