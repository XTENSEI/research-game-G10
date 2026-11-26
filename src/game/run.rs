//! Core game loop, phase state machine, minigames, dialogue routing, and QA hooks.

use std::collections::VecDeque;
use macroquad::prelude::*;

use crate::art::{c, fx::Fx, Textures};
use crate::audio::{AudioSys, MusicTrack};
use crate::game::{
    cosma, gate, kenny_battle, maze, save,
    script::{self, Biome, Line},
    set_swift_swim,
    ui::{
        draw_btn, draw_btn_icon, draw_dlg_shadow, draw_field_notes, draw_nine_slice,
        draw_option_row_skinned, draw_panel, draw_pause_overlay, draw_tag_shadow, draw_wrapped_col,
        draw_wrapped_rich, dlg_width, quiz_header_skinned, tag_width, wrapped_line_count,
        DialogueBox, PauseAction, Skin,
    },
    world::World,
    Progress,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum AfterDialogue {
    None,
    StartQte,
    StartKennyTree,
    AskKennyWeapons,
    StartKennyBattle(bool),
    AskKennyRetry,
    AskQueenEval,
    LaunchQueenTrap,
    AskLadyClue,
    ToBiome(Biome),
    StartRapBattle,
    StartMaze,
    StartAlgaeTemple,
    StartStars,
    StartCrumble,
    FinishEnding,
}

enum Phase {
    Title,
    Intro {
        idx: usize,
        timer: f32,
    },
    Explore,
    Dialogue {
        queue: VecDeque<Line>,
        after: AfterDialogue,
    },
    CrabQte {
        timer: f32,
        prog: f32,
    },
    KennyTree {
        step: u8,
    },
    KennyBattle {
        q_idx: usize,
        round_dodge: bool,
        timer: f32,
        hp: u8,
        boss_hp: u8,
        fewer: bool,
        flawless: bool,
        heart: (f32, f32),
        bullets: Vec<(f32, f32, f32, f32)>,
        hit_cd: f32,
        sel: usize,
        feedback: Option<(bool, &'static str, f32)>,
    },
    Riddle {
        idx: usize,
        sel: usize,
        picked: Option<usize>,
    },
    Gate {
        picks: [Option<usize>; 14],
        active_slot: Option<usize>,
        feedback: Option<(usize, String)>,
    },
    RapBattle {
        bar: usize,
        beat_t: f32,
        score: u32,
        notes: Vec<(f32, usize, bool)>,
        sel: usize,
        feedback: Option<(&'static str, f32)>,
    },
    Maze {
        depth: u8,
        q_idx: usize,
        sel: usize,
        pool_pos: usize,
        order: [usize; 8],
        dir_order: [maze::Dir; 4],
    },
    AlgaeTemple {
        stage: usize,
        sel: usize,
        feedback: Option<(bool, &'static str, f32)>,
    },
    StarCatch {
        caught: u32,
        lives: u8,
        basket_x: f32,
        spawn_t: f32,
        stars: Vec<(f32, f32, f32, bool)>,
    },
    TorchPlace {
        lit: [bool; 3],
    },
    GenieQuiz {
        poem_idx: usize,
        q_idx: usize,
        sel: usize,
        feedback: Option<(bool, &'static str, f32)>,
    },
    Crumble {
        dist: f32,
        py: f32,
        vy: f32,
        lives: u8,
        hit_cd: f32,
        spawn_t: f32,
        shards: Vec<(f32, f32, f32, bool)>,
    },
    Ending,
}

pub struct Game {
    tex: Textures,
    fx: Fx,
    audio: AudioSys,
    dlg: DialogueBox,
    world: World,
    prog: Progress,
    phase: Phase,
    biome: Biome,
    next_biome: Biome,
    fade: f32,
    show_notes: bool,
    paused: bool,
    has_save: bool,
    anglers_cleared: bool,
    towtou_ready: bool,
    autoplay_on: bool,
    auto_t: f32,
    auto_seen: std::collections::HashSet<String>,
    last_chars: usize,
}

impl Game {
    pub async fn new() -> Self {
        crate::game::ui::ensure_fonts().await;
        let tex = Textures::load().await;
        let fx = Fx::new(screen_width().max(1024.0), screen_height().max(720.0)).await;
        let audio = crate::audio::load().await;
        let world = World::new(Biome::Kelp, &tex);
        let has_save = save::load().is_some();

        let mut g = Self {
            tex,
            fx,
            audio,
            dlg: DialogueBox::new(),
            world,
            prog: Progress::new(),
            phase: Phase::Title,
            biome: Biome::Kelp,
            next_biome: Biome::Kelp,
            fade: 1.0,
            show_notes: false,
            paused: false,
            has_save,
            anglers_cleared: false,
            towtou_ready: false,
            autoplay_on: false,
            auto_t: 0.0,
            auto_seen: std::collections::HashSet::new(),
            last_chars: 0,
        };
        g.audio.ensure_music(MusicTrack::Town);
        g
    }

    pub fn autoplay(&mut self) {
        self.autoplay_on = true;
    }

    pub fn shot_image(&self) -> Image {
        self.fx.capture()
    }

    pub fn qa_dir(&mut self, dx: f32, dy: f32) {
        self.world.bot_dir = dx;
        self.world.bot_y = dy;
    }

    pub fn qa_enable_bot(&mut self) {
        self.autoplay_on = true;
    }

    pub fn qa_player(&self) -> (f32, f32) {
        self.world.player
    }

    pub fn qa_world_len(&self) -> f32 {
        self.world.length
    }

    pub fn qa_biome_name(&self) -> &'static str {
        script::biome_title(self.biome)
    }

    pub fn qa_teleport(&mut self, x: f32) {
        self.world.player.0 = x.clamp(40.0, self.world.length - 40.0);
        self.world.click = None;
    }

    pub fn qa_click(&mut self, x: f32) {
        self.world.click = Some(x);
    }

    pub fn qa_click_target(&self) -> Option<f32> {
        self.world.click
    }

    pub fn qa_pose(&self) -> String {
        format!(
            "x={:.1} y={:.1} facing={:+.0} moving={} bot={:.1}",
            self.world.player.0,
            self.world.player.1,
            self.world.facing,
            self.world.moving,
            self.world.drawn_bottom.get()
        )
    }

    pub fn qa_state(&self) -> String {
        let ph = match &self.phase {
            Phase::Title => "title",
            Phase::Intro { .. } => "intro",
            Phase::Explore => "explore",
            Phase::Dialogue { .. } => "dialogue",
            Phase::CrabQte { .. } => "crab_qte",
            Phase::KennyTree { .. } => "kenny_tree",
            Phase::KennyBattle { .. } => "kenny_battle",
            Phase::Riddle { .. } => "riddle",
            Phase::Gate { .. } => "gate",
            Phase::RapBattle { .. } => "rap_battle",
            Phase::Maze { .. } => "maze",
            Phase::AlgaeTemple { .. } => "algae_temple",
            Phase::StarCatch { .. } => "stars",
            Phase::TorchPlace { .. } => "torches",
            Phase::GenieQuiz { .. } => "genie",
            Phase::Crumble { .. } => "crumble",
            Phase::Ending => "ending",
        };
        format!("phase={} biome={:?} elixir={}", ph, self.biome, self.prog.elixir)
    }

    pub fn boot_candy(&mut self) {
        self.enter_biome(Biome::Kelp, false);
    }
    pub fn boot_aqua(&mut self) {
        self.enter_biome(Biome::Coral, false);
    }
    pub fn boot_cosma(&mut self) {
        self.enter_biome(Biome::Slums, false);
    }
    pub fn demo_dialog(&mut self) {
        self.enter_biome(Biome::Kelp, false);
        self.world.player.0 = 500.0;
        self.queue_lines(&[Line::expr("Kenny", "happy", "Look friend, *questions* and sharp reading settle every duel down here!")], AfterDialogue::None);
    }

    /// Boot any scene cleanly by name for native CLI or headless WebGL QA.
    pub fn qa_boot(&mut self, name: &str) {
        self.paused = false;
        self.show_notes = false;
        self.dlg.close();
        self.fade = 1.0;
        self.fx.iris_open();
        match name {
            "title" => {
                self.phase = Phase::Title;
                self.audio.ensure_music(MusicTrack::Town);
            }
            "intro" => {
                self.phase = Phase::Intro { idx: 0, timer: 0.0 };
                let (_, who, txt) = script::INTRO_PLATES[0];
                self.dlg.say_expr(who, "sick", txt, &self.tex);
            }
            "kelp" | "candy" => {
                self.enter_biome(Biome::Kelp, false);
                self.world.player.0 = 440.0;
            }
            "crab_qte" => {
                self.enter_biome(Biome::Kelp, false);
                self.phase = Phase::CrabQte { timer: 5.5, prog: 0.42 };
                self.audio.ensure_music(MusicTrack::Battle);
            }
            "kenny_battle" => {
                self.enter_biome(Biome::Kelp, false);
                self.start_kenny_battle(false);
            }
            "riddle" => {
                self.enter_biome(Biome::Kelp, false);
                self.phase = Phase::Riddle { idx: 0, sel: 2, picked: None };
            }
            "gate" => {
                self.enter_biome(Biome::Kelp, false);
                self.prog.done.insert("riddle_0");
                self.phase = Phase::Gate {
                    picks: [None; 14],
                    active_slot: Some(0),
                    feedback: None,
                };
            }
            "guards" => {
                self.enter_biome(Biome::Kelp, false);
                self.world.player.0 = 2280.0;
                self.queue_lines(script::GUARDS_SCENE, AfterDialogue::None);
            }
            "queen_crab" => {
                self.enter_biome(Biome::Kelp, false);
                self.prog.gate_solved = true;
                self.world.player.0 = 3200.0;
                self.queue_lines(&script::QUEEN_SUCK_REPLY[..1], AfterDialogue::None);
            }
            "coral" | "aqua" | "swim" => {
                self.enter_biome(Biome::Coral, false);
                self.world.player.0 = 1380.0;
            }
            "rap_battle" => {
                self.enter_biome(Biome::Coral, false);
                self.world.player.0 = 3680.0;
                self.start_rap_battle();
            }
            "maze" => {
                self.enter_biome(Biome::Coral, false);
                let (order, dir_order) = maze::shuffled_cycle();
                self.phase = Phase::Maze {
                    depth: 1,
                    q_idx: 0,
                    sel: 0,
                    pool_pos: 1,
                    order,
                    dir_order,
                };
                self.audio.ensure_music(MusicTrack::Cave);
            }
            "slums" | "cosma" => {
                self.enter_biome(Biome::Slums, false);
                self.world.player.0 = 1220.0;
            }
            "algae_temple" => {
                self.enter_biome(Biome::Slums, false);
                self.prog.elixir = 1;
                self.phase = Phase::AlgaeTemple { stage: 0, sel: 0, feedback: None };
            }
            "stars" => {
                self.enter_biome(Biome::Slums, false);
                self.phase = Phase::StarCatch {
                    caught: 6,
                    lives: 3,
                    basket_x: 512.0,
                    spawn_t: 0.2,
                    stars: vec![(320.0, 220.0, 150.0, false), (580.0, 160.0, 170.0, true), (740.0, 260.0, 140.0, false)],
                };
                self.audio.ensure_music(MusicTrack::Rainbow);
            }
            "genie" => {
                self.enter_biome(Biome::Slums, false);
                self.world.torches_lit = 3;
                self.phase = Phase::GenieQuiz { poem_idx: 0, q_idx: 0, sel: 0, feedback: None };
                self.audio.ensure_music(MusicTrack::Rainbow);
            }
            "crumble" => {
                self.enter_biome(Biome::Slums, false);
                self.phase = Phase::Crumble {
                    dist: 0.35,
                    py: 500.0,
                    vy: 0.0,
                    lives: 3,
                    hit_cd: 0.0,
                    spawn_t: 0.3,
                    shards: vec![(760.0, 480.0, 320.0, false), (920.0, 320.0, 290.0, true)],
                };
                self.audio.ensure_music(MusicTrack::Battle);
            }
            "ending" | "home" => {
                self.enter_biome(Biome::Home, false);
                self.prog.elixir = 3;
                self.prog.kelp_riddles_seen = 3;
                self.prog.kelp_riddles_right = 3;
                self.prog.coral_riddles_seen = 3;
                self.prog.coral_riddles_right = 2;
                self.prog.slums_riddles_seen = 2;
                self.prog.slums_riddles_right = 2;
                self.prog.gate_solved = true;
                self.prog.maze_solved = true;
                self.prog.algae_solved = true;
                self.prog.rainbow_solved = true;
                self.phase = Phase::Ending;
                self.audio.ensure_music(MusicTrack::Town);
            }
            "autoplay" => {
                self.autoplay_on = true;
                self.start_new_game();
            }
            _ => {}
        }
    }

    fn start_new_game(&mut self) {
        self.prog = Progress::new();
        set_swift_swim(false);
        self.anglers_cleared = false;
        self.towtou_ready = false;
        save::clear();
        self.phase = Phase::Intro { idx: 0, timer: 0.0 };
        let (_, who, txt) = script::INTRO_PLATES[0];
        self.dlg.say_expr(who, "sick", txt, &self.tex);
        self.audio.ensure_music(MusicTrack::Town);
    }

    fn enter_biome(&mut self, biome: Biome, with_scene: bool) {
        self.biome = biome;
        self.next_biome = biome;
        self.world.populate(biome, &self.tex);
        self.fade = 0.0;
        self.fx.iris_open();
        self.fx.punch(0.032);
        self.fx.grade = match biome {
            Biome::Kelp => Color::new(0.02, 0.14, 0.14, 0.08),
            Biome::Coral => Color::new(0.02, 0.10, 0.20, 0.08),
            Biome::Slums => Color::new(0.06, 0.03, 0.14, 0.12),
            Biome::Home => Color::new(0.18, 0.10, 0.06, 0.06),
        };
        let track = match biome {
            Biome::Kelp => MusicTrack::Kelp,
            Biome::Coral => MusicTrack::Coral,
            Biome::Slums => MusicTrack::Slums,
            Biome::Home => MusicTrack::Town,
        };
        self.auto_seen.clear();
        self.audio.ensure_music(track);
        self.save_checkpoint();

        if with_scene {
            match biome {
                Biome::Kelp => self.queue_lines(script::KELP_ARRIVE, AfterDialogue::StartQte),
                Biome::Coral => self.queue_lines(script::AQUA_WAKE, AfterDialogue::None),
                Biome::Slums => self.queue_lines(script::SLUMS_ARRIVE, AfterDialogue::None),
                Biome::Home => self.queue_lines(script::HOME_ENDING, AfterDialogue::FinishEnding),
            }
        } else {
            self.phase = Phase::Explore;
        }
    }

    fn save_checkpoint(&mut self) {
        self.has_save = save::save(&save::SaveData {
            swift_swim: self.prog.kenny_flawless,
            kelp_riddles: self.prog.kelp_riddles_seen,
            coral_riddles: self.prog.coral_riddles_seen,
            slums_riddles: self.prog.slums_riddles_seen,
            gate_solved: self.prog.gate_solved,
            maze_solved: self.prog.maze_solved,
            elixir: self.prog.elixir,
            biome: save::BiomeId::from_biome(self.biome),
            player_x: self.world.player.0,
        });
    }

    fn load_checkpoint(&mut self) {
        if let Some(d) = save::load() {
            self.prog = Progress::new();
            self.prog.kenny_flawless = d.swift_swim;
            set_swift_swim(d.swift_swim);
            self.prog.kelp_riddles_seen = d.kelp_riddles;
            self.prog.coral_riddles_seen = d.coral_riddles;
            self.prog.slums_riddles_seen = d.slums_riddles;
            self.prog.gate_solved = d.gate_solved;
            self.prog.maze_solved = d.maze_solved;
            self.prog.elixir = d.elixir;
            let b = d.biome.to_biome();
            self.enter_biome(b, false);
            self.world.player.0 = d.player_x;
        }
    }

    fn queue_lines(&mut self, lines: &[Line], after: AfterDialogue) {
        let mut q: VecDeque<Line> = lines.iter().copied().collect();
        if let Some(first) = q.pop_front() {
            self.dlg.say_expr(first.who, first.expr, first.text, &self.tex);
            self.last_chars = 0;
            self.phase = Phase::Dialogue { queue: q, after };
        }
    }

    fn start_kenny_battle(&mut self, fewer: bool) {
        self.audio.ensure_music(MusicTrack::Battle);
        self.phase = Phase::KennyBattle {
            q_idx: 0,
            round_dodge: false,
            timer: 0.0,
            hp: 4,
            boss_hp: kenny_battle::questions_for(fewer).len() as u8,
            fewer,
            flawless: true,
            heart: (512.0, 440.0),
            bullets: Vec::new(),
            hit_cd: 0.0,
            sel: 0,
            feedback: None,
        };
    }

    fn start_rap_battle(&mut self) {
        self.audio.ensure_music(MusicTrack::Battle);
        let mut notes = Vec::new();
        for i in 0..10 {
            notes.push((0.8 + i as f32 * 0.72, i % 4, false));
        }
        self.phase = Phase::RapBattle {
            bar: 0,
            beat_t: 0.0,
            score: 0,
            notes,
            sel: 0,
            feedback: None,
        };
    }

    pub fn update(&mut self, dt_raw: f32) {
        self.fx.tick(dt_raw);
        let dt = if self.fx.freeze > 0.0 { dt_raw * 0.08 } else { dt_raw.min(0.05) };

        if self.fade < 1.0 {
            self.fade = (self.fade + dt_raw * 2.2).min(1.0);
        }

        if is_key_pressed(KeyCode::M) {
            self.audio.toggle_mute();
        }

        // Global UI overlays inside Explore
        if matches!(self.phase, Phase::Explore) {
            if is_key_pressed(KeyCode::Tab) && !self.paused {
                self.show_notes = !self.show_notes;
                self.audio.play_sfx(&self.audio.sfx.page, 0.55);
            }
            if is_key_pressed(KeyCode::Escape) {
                if self.show_notes {
                    self.show_notes = false;
                } else {
                    self.paused = !self.paused;
                }
            }
        }

        // Voice typewriter sound cadence
        let shown = self.dlg.chars_shown();
        if shown > self.last_chars && shown % 3 == 0 {
            self.audio.play_voice(self.dlg.speaker());
        }
        self.last_chars = shown;

        // Smooth vertical camera lift when dialogue box is open
        let want_lift = if self.dlg.open { 64.0 } else { 0.0 };
        self.fx.lift += (want_lift - self.fx.lift) * (dt_raw * 9.0).min(1.0);

        if self.paused || self.show_notes {
            return;
        }

        if self.autoplay_on {
            self.step_autoplay(dt);
        }

        self.step_phase(dt);
    }

    fn step_autoplay(&mut self, dt: f32) {
        self.auto_t += dt;
        if self.auto_t < 0.04 {
            return;
        }
        self.auto_t = 0.0;
        if self.dlg.open {
            if !self.dlg.is_finished() {
                self.dlg.force_advance = true;
            } else {
                self.dlg.force_advance = true;
            }
            return;
        }
        match &mut self.phase {
            Phase::Title => self.start_new_game(),
            Phase::CrabQte { prog, .. } => {
                *prog += 0.25;
            }
            Phase::KennyBattle { round_dodge, q_idx, fewer, boss_hp, .. } => {
                if !*round_dodge {
                    let qs = kenny_battle::questions_for(*fewer);
                    if *q_idx + 1 >= qs.len() || *boss_hp <= 1 {
                        self.prog.kenny_flawless = true;
                        set_swift_swim(true);
                        self.prog.done.insert("kenny");
                        self.enter_biome(Biome::Kelp, false);
                        self.world.player.0 = 1040.0;
                    } else {
                        *q_idx += 1;
                        *boss_hp = boss_hp.saturating_sub(1);
                    }
                }
            }
            Phase::Explore => {
                self.world.bot_dir = 1.0;
                if let Some(act) = self.world.nearest_action() {
                    if !self.prog.done.contains(act.as_str()) && !self.auto_seen.contains(act.as_str()) {
                        self.world.bot_dir = 0.0;
                        self.auto_seen.insert(act.clone());
                        self.trigger_action(&act);
                    }
                }
                if self.biome == Biome::Kelp && self.world.player.0 >= 3380.0 && !self.auto_seen.contains("queen_crab") {
                    self.world.bot_dir = 0.0;
                    self.auto_seen.insert("queen_crab".to_string());
                    self.trigger_action("queen_crab");
                }
                if self.biome == Biome::Home && self.world.player.0 >= 860.0 && !self.auto_seen.contains("nadia") {
                    self.world.bot_dir = 0.0;
                    self.auto_seen.insert("nadia".to_string());
                    self.trigger_action("nadia");
                }
            }
            Phase::Riddle { idx, .. } => {
                let id = *idx;
                let cor = script::RIDDLES[id].correct;
                self.resolve_riddle(id, cor);
                self.phase = Phase::Explore;
            }
            Phase::Gate { picks, .. } => {
                for (i, slot) in gate::SLOTS.iter().enumerate() {
                    let pi = gate::PALETTE.iter().position(|p| p.0 == slot.answer).unwrap_or(0);
                    picks[i] = Some(pi);
                }
                self.prog.gate_solved = true;
                self.prog.done.insert("gate");
                self.phase = Phase::Explore;
            }
            Phase::RapBattle { .. } => {
                self.anglers_cleared = true;
                self.prog.done.insert("anglers");
                self.phase = Phase::Explore;
            }
            Phase::Maze { .. } => {
                self.prog.maze_solved = true;
                self.enter_biome(Biome::Slums, false);
                self.world.player.0 = 1220.0;
            }
            Phase::AlgaeTemple { .. } => {
                self.prog.algae_solved = true;
                self.prog.done.insert("algae_temple");
                self.prog.elixir = 3;
                self.phase = Phase::Explore;
            }
            Phase::StarCatch { caught, .. } => {
                *caught = cosma::STAR_TARGET;
                self.phase = Phase::TorchPlace { lit: [true; 3] };
            }
            Phase::TorchPlace { .. } => {
                self.world.torches_lit = 3;
                self.phase = Phase::GenieQuiz { poem_idx: 0, q_idx: 0, sel: 0, feedback: None };
            }
            Phase::GenieQuiz { .. } => {
                self.prog.rainbow_solved = true;
                self.phase = Phase::Crumble {
                    dist: 0.96,
                    py: 500.0,
                    vy: 0.0,
                    lives: 3,
                    hit_cd: 0.0,
                    spawn_t: 0.5,
                    shards: Vec::new(),
                };
            }
            Phase::Crumble { dist, .. } => {
                *dist = 1.01;
            }
            _ => {}
        }
    }

    fn step_phase(&mut self, dt: f32) {
        if let Some(res) = self.dlg.update(dt) {
            self.audio.play_sfx(&self.audio.sfx.select, 0.55);
            self.handle_dialogue_result(res);
            return;
        }

        let phase = std::mem::replace(&mut self.phase, Phase::Explore);
        match phase {
            Phase::Title => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.audio.play_sfx(&self.audio.sfx.select, 0.7);
                    self.start_new_game();
                    return;
                }
                self.phase = Phase::Title;
            }
            Phase::Intro { mut idx, mut timer } => {
                timer += dt;
                if !self.dlg.open {
                    idx += 1;
                    if idx >= script::INTRO_PLATES.len() {
                        self.audio.play_sfx(&self.audio.sfx.splash, 0.7);
                        self.fx.flash(Color::new(0.45, 0.85, 1.0, 1.0), 0.45);
                        self.enter_biome(Biome::Kelp, true);
                        return;
                    }
                    let (_, who, txt) = script::INTRO_PLATES[idx];
                    self.dlg.say_expr(who, "sick", txt, &self.tex);
                    timer = 0.0;
                }
                self.phase = Phase::Intro { idx, timer };
            }
            Phase::Dialogue { queue, after } => {
                self.phase = Phase::Dialogue { queue, after };
            }
            Phase::Explore => {
                if let Some(act) = self.world.update(dt) {
                    self.trigger_action(&act);
                    return;
                }
                if self.world.step_hit {
                    self.world.step_hit = false;
                    let base = match self.biome {
                        Biome::Kelp => 0,
                        Biome::Coral => 2,
                        Biome::Slums => 4,
                        Biome::Home => 6,
                    };
                    let idx = base + ((get_time() * 7.0) as usize % 2);
                    if idx < self.audio.sfx.steps.len() {
                        self.audio.play_sfx(&self.audio.sfx.steps[idx], 0.42);
                    }
                }
                self.phase = Phase::Explore;
            }
            Phase::CrabQte { mut timer, mut prog } => {
                timer -= dt;
                if is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Z)
                    || is_key_pressed(KeyCode::Enter)
                    || is_mouse_button_pressed(MouseButton::Left)
                {
                    prog = (prog + 0.14).min(1.0);
                    self.audio.play_sfx(&self.audio.sfx.click, 0.45);
                    self.fx.punch(0.012);
                } else {
                    prog = (prog - dt * 0.09).max(0.0);
                }
                if prog >= 1.0 {
                    self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                    self.audio.ensure_music(MusicTrack::Kelp);
                    self.queue_lines(script::KENNY_QTE_PASS, AfterDialogue::StartKennyTree);
                } else if timer <= 0.0 {
                    self.audio.play_sfx(&self.audio.sfx.bad, 0.55);
                    self.audio.ensure_music(MusicTrack::Kelp);
                    self.queue_lines(script::KENNY_QTE_FAIL, AfterDialogue::StartKennyTree);
                } else {
                    self.phase = Phase::CrabQte { timer, prog };
                }
            }
            Phase::KennyTree { step } => {
                self.phase = Phase::KennyTree { step };
            }
            Phase::KennyBattle {
                mut q_idx,
                mut round_dodge,
                mut timer,
                mut hp,
                mut boss_hp,
                fewer,
                mut flawless,
                mut heart,
                mut bullets,
                mut hit_cd,
                mut sel,
                mut feedback,
            } => {
                if let Some((good, msg, mut ft)) = feedback {
                    ft -= dt;
                    if ft <= 0.0 {
                        if boss_hp == 0 {
                            self.prog.done.insert("kenny");
                            self.prog.kenny_flawless = flawless;
                            set_swift_swim(flawless);
                            self.audio.ensure_music(MusicTrack::Kelp);
                            if flawless {
                                self.queue_lines(script::KENNY_FLAWLESS_WIN, AfterDialogue::None);
                            } else {
                                self.queue_lines(script::KENNY_EVEN_WIN, AfterDialogue::None);
                            }
                            return;
                        }
                        if hp == 0 {
                            self.audio.ensure_music(MusicTrack::Kelp);
                            self.dlg.ask_expr(
                                "Kenny",
                                "worried",
                                script::KENNY_RETRY_PROMPT,
                                &[("Challenge him (fewer questions)", "retry"), ("Step back for now", "quit")],
                                &self.tex,
                            );
                            self.phase = Phase::Dialogue {
                                queue: VecDeque::new(),
                                after: AfterDialogue::AskKennyRetry,
                            };
                            return;
                        }
                        feedback = None;
                        if good {
                            round_dodge = true;
                            timer = 4.2;
                            bullets.clear();
                            for b in 0..8 {
                                let bx = 340.0 + (b % 4) as f32 * 96.0;
                                let by = 348.0 + (b / 4) as f32 * 140.0;
                                let vx = if b % 2 == 0 { 130.0 } else { -135.0 };
                                let vy = if b < 4 { 105.0 } else { -110.0 };
                                bullets.push((bx, by, vx, vy));
                            }
                        }
                    } else {
                        feedback = Some((good, msg, ft));
                    }
                } else if round_dodge {
                    timer -= dt;
                    hit_cd = (hit_cd - dt).max(0.0);
                    let mut hx = 0.0f32;
                    let mut hy = 0.0f32;
                    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { hx -= 1.0; }
                    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { hx += 1.0; }
                    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) { hy -= 1.0; }
                    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { hy += 1.0; }
                    heart.0 = (heart.0 + hx * 240.0 * dt).clamp(328.0, 696.0);
                    heart.1 = (heart.1 + hy * 240.0 * dt).clamp(352.0, 520.0);

                    for (bx, by, vx, vy) in &mut bullets {
                        *bx += *vx * dt;
                        *by += *vy * dt;
                        if *bx < 322.0 || *bx > 702.0 { *vx = -*vx; }
                        if *by < 346.0 || *by > 526.0 { *vy = -*vy; }
                        if hit_cd <= 0.0 && (*bx - heart.0).hypot(*by - heart.1) < 16.0 {
                            hit_cd = 0.85;
                            hp = hp.saturating_sub(1);
                            flawless = false;
                            self.audio.play_sfx(&self.audio.sfx.bad, 0.6);
                            self.fx.accent(false);
                        }
                    }
                    if hp == 0 {
                        feedback = Some((false, "Ouch, shelled out!", 0.9));
                    } else if timer <= 0.0 {
                        round_dodge = false;
                        q_idx += 1;
                    }
                } else {
                    let qs = kenny_battle::questions_for(fewer);
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) { sel = sel.saturating_sub(1); }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) { sel = (sel + 1).min(2); }
                    for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3].iter().enumerate() {
                        if is_key_pressed(*key) { sel = k; }
                    }
                    let qs_len = qs.len();
                    let cur_q = &qs[q_idx.min(qs_len - 1)];
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Space) {
                        if sel == cur_q.correct {
                            boss_hp = boss_hp.saturating_sub(1);
                            self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                            self.fx.accent(true);
                            feedback = Some((true, "Direct literacy strike!!", 0.9));
                        } else {
                            hp = hp.saturating_sub(1);
                            flawless = false;
                            self.audio.play_sfx(&self.audio.sfx.bad, 0.55);
                            self.fx.accent(false);
                            feedback = Some((false, "Misread that strike! Guard up!", 0.9));
                        }
                    }
                }
                self.phase = Phase::KennyBattle {
                    q_idx, round_dodge, timer, hp, boss_hp, fewer, flawless, heart, bullets, hit_cd, sel, feedback,
                };
            }
            Phase::Riddle { idx, mut sel, mut picked } => {
                if picked.is_none() {
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) { sel = sel.saturating_sub(1); }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) { sel = (sel + 1).min(3); }
                    for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].iter().enumerate() {
                        if is_key_pressed(*key) { sel = k; }
                    }
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Space) {
                        picked = Some(sel);
                        self.resolve_riddle(idx, sel);
                    }
                } else if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Z)
                    || is_key_pressed(KeyCode::Space)
                    || is_mouse_button_pressed(MouseButton::Left)
                {
                    self.world.interact_cd = 0.25;
                    self.phase = Phase::Explore;
                    return;
                }
                self.phase = Phase::Riddle { idx, sel, picked };
            }
            Phase::Gate { picks, active_slot, feedback } => {
                if is_key_pressed(KeyCode::Escape) {
                    self.world.interact_cd = 0.25;
                    self.phase = Phase::Explore;
                    return;
                }
                self.phase = Phase::Gate { picks, active_slot, feedback };
            }
            Phase::RapBattle { mut bar, mut beat_t, mut score, mut notes, mut sel, mut feedback } => {
                beat_t += dt;
                if let Some((msg, mut ft)) = feedback {
                    ft -= dt;
                    if ft <= 0.0 {
                        feedback = None;
                    } else {
                        feedback = Some((msg, ft));
                    }
                }
                let lanes = [KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::F];
                for (li, key) in lanes.iter().enumerate() {
                    if is_key_pressed(*key) {
                        if let Some(n) = notes.iter_mut().find(|(nt, nl, hit)| !*hit && *nl == li && (*nt - beat_t).abs() < 0.28) {
                            n.2 = true;
                            score += 100;
                            self.audio.play_sfx(&self.audio.sfx.coin, 0.45);
                        }
                    }
                }
                for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3].iter().enumerate() {
                    if is_key_pressed(*key) { sel = k; }
                }
                if is_key_pressed(KeyCode::Up) { sel = sel.saturating_sub(1); }
                if is_key_pressed(KeyCode::Down) { sel = (sel + 1).min(2); }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let cur = &cosma::RAP_BARS[bar];
                    let right_idx = (bar * 2) % 3;
                    if sel == right_idx {
                        score += 250;
                        self.audio.play_sfx(&self.audio.sfx.good, 0.55);
                        bar += 1;
                        if bar >= cosma::RAP_BARS.len() {
                            self.anglers_cleared = true;
                            self.prog.done.insert("anglers");
                            self.audio.ensure_music(MusicTrack::Coral);
                            self.queue_lines(script::ANGLERS_WIN, AfterDialogue::None);
                            return;
                        }
                        feedback = Some(("Clean rhyme!! Keep flowing!", 0.8));
                    } else {
                        self.audio.play_sfx(&self.audio.sfx.bad, 0.45);
                        feedback = Some(("Off-rhythm bar! Match the end rhyme!", 0.9));
                    }
                    let _ = cur;
                }
                self.phase = Phase::RapBattle { bar, beat_t, score, notes, sel, feedback };
            }
            Phase::Maze { mut depth, mut q_idx, mut sel, mut pool_pos, mut order, mut dir_order } => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) { sel = sel.saturating_sub(1); }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) { sel = (sel + 1).min(3); }
                for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].iter().enumerate() {
                    if is_key_pressed(*key) { sel = k; }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Space) {
                    let q = &maze::QUESTIONS[order[q_idx % 8]];
                    if dir_order[sel] == q.dir {
                        depth += 1;
                        self.audio.play_sfx(&self.audio.sfx.click, 0.45);
                        if depth >= maze::MAZE_TARGET_DEPTH {
                            self.prog.maze_solved = true;
                            self.audio.play_sfx(&self.audio.sfx.good, 0.7);
                            self.fx.accent(true);
                            self.enter_biome(Biome::Slums, true);
                            return;
                        }
                    } else {
                        // Exact SCRIPT RES.pdf p.17 rule: silent step back with ZERO visual cue.
                        depth = depth.saturating_sub(1);
                    }
                    pool_pos = (pool_pos + 1) % 8;
                    if pool_pos == 0 {
                        order = maze::shuffled_cycle().0;
                    }
                    q_idx = pool_pos;
                    dir_order = maze::shuffled_dirs();
                    sel = 0;
                }
                self.phase = Phase::Maze { depth, q_idx, sel, pool_pos, order, dir_order };
            }
            Phase::AlgaeTemple { mut stage, mut sel, mut feedback } => {
                if let Some((good, msg, mut ft)) = feedback {
                    ft -= dt;
                    if ft <= 0.0 {
                        feedback = None;
                        if good {
                            stage += 1;
                            if stage >= cosma::ALGAE_TRIALS.len() {
                                self.prog.algae_solved = true;
                                self.prog.elixir = 3;
                                self.prog.done.insert("algae_temple");
                                self.audio.play_sfx(&self.audio.sfx.good, 0.7);
                                self.fx.accent(true);
                                self.queue_lines(script::ALGAE_TEMPLE_POST, AfterDialogue::None);
                                return;
                            }
                            sel = 0;
                        }
                    } else {
                        feedback = Some((good, msg, ft));
                    }
                } else {
                    if is_key_pressed(KeyCode::Up) { sel = sel.saturating_sub(1); }
                    if is_key_pressed(KeyCode::Down) { sel = (sel + 1).min(3); }
                    for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].iter().enumerate() {
                        if is_key_pressed(*key) { sel = k; }
                    }
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                        let tr = &cosma::ALGAE_TRIALS[stage];
                        if sel == tr.correct {
                            self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                            feedback = Some((true, tr.infuse_note, 1.3));
                        } else {
                            self.audio.play_sfx(&self.audio.sfx.bad, 0.5);
                            feedback = Some((false, "Crystal mismatch! Check the figure and poetic device!", 1.1));
                        }
                    }
                }
                self.phase = Phase::AlgaeTemple { stage, sel, feedback };
            }
            Phase::StarCatch { mut caught, mut lives, mut basket_x, mut spawn_t, mut stars } => {
                let mut mv = 0.0f32;
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { mv -= 1.0; }
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { mv += 1.0; }
                if is_mouse_button_down(MouseButton::Left) {
                    let (mx, _) = mouse_position();
                    basket_x += (mx.clamp(120.0, 904.0) - basket_x) * (dt * 12.0).min(1.0);
                } else {
                    basket_x = (basket_x + mv * 520.0 * dt).clamp(120.0, 904.0);
                }

                spawn_t -= dt;
                if spawn_t <= 0.0 {
                    spawn_t = 0.52;
                    let s = ((get_time() * 1000.0) as u64).wrapping_mul(2654435761);
                    let sx = 110.0 + (s % 804) as f32;
                    let spd = 175.0 + ((s >> 12) % 120) as f32;
                    let dying = (s >> 20) % 4 == 0;
                    stars.push((sx, -20.0, spd, dying));
                }

                for (_, sy, spd, _) in &mut stars {
                    *sy += *spd * dt;
                }
                let mut i = 0;
                while i < stars.len() {
                    let (sx, sy, _, dying) = stars[i];
                    if sy >= 596.0 && sy <= 646.0 && (sx - basket_x).abs() < 64.0 {
                        stars.remove(i);
                        if dying {
                            lives = lives.saturating_sub(1);
                            caught = caught.saturating_sub(cosma::DYING_PENALTY);
                            self.audio.play_sfx(&self.audio.sfx.bad, 0.6);
                            self.fx.accent(false);
                        } else {
                            caught += 1;
                            self.audio.play_sfx(&self.audio.sfx.coin, 0.55);
                        }
                        continue;
                    }
                    if sy > 740.0 {
                        stars.remove(i);
                        continue;
                    }
                    i += 1;
                }

                if lives == 0 {
                    self.world.interact_cd = 0.25;
                    self.world.say_hint("Dying stars dimmed your lantern! Approach Genie to try catching again.");
                    self.audio.ensure_music(MusicTrack::Slums);
                    self.phase = Phase::Explore;
                    return;
                }
                if caught >= cosma::STAR_TARGET {
                    self.audio.play_sfx(&self.audio.sfx.good, 0.7);
                    self.fx.accent(true);
                    self.phase = Phase::TorchPlace { lit: [false; 3] };
                    return;
                }
                self.phase = Phase::StarCatch { caught, lives, basket_x, spawn_t, stars };
            }
            Phase::TorchPlace { mut lit } => {
                if is_key_pressed(KeyCode::Key1) { lit[0] = true; self.audio.play_sfx(&self.audio.sfx.click, 0.5); }
                if is_key_pressed(KeyCode::Key2) { lit[1] = true; self.audio.play_sfx(&self.audio.sfx.click, 0.5); }
                if is_key_pressed(KeyCode::Key3) { lit[2] = true; self.audio.play_sfx(&self.audio.sfx.click, 0.5); }
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    if let Some(p) = lit.iter_mut().find(|v| !**v) {
                        *p = true;
                        self.audio.play_sfx(&self.audio.sfx.good, 0.5);
                    }
                }
                self.world.torches_lit = lit.iter().filter(|b| **b).count() as u32;
                if lit.iter().all(|b| *b) {
                    self.fx.accent(true);
                    self.phase = Phase::GenieQuiz { poem_idx: 0, q_idx: 0, sel: 0, feedback: None };
                    return;
                }
                self.phase = Phase::TorchPlace { lit };
            }
            Phase::GenieQuiz { mut poem_idx, mut q_idx, mut sel, mut feedback } => {
                if let Some((good, msg, mut ft)) = feedback {
                    ft -= dt;
                    if ft <= 0.0 {
                        feedback = None;
                        if good {
                            q_idx += 1;
                            if q_idx >= cosma::POEMS[poem_idx].questions.len() {
                                self.prog.rainbow_solved = true;
                                self.audio.play_sfx(&self.audio.sfx.good, 0.75);
                                self.fx.accent(true);
                                self.queue_lines(script::CRUMBLE_START, AfterDialogue::StartCrumble);
                                return;
                            }
                            sel = 0;
                        } else {
                            // Exact SCRIPT RES.pdf p.18 rule: failing Genie switches to another poem!
                            poem_idx = (poem_idx + 1) % cosma::POEMS.len();
                            q_idx = 0;
                            sel = 0;
                        }
                    } else {
                        feedback = Some((good, msg, ft));
                    }
                } else {
                    if is_key_pressed(KeyCode::Up) { sel = sel.saturating_sub(1); }
                    if is_key_pressed(KeyCode::Down) { sel = (sel + 1).min(3); }
                    for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].iter().enumerate() {
                        if is_key_pressed(*key) { sel = k; }
                    }
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                        let q = &cosma::POEMS[poem_idx].questions[q_idx];
                        if sel == q.correct {
                            self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                            feedback = Some((true, q.feedback, 1.3));
                        } else {
                            self.audio.play_sfx(&self.audio.sfx.bad, 0.6);
                            feedback = Some((false, "False interpretation! The Genie turns to a fresh poem...", 1.4));
                        }
                    }
                }
                self.phase = Phase::GenieQuiz { poem_idx, q_idx, sel, feedback };
            }
            Phase::Crumble { mut dist, mut py, mut vy, mut lives, mut hit_cd, mut spawn_t, mut shards } => {
                dist += dt * 0.085;
                hit_cd = (hit_cd - dt).max(0.0);
                if (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || is_mouse_button_pressed(MouseButton::Left)) && py >= 496.0 {
                    vy = -520.0;
                    self.audio.play_sfx(&self.audio.sfx.click, 0.5);
                }
                vy += 1280.0 * dt;
                py = (py + vy * dt).min(500.0);
                if py >= 500.0 {
                    vy = 0.0;
                }

                spawn_t -= dt;
                if spawn_t <= 0.0 {
                    spawn_t = 0.85;
                    let low = ((get_time() * 100.0) as u64) % 2 == 0;
                    shards.push((1060.0, if low { 485.0 } else { 370.0 }, 360.0, low));
                }
                for (sx, _, spd, _) in &mut shards {
                    *sx -= *spd * dt;
                }
                shards.retain(|(sx, _, _, _)| *sx > -80.0);

                for (sx, sy, _, _) in &shards {
                    if hit_cd <= 0.0 && (*sx - 240.0).abs() < 34.0 && (*sy - (py - 36.0)).abs() < 42.0 {
                        hit_cd = 0.9;
                        lives = lives.saturating_sub(1);
                        self.audio.play_sfx(&self.audio.sfx.bad, 0.6);
                        self.fx.accent(false);
                    }
                }
                if lives == 0 {
                    dist = 0.0;
                    lives = 3;
                    shards.clear();
                }
                if dist >= 1.0 {
                    self.audio.play_sfx(&self.audio.sfx.good, 0.75);
                    self.fx.flash(WHITE, 0.7);
                    self.enter_biome(Biome::Home, true);
                    return;
                }
                self.phase = Phase::Crumble { dist, py, vy, lives, hit_cd, spawn_t, shards };
            }
            Phase::Ending => {
                self.phase = Phase::Ending;
            }
        }
    }

    fn handle_dialogue_result(&mut self, res: String) {
        let phase = std::mem::replace(&mut self.phase, Phase::Explore);
        match phase {
            Phase::Intro { idx, timer } => {
                self.phase = Phase::Intro { idx, timer };
            }
            Phase::Dialogue { mut queue, after } => {
                if let Some(next) = queue.pop_front() {
                    self.dlg.say_expr(next.who, next.expr, next.text, &self.tex);
                    self.phase = Phase::Dialogue { queue, after };
                    return;
                }
                match after {
                    AfterDialogue::None => {
                        self.world.interact_cd = 0.25;
                        self.phase = Phase::Explore;
                    }
                    AfterDialogue::StartQte => {
                        self.audio.ensure_music(MusicTrack::Battle);
                        self.phase = Phase::CrabQte { timer: 6.5, prog: 0.15 };
                    }
                    AfterDialogue::StartKennyTree => {
                        self.dlg.ask_expr(
                            "Kenny",
                            "normal",
                            "Catch your breath! What did you just try asking?",
                            &[("Where am I?", "w1"), ("Where are we?", "w2"), ("No, stay back!", "w3")],
                            &self.tex,
                        );
                        self.phase = Phase::KennyTree { step: 1 };
                    }
                    AfterDialogue::AskKennyWeapons => {
                        self.dlg.ask_expr(
                            "Kenny",
                            "normal",
                            "In this world, our King prohibited weapons, so every fight settles by...",
                            &[("Risk?", "r"), ("Riddles?", "d"), ("Questions?", "q")],
                            &self.tex,
                        );
                        self.phase = Phase::KennyTree { step: 3 };
                    }
                    AfterDialogue::StartKennyBattle(fewer) => {
                        self.start_kenny_battle(fewer);
                    }
                    AfterDialogue::AskKennyRetry => {
                        if res == "retry" {
                            self.start_kenny_battle(true);
                        } else {
                            self.world.interact_cd = 0.25;
                            self.phase = Phase::Explore;
                        }
                    }
                    AfterDialogue::AskQueenEval => {
                        self.dlg.ask_expr(
                            "Queen Crab",
                            "normal",
                            script::QUEEN_EVAL_Q,
                            &[
                                ("\"I thought they were great!\"", "great"),
                                ("\"They were really easy.\"", "easy"),
                                ("\"Those puzzles sucked.\"", "sucked"),
                            ],
                            &self.tex,
                        );
                        self.phase = Phase::Dialogue {
                            queue: VecDeque::new(),
                            after: AfterDialogue::LaunchQueenTrap,
                        };
                    }
                    AfterDialogue::LaunchQueenTrap => {
                        let first = match res.as_str() {
                            "sucked" => script::QUEEN_SUCK_REPLY,
                            "easy" => script::QUEEN_EASY_REPLY,
                            _ => script::QUEEN_GREAT_REPLY,
                        };
                        let mut q: VecDeque<Line> = first.iter().copied().collect();
                        q.extend(script::QUEEN_DROP.iter().copied());
                        if let Some(head) = q.pop_front() {
                            self.dlg.say_expr(head.who, head.expr, head.text, &self.tex);
                            self.phase = Phase::Dialogue { queue: q, after: AfterDialogue::ToBiome(Biome::Coral) };
                        }
                    }
                    AfterDialogue::AskLadyClue => {
                        if res == "yes" {
                            self.prog.add_clue("Market Lady: Genuine berries grow in the NORTH market patch.");
                            self.queue_lines(script::MARKET_LADY_CLUE, AfterDialogue::None);
                        } else {
                            self.dlg.say("Market Lady", "Oh, well go say hello to Scalleon & Gillbert first, sweetie!", &self.tex);
                            self.phase = Phase::Dialogue { queue: VecDeque::new(), after: AfterDialogue::None };
                        }
                    }
                    AfterDialogue::ToBiome(b) => {
                        if b == Biome::Coral {
                            self.prog.elixir = self.prog.elixir.max(1);
                            self.audio.play_sfx(&self.audio.sfx.splash, 0.7);
                        }
                        self.enter_biome(b, true);
                    }
                    AfterDialogue::StartRapBattle => self.start_rap_battle(),
                    AfterDialogue::StartMaze => {
                        self.audio.ensure_music(MusicTrack::Cave);
                        let (order, dir_order) = maze::shuffled_cycle();
                        self.phase = Phase::Maze {
                            depth: 0,
                            q_idx: 0,
                            sel: 0,
                            pool_pos: 0,
                            order,
                            dir_order,
                        };
                    }
                    AfterDialogue::StartAlgaeTemple => {
                        self.phase = Phase::AlgaeTemple { stage: 0, sel: 0, feedback: None };
                    }
                    AfterDialogue::StartStars => {
                        self.audio.ensure_music(MusicTrack::Rainbow);
                        self.phase = Phase::StarCatch {
                            caught: 0,
                            lives: cosma::LIVES as u8,
                            basket_x: 512.0,
                            spawn_t: 0.4,
                            stars: Vec::new(),
                        };
                    }
                    AfterDialogue::StartCrumble => {
                        self.audio.ensure_music(MusicTrack::Battle);
                        self.phase = Phase::Crumble {
                            dist: 0.0,
                            py: 500.0,
                            vy: 0.0,
                            lives: 3,
                            hit_cd: 0.0,
                            spawn_t: 0.5,
                            shards: Vec::new(),
                        };
                    }
                    AfterDialogue::FinishEnding => {
                        self.phase = Phase::Ending;
                    }
                }
            }
            Phase::KennyTree { step } => {
                match step {
                    0 => {
                        self.dlg.ask_expr(
                            "Kenny",
                            "normal",
                            "Catch your breath! What did you just try asking?",
                            &[("Where am I?", "w1"), ("Where are we?", "w2"), ("No, stay back!", "w3")],
                            &self.tex,
                        );
                        self.phase = Phase::KennyTree { step: 1 };
                    }
                    1 => {
                        let prompt = match res.as_str() {
                            "w1" => "Where am I? We're in the *Ocean* *Plains*! Wait... Coral City, I presume?",
                            "w2" => "Where are *we*? Right in the *Ocean* *Plains*! So... Coral City, I presume?",
                            _ => "Whoa, hold your bubbles! We're just in the *Ocean* *Plains*. Coral City, I presume?",
                        };
                        self.dlg.ask_expr(
                            "Kenny",
                            "normal",
                            prompt,
                            &[("Yes?", "y"), ("No.", "n"), ("We're going back to my actual home.", "h")],
                            &self.tex,
                        );
                        self.phase = Phase::KennyTree { step: 2 };
                    }
                    2 => {
                        let opener = match res.as_str() {
                            "y" => Line::expr("Kenny", "happy", "Ooh, a fancy reef resident! Look sharp out here though..."),
                            "h" => Line::expr("Kenny", "worried", "Your... actual home? Above the tide?! Wait, listen up..."),
                            _ => Line::expr("Kenny", "normal", script::KENNY_HOME_Q),
                        };
                        let mut q: VecDeque<Line> = VecDeque::new();
                        if res != "n" {
                            q.push_back(opener);
                        }
                        q.extend(script::KENNY_WORLD_EXPLAIN.iter().copied());
                        if let Some(head) = q.pop_front() {
                            self.dlg.say_expr(head.who, head.expr, head.text, &self.tex);
                            self.phase = Phase::Dialogue { queue: q, after: AfterDialogue::AskKennyWeapons };
                        }
                    }
                    _ => {
                        let opener = match res.as_str() {
                            "r" => Line::expr("Kenny", "normal", "Risk makes things spicy, sure, but we settle disputes purely with *words*!"),
                            "d" => Line::expr("Kenny", "happy", "Bingo!! Riddles, grammar, and razor-sharp wit!"),
                            _ => Line::expr("Kenny", "happy", "Yes! Questions and sharp literacy!"),
                        };
                        let mut q: VecDeque<Line> = VecDeque::new();
                        q.push_back(opener);
                        q.extend(script::KENNY_PRE_DUEL.iter().copied());
                        if let Some(head) = q.pop_front() {
                            self.dlg.say_expr(head.who, head.expr, head.text, &self.tex);
                            self.phase = Phase::Dialogue { queue: q, after: AfterDialogue::StartKennyBattle(false) };
                        }
                    }
                }
            }
            _ => {
                self.phase = phase;
            }
        }
    }

    fn trigger_action(&mut self, act: &str) {
        self.world.interact_cd = 0.25;
        if let Some(num) = act.strip_prefix("riddle_") {
            if let Ok(idx) = num.parse::<usize>() {
                if self.prog.done.contains(act) {
                    self.world.say_hint("This rune tablet is sealed (riddles allow only ONE attempt!).");
                    return;
                }
                self.audio.play_sfx(&self.audio.sfx.question, 0.55);
                self.phase = Phase::Riddle { idx, sel: 0, picked: None };
                return;
            }
        }
        match act {
            "kenny" => {
                if self.prog.done.contains("kenny") {
                    self.queue_lines(&[Line::expr("Kenny", "happy", "Good luck past the Lone Stone Gate, pal!")], AfterDialogue::None);
                } else {
                    self.queue_lines(script::KENNY_INTRO, AfterDialogue::StartKennyTree);
                }
            }
            "gate" => {
                if self.prog.gate_solved {
                    self.world.say_hint("The Lone Stone Gate already stands open!");
                } else if self.prog.kelp_riddles_seen == 0 {
                    self.world.say_hint("Solve at least one Kelp Riddle Monolith first to read the ancient script!");
                } else {
                    self.phase = Phase::Gate {
                        picks: [None; 14],
                        active_slot: Some(0),
                        feedback: None,
                    };
                }
            }
            "guards" => {
                self.prog.done.insert("guards");
                self.queue_lines(script::GUARDS_SCENE, AfterDialogue::None);
            }
            "queen_crab" => {
                if !self.prog.gate_solved {
                    self.world.say_hint("Complete the Stone Throne Story Gate before entering Queen Crab's throne!");
                } else {
                    self.queue_lines(script::QUEEN_CRAB_INTRO, AfterDialogue::AskQueenEval);
                }
            }
            "jack_jill" => {
                if self.prog.all_coral_items() {
                    self.queue_lines(&[Line::expr("Jack", "normal", "Superb!! All 3 reef ingredients gathered! Head east for Tow-Tou's Scrapyard!")], AfterDialogue::None);
                } else {
                    self.queue_lines(&[Line::expr("Jill", "normal", "Bring a Blue Seashroom, Tentacle Anemone & True Seaberry! Check [Tab] Field Notes for clues.")], AfterDialogue::None);
                }
            }
            "alley" => {
                self.queue_lines(script::ALLEY_DUEL, AfterDialogue::None);
            }
            "clue_gill_scal" => {
                self.prog.add_clue("Gillbert & Scalleon: Check WEST terrace & ask Market Lady.");
                self.queue_lines(script::CLUE_GILL_SCAL, AfterDialogue::None);
            }
            "clue_lady" => {
                self.dlg.ask("Market Lady", script::MARKET_LADY_ASK, &[("Yes!", "yes"), ("Not yet", "no")], &self.tex);
                self.phase = Phase::Dialogue { queue: VecDeque::new(), after: AfterDialogue::AskLadyClue };
            }
            "clue_neptune" => {
                self.prog.add_clue("City Hall Statue of Neptune points strictly EAST.");
                self.queue_lines(script::NEPTUNE_STATUE_EXAMINE, AfterDialogue::None);
            }
            "decoy_berry" => {
                self.queue_lines(script::DECOY_BERRY_WARN, AfterDialogue::None);
            }
            "item_0" => {
                self.prog.items[0] = true;
                self.world.burst(1360.0, 520.0, 14, 2);
                self.audio.play_sfx(&self.audio.sfx.coin, 0.6);
                self.queue_lines(script::SEASHROOM_PICK, AfterDialogue::None);
            }
            "item_1" => {
                self.prog.items[1] = true;
                self.world.burst(2120.0, 520.0, 14, 2);
                self.audio.play_sfx(&self.audio.sfx.coin, 0.6);
                self.queue_lines(script::ANEMONE_PICK, AfterDialogue::None);
            }
            "item_2" => {
                self.prog.items[2] = true;
                self.world.burst(2790.0, 520.0, 18, 2);
                self.audio.play_sfx(&self.audio.sfx.coin, 0.65);
                self.queue_lines(script::TRUE_SEABERRY_FOUND, AfterDialogue::None);
            }
            "towtou" => {
                if !self.prog.all_coral_items() {
                    self.world.say_hint("Collect all 3 reef reagents first (use [Tab] Field Notes)!");
                } else {
                    self.towtou_ready = true;
                    self.world.player.0 = 3620.0;
                    self.queue_lines(script::TOWTOU_GREET, AfterDialogue::None);
                }
            }
            "anglers" => {
                if self.anglers_cleared {
                    self.world.say_hint("The Angler Trio already respects your flow. Head east into the Cave Maze!");
                } else {
                    self.queue_lines(script::ANGLERS_AMBUSH, AfterDialogue::StartRapBattle);
                }
            }
            "maze" => {
                if !self.anglers_cleared {
                    self.world.say_hint("Pass the Sea Anglers' cypher at Jevil Ridge before entering the Cave!");
                } else if self.prog.coral_riddles_seen == 0 {
                    self.world.say_hint("Attempt at least one Coral Riddle Tablet before braving the Grave Cave Maze!");
                } else {
                    self.queue_lines(&[Line::new("Tow-Tou", "Careful inside the Grave Cave Maze of Craze! Wrong answers push you back silently without a sound!")], AfterDialogue::StartMaze);
                }
            }
            "thaumiel" | "tenebris" => {
                self.queue_lines(script::SLUMS_ARRIVE, AfterDialogue::None);
            }
            "algae_temple" => {
                if self.prog.algae_solved {
                    self.world.say_hint("The Elixir of Literacy glows safely in your satchel!");
                } else if self.prog.slums_riddles_seen == 0 {
                    self.world.say_hint("Solve at least one Abyss Bridge Riddle before entering the Figurate Algae Temple!");
                } else {
                    self.queue_lines(script::ALGAE_TEMPLE_PRE, AfterDialogue::StartAlgaeTemple);
                }
            }
            "rainbow" => {
                if !self.prog.algae_solved {
                    self.world.say_hint("Fuse the complete Elixir at the Temple of Figurate Algae first!");
                } else {
                    self.queue_lines(script::RAINBOW_GENIE_INTRO, AfterDialogue::StartStars);
                }
            }
            "grandparent" | "mark" | "nadia" => {
                self.queue_lines(script::HOME_ENDING, AfterDialogue::FinishEnding);
            }
            _ => {}
        }
    }

    fn resolve_riddle(&mut self, idx: usize, choice: usize) {
        let r = &script::RIDDLES[idx];
        let right = choice == r.correct;
        let key = match idx {
            0 => "riddle_0",
            1 => "riddle_1",
            2 => "riddle_2",
            3 => "riddle_3",
            4 => "riddle_4",
            5 => "riddle_5",
            6 => "riddle_6",
            _ => "riddle_7",
        };
        self.prog.done.insert(key);
        match r.biome {
            Biome::Kelp => {
                self.prog.kelp_riddles_seen += 1;
                if right { self.prog.kelp_riddles_right += 1; }
            }
            Biome::Coral => {
                self.prog.coral_riddles_seen += 1;
                if right { self.prog.coral_riddles_right += 1; }
            }
            _ => {
                self.prog.slums_riddles_seen += 1;
                if right { self.prog.slums_riddles_right += 1; }
            }
        }
        if right {
            self.audio.play_sfx(&self.audio.sfx.good, 0.65);
            self.fx.accent(true);
        } else {
            self.audio.play_sfx(&self.audio.sfx.bad, 0.6);
            self.fx.accent(false);
        }
    }

    // -----------------------------------------------------------------------
    // Draw Pipeline
    // -----------------------------------------------------------------------

    pub fn draw(&mut self) {
        self.fx.begin();
        let skin = Skin::for_biome(self.biome);

        let mut ui_pick: Option<usize> = None;
        let mut gate_res: Option<(Option<usize>, Option<usize>, bool)> = None;

        match &self.phase {
            Phase::Title => self.draw_title(),
            Phase::Intro { idx, .. } => {
                crate::art::draw_biome_bg(Biome::Home, get_time() as f32, 0.0, &self.fx, &self.tex);
                let plate = &self.tex.intro[(*idx).min(3)];
                let w = screen_width();
                let pw = 720.0;
                let ph = 360.0;
                let px = (w - pw) * 0.5;
                draw_panel(&self.tex, px - 14.0, 52.0, pw + 28.0, ph + 28.0, &skin);
                draw_texture_ex(plate, px, 66.0, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(pw, ph)),
                    ..Default::default()
                });
            }
            Phase::Explore | Phase::Dialogue { .. } | Phase::KennyTree { .. } => {
                self.world.draw(&self.tex, &self.fx, &self.prog);
            }
            Phase::CrabQte { timer, prog } => self.draw_qte(*timer, *prog, &skin),
            Phase::Riddle { .. } | Phase::Gate { .. } | Phase::TorchPlace { .. } | Phase::Ending => {
                self.world.draw(&self.tex, &self.fx, &self.prog);
            }
            Phase::Maze { .. } | Phase::AlgaeTemple { .. } | Phase::GenieQuiz { .. } => {
                crate::art::draw_biome_bg(Biome::Slums, get_time() as f32, 0.0, &self.fx, &self.tex);
            }
            Phase::KennyBattle { .. } => {
                crate::art::draw_biome_bg(Biome::Kelp, get_time() as f32, 0.0, &self.fx, &self.tex);
            }
            Phase::RapBattle { beat_t, .. } => {
                crate::art::draw_biome_bg(Biome::Coral, *beat_t, 0.0, &self.fx, &self.tex);
            }
            Phase::StarCatch { caught, lives, basket_x, stars, .. } => {
                self.draw_stars(*caught, *lives, *basket_x, stars, &skin);
            }
            Phase::Crumble { dist, py, lives, shards, .. } => {
                self.draw_crumble(*dist, *py, *lives, shards, &skin);
            }
        }

        // Fixed crisp UI layer, free of camera shake, holding every interactive widget
        self.fx.begin_ui();

        match &self.phase {
            Phase::Title => {}
            Phase::Explore | Phase::Dialogue { .. } | Phase::KennyTree { .. } => {}
            Phase::KennyBattle { q_idx, round_dodge, timer, hp, boss_hp, fewer, heart, bullets, sel, feedback, .. } => {
                ui_pick = self.draw_kenny_battle(*q_idx, *round_dodge, *timer, *hp, *boss_hp, *fewer, *heart, bullets, *sel, feedback.as_ref().map(|x| (x.0, x.1)), &skin);
            }
            Phase::Riddle { idx, sel, picked } => {
                ui_pick = self.draw_riddle(*idx, *sel, *picked, &skin);
            }
            Phase::Gate { picks, active_slot, feedback } => {
                gate_res = self.draw_gate_ui(picks, *active_slot, feedback.as_ref(), &skin);
            }
            Phase::RapBattle { bar, beat_t, score, notes, sel, feedback } => {
                ui_pick = self.draw_rap_battle(*bar, *beat_t, *score, notes, *sel, feedback.as_ref().map(|x| x.0), &skin);
            }
            Phase::Maze { depth, q_idx, sel, order, dir_order, .. } => {
                ui_pick = self.draw_maze(*depth, order[*q_idx % 8], *sel, *dir_order, &skin);
            }
            Phase::AlgaeTemple { stage, sel, feedback } => {
                ui_pick = self.draw_algae_temple(*stage, *sel, feedback.as_ref().map(|x| (x.0, x.1)), &skin);
            }
            Phase::TorchPlace { lit } => {
                ui_pick = self.draw_torches(*lit, &skin);
            }
            Phase::GenieQuiz { poem_idx, q_idx, sel, feedback } => {
                ui_pick = self.draw_genie(*poem_idx, *q_idx, *sel, feedback.as_ref().map(|x| (x.0, x.1)), &skin);
            }
            _ => {}
        }

        if matches!(self.phase, Phase::Title) {
            self.draw_title_buttons(&skin);
        } else if matches!(self.phase, Phase::Ending) {
            self.draw_ending(&skin);
        }
        if matches!(self.phase, Phase::Explore | Phase::Dialogue { .. } | Phase::KennyTree { .. }) {
            self.draw_hud(&skin);
        }

        if let Some(pick) = ui_pick {
            self.handle_ui_click(pick);
        }
        if let Some((new_active, set_pick, check_now)) = gate_res {
            if let Phase::Gate { picks, active_slot, feedback } = &mut self.phase {
                *active_slot = new_active;
                if let Some((slot_i, pat_i)) = active_slot.zip(set_pick) {
                    picks[slot_i] = Some(pat_i);
                    self.audio.play_sfx(&self.audio.sfx.click, 0.5);
                }
                if check_now {
                    let (right, hint) = gate::evaluate(picks);
                    if right == gate::SLOTS.len() {
                        self.prog.gate_solved = true;
                        self.prog.done.insert("gate");
                        self.audio.play_sfx(&self.audio.sfx.good, 0.75);
                        self.fx.accent(true);
                        self.phase = Phase::Explore;
                        self.world.say_hint("All 14 marks aligned! The Lone Stone Gate swings wide open!");
                    } else {
                        self.audio.play_sfx(&self.audio.sfx.bad, 0.5);
                        *feedback = Some((right, hint.unwrap_or("Check punctuation inside the quotes.").to_string()));
                    }
                }
            }
        }

        if self.dlg.open {
            self.dlg.draw(&self.tex, &skin);
        }

        if self.show_notes {
            if draw_field_notes(&self.tex, &self.prog, self.biome, &skin) {
                self.show_notes = false;
            }
        }
        if self.paused {
            match draw_pause_overlay(&self.tex, self.audio.muted, self.audio.music_vol, &skin) {
                PauseAction::Resume => self.paused = false,
                PauseAction::ToggleMute => { self.audio.toggle_mute(); }
                PauseAction::VolDown => self.audio.set_music_vol(self.audio.music_vol - 0.1),
                PauseAction::VolUp => self.audio.set_music_vol(self.audio.music_vol + 0.1),
                PauseAction::RestartBiome => {
                    self.paused = false;
                    self.enter_biome(self.biome, false);
                }
                PauseAction::QuitTitle => {
                    self.paused = false;
                    self.phase = Phase::Title;
                    self.audio.ensure_music(MusicTrack::Town);
                }
                PauseAction::None => {}
            }
        }

        self.fx.finish(self.fade);
    }

    fn draw_title(&self) {
        let w = screen_width();
        let t = get_time() as f32;
        crate::art::draw_biome_bg(Biome::Coral, t * 0.8, t * 22.0, &self.fx, &self.tex);

        // Passing Ansimuz reef fish silhouettes in the lagoon behind the title
        let f_frame = ((t * 5.0) as usize) % 4;
        for i in 0..3 {
            let fx_pos = (w + 100.0) - ((t * 42.0 + i as f32 * 390.0).rem_euclid(w + 200.0));
            let fy = 228.0 + i as f32 * 36.0 + (t * 1.4 + i as f32).sin() * 10.0;
            let fspr = if i % 2 == 0 { &self.tex.aqua.anim_fish } else { &self.tex.aqua.anim_fish_big };
            let fw = fspr.width() / 4.0;
            draw_texture_ex(fspr, fx_pos, fy, c(210, 245, 255, 210), DrawTextureParams {
                source: Some(Rect::new(f_frame as f32 * fw, 0.0, fw, fspr.height())),
                dest_size: Some(vec2(fw * 2.0, fspr.height() * 2.0)),
                ..Default::default()
            });
        }

        // Rich sandy seabed floor so lower 160px of title screen integrates seamlessly
        let tp = &self.tex.tiles;
        let step = 48.0;
        for c_idx in 0..((w / step).ceil() as i32 + 1) {
            let x = c_idx as f32 * step;
            for r in 0..4 {
                let tile = if r <= 1 { &tp.coral_sand } else { &tp.dirt };
                crate::art::draw_tile(tile, x, 556.0 + r as f32 * step, step);
            }
        }
        let ledge_w = 144.0;
        for lc in 0..((w / ledge_w).ceil() as i32 + 1) {
            draw_texture_ex(&self.tex.aqua.seabed_ledge, lc as f32 * ledge_w, 542.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(ledge_w + 1.0, 22.0)),
                ..Default::default()
            });
        }
        draw_texture_ex(&self.tex.aqua.coral_red, 92.0, 488.0, WHITE, DrawTextureParams { dest_size: Some(vec2(68.0, 76.0)), ..Default::default() });
        draw_texture_ex(&self.tex.aqua.sea_bush, 864.0, 492.0, WHITE, DrawTextureParams { dest_size: Some(vec2(68.0, 72.0)), ..Default::default() });

        let title_skin = Skin::for_biome(Biome::Coral);
        draw_nine_slice(&self.tex, (w - 720.0) * 0.5, 44.0, 720.0, 150.0, title_skin.bg, title_skin.accent);
        let t1 = "GIFT OF EDUCATION";
        let t2 = "A SUNKEN REALM LITERACY RPG";
        draw_tag_shadow(t1, (w - tag_width(t1, 28)) * 0.5, 108.0, 28, title_skin.accent);
        draw_tag_shadow(t2, (w - tag_width(t2, 13)) * 0.5, 154.0, 13, title_skin.chip_ink);

        // Glowing floating Elixir Tome sprite + sparkles
        let tome_y = 266.0 + (t * 2.2).sin() * 8.0;
        self.fx.glow_rect(w * 0.5, tome_y + 66.0, 220.0, 220.0, c(104, 240, 255, 58));
        draw_texture_ex(&self.tex.props.tome, w * 0.5 - 54.0, tome_y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(108.0, 132.0)),
            ..Default::default()
        });

        // Grounded animated River (idle blink) & claw-snapping Kenny flanking the Tome on sand
        let feet_y = 562.0;
        if let Some(sh) = self.tex.shadow_of("river") {
            let sw = sh.width() * 3.0;
            draw_texture_ex(sh, w * 0.5 - 230.0 - sw * 0.5, feet_y - sh.height() * 1.5, WHITE, DrawTextureParams {
                dest_size: Some(vec2(sw, sh.height() * 3.0)),
                ..Default::default()
            });
        }
        let r_spr = if (t % 3.6) > 3.42 { self.tex.idle_of("river").unwrap_or(&self.tex.river) } else { &self.tex.river };
        let rw = r_spr.width() * 3.0;
        let rh = r_spr.height() * 3.0;
        draw_texture_ex(r_spr, w * 0.5 - 230.0 - rw * 0.5, feet_y - rh, WHITE, DrawTextureParams {
            dest_size: Some(vec2(rw, rh)),
            ..Default::default()
        });

        if let Some(sh) = self.tex.shadow_of("kenny") {
            let sw = sh.width() * 3.0;
            draw_texture_ex(sh, w * 0.5 + 230.0 - sw * 0.5, feet_y - sh.height() * 1.5, WHITE, DrawTextureParams {
                dest_size: Some(vec2(sw, sh.height() * 3.0)),
                ..Default::default()
            });
        }
        let k_spr = match ((t * 3.4) as usize) % 5 {
            0 | 1 => &self.tex.kenny,
            2 => &self.tex.walks["kenny"][0],
            3 => &self.tex.walks["kenny"][1],
            _ => self.tex.idle_of("kenny").unwrap_or(&self.tex.kenny),
        };
        let kw = k_spr.width() * 3.0;
        let kh = k_spr.height() * 3.0;
        draw_texture_ex(k_spr, w * 0.5 + 230.0 - kw * 0.5, feet_y - kh, WHITE, DrawTextureParams {
            dest_size: Some(vec2(kw, kh)),
            flip_x: true,
            ..Default::default()
        });
    }

    fn draw_title_buttons(&mut self, skin: &Skin) {
        let w = screen_width();
        let title_skin = Skin::for_biome(Biome::Coral);
        let bw = 280.0;
        let bx = (w - bw) * 0.5;
        if draw_btn(&self.tex, bx, 466.0, bw, 52.0, "NEW ADVENTURE", &title_skin, true) {
            self.audio.play_sfx(&self.audio.sfx.select, 0.7);
            self.start_new_game();
            return;
        }
        if self.has_save {
            if draw_btn(&self.tex, bx, 530.0, bw, 52.0, "CONTINUE SAVE", &title_skin, false) {
                self.audio.play_sfx(&self.audio.sfx.select, 0.7);
                self.load_checkpoint();
                return;
            }
        }
        let s_ico = if self.audio.muted { &self.tex.ui.sound_off } else { &self.tex.ui.sound_on };
        let m_lbl = if self.audio.muted { "SOUND: OFF [M]" } else { "SOUND: ON [M]" };
        if draw_btn_icon(&self.tex, bx, if self.has_save { 594.0 } else { 536.0 }, bw, 46.0, s_ico, m_lbl, &title_skin, false) {
            self.audio.toggle_mute();
        }
        let _ = skin;
        let hint = "Arrow Keys / WASD / Click to Move   •   Enter / E / Click to Interact";
        draw_dlg_shadow(hint, (w - dlg_width(hint, 22)) * 0.5, 686.0, 22, title_skin.muted);
    }

    fn draw_hud(&mut self, skin: &Skin) {
        let w = screen_width();
        let title = script::biome_title(self.biome);
        draw_nine_slice(&self.tex, 12.0, 12.0, 400.0, 44.0, skin.bg, skin.border);
        let bsize = if tag_width(title, 10) <= 372.0 { 10 } else { 9 };
        draw_tag_shadow(title, 26.0, 39.0, bsize, skin.accent);

        // Elixir vial & Riddle scroll badge cleanly spaced with 8px gutters
        let vial = match self.prog.elixir {
            0 => &self.tex.ui.vial_empty,
            1 => &self.tex.ui.vial_red,
            2 => &self.tex.ui.vial_green,
            _ => &self.tex.ui.vial_blue,
        };
        draw_nine_slice(&self.tex, 420.0, 12.0, 248.0, 44.0, skin.bg, skin.border);
        draw_texture_ex(vial, 428.0, 22.0, WHITE, DrawTextureParams { dest_size: Some(vec2(24.0, 24.0)), ..Default::default() });
        draw_dlg_shadow(&format!("Elixir {}/3", self.prog.elixir), 456.0, 40.0, 21, skin.ink);
        draw_texture_ex(&self.tex.ui.scroll, 548.0, 23.0, WHITE, DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() });
        draw_dlg_shadow(&format!("Riddles {}", self.prog.total_riddles_right()), 574.0, 40.0, 21, skin.ink);

        let s_ico = if self.audio.muted { &self.tex.ui.sound_off } else { &self.tex.ui.sound_on };
        if crate::game::ui::draw_icon_only_btn(&self.tex, w - 348.0, 12.0, 44.0, 44.0, s_ico, skin, self.audio.muted) {
            self.audio.toggle_mute();
        }
        if draw_btn_icon(&self.tex, w - 296.0, 12.0, 152.0, 44.0, &self.tex.ui.book, "NOTES [Tab]", skin, self.show_notes) {
            self.show_notes = !self.show_notes;
            self.audio.play_sfx(&self.audio.sfx.page, 0.55);
        }
        if draw_btn(&self.tex, w - 136.0, 12.0, 124.0, 44.0, "PAUSE [Esc]", skin, false) {
            self.paused = true;
        }
    }

    fn draw_qte(&self, timer: f32, prog: f32, skin: &Skin) {
        let w = screen_width();
        crate::art::draw_biome_bg(Biome::Kelp, get_time() as f32, get_time() as f32 * 180.0, &self.fx, &self.tex);
        self.world.draw_ground(&self.tex, get_time() as f32 * 220.0, 556.0, get_time() as f32);

        let chase_gap = (1.0 - prog) * 320.0;
        let rx = 680.0;
        let kx = rx - 140.0 - chase_gap;
        let swim = &self.tex.river_swim[((get_time() * 10.0) as usize) % 4];
        draw_texture_ex(swim, rx, 456.0 + (get_time() as f32 * 8.0).sin() * 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(114.0, 96.0)), ..Default::default() });
        let ken = &self.tex.walks["kenny"][((get_time() * 10.0) as usize) % 2];
        draw_texture_ex(ken, kx, 448.0, WHITE, DrawTextureParams { dest_size: Some(vec2(114.0, 108.0)), ..Default::default() });

        draw_panel(&self.tex, (w - 640.0) * 0.5, 58.0, 640.0, 160.0, skin);
        let head = "SWIM FAST!! OUTRUN THE KELP CRAB!";
        draw_tag_shadow(head, (w - tag_width(head, 15)) * 0.5, 102.0, 15, skin.accent);
        let sub = &format!("MASH [SPACE] / [Z] / CLICK!    Time: {:.1}s", timer.max(0.0));
        draw_dlg_shadow(sub, (w - dlg_width(sub, 26)) * 0.5, 140.0, 26, skin.ink);

        let bw = 540.0;
        let bx = (w - bw) * 0.5;
        draw_nine_slice(&self.tex, bx, 158.0, bw, 36.0, skin.chip, skin.border);
        draw_rectangle(bx + 8.0, 166.0, (bw - 16.0) * prog.clamp(0.0, 1.0), 20.0, skin.accent);
    }

    fn draw_kenny_battle(
        &self,
        q_idx: usize,
        round_dodge: bool,
        timer: f32,
        hp: u8,
        boss_hp: u8,
        fewer: bool,
        heart: (f32, f32),
        bullets: &[(f32, f32, f32, f32)],
        sel: usize,
        feedback: Option<(bool, &'static str)>,
        skin: &Skin,
    ) -> Option<usize> {
        let w = screen_width();
        let mut hit = None;

        // Top arena header & boss portrait with impact shake on direct hit
        let t = get_time() as f32;
        let kenny_hit = feedback.map(|(good, _)| good).unwrap_or(false);
        draw_panel(&self.tex, (w - 760.0) * 0.5, 24.0, 760.0, 200.0, skin);
        let kenny_p = self.tex.portrait_expr("Kenny", if boss_hp <= 1 || kenny_hit { "worried" } else { "normal" });
        if let Some(p) = kenny_p {
            let jx = if kenny_hit { (t * 52.0).sin() * 6.0 } else { 0.0 };
            let jy = if kenny_hit { (t * 38.0).cos() * 3.0 } else { (t * 3.4).sin() * 2.5 };
            let tint = if kenny_hit && (t * 28.0).sin() > 0.0 { c(255, 165, 165, 255) } else { WHITE };
            draw_nine_slice(&self.tex, 154.0 + jx, 48.0 + jy, 144.0, 144.0, skin.chip, skin.accent);
            draw_texture_ex(p, 162.0 + jx, 56.0 + jy, tint, DrawTextureParams { dest_size: Some(vec2(128.0, 128.0)), ..Default::default() });
        }

        draw_tag_shadow("KELP DUEL: KENNY THE CRAB", 322.0, 68.0, 14, skin.accent);
        for i in 0..4 {
            let htex = if (i as u8) < hp { &self.tex.ui.heart_full } else { &self.tex.ui.heart_empty };
            draw_texture_ex(htex, 322.0 + i as f32 * 34.0, 86.0, WHITE, DrawTextureParams { dest_size: Some(vec2(28.0, 28.0)), ..Default::default() });
        }
        let max_b = kenny_battle::questions_for(fewer).len();
        draw_dlg_shadow(&format!("Kenny Resolve: {} / {}", boss_hp, max_b), 490.0, 108.0, 24, skin.ink);

        if let Some((good, msg)) = feedback {
            draw_panel(&self.tex, (w - 640.0) * 0.5, 290.0, 640.0, 150.0, skin);
            draw_tag_shadow(msg, (w - tag_width(msg, 15)) * 0.5, 372.0, 15, if good { skin.accent } else { c(255, 110, 130, 255) });
            return None;
        }

        if round_dodge {
            draw_panel(&self.tex, 296.0, 324.0, 432.0, 224.0, skin);
            draw_tag_shadow(&format!("DODGE KENNY'S BUBBLE SHELLS!  {:.1}s", timer), 312.0, 304.0, 12, skin.accent);
            for (i, &(bx, by, _, _)) in bullets.iter().enumerate() {
                draw_texture_ex(&self.tex.ui.gem, bx - 14.0, by - 14.0, c(120, 246, 220, 255), DrawTextureParams {
                    dest_size: Some(vec2(28.0, 28.0)),
                    rotation: t * 4.5 + i as f32,
                    pivot: Some(vec2(bx, by)),
                    ..Default::default()
                });
            }
            let h_scale = 28.0 * (1.0 + (t * 8.0).sin() * 0.07);
            draw_texture_ex(&self.tex.ui.heart_full, heart.0 - h_scale * 0.5, heart.1 - h_scale * 0.5, WHITE, DrawTextureParams { dest_size: Some(vec2(h_scale, h_scale)), ..Default::default() });
        } else {
            let qs = kenny_battle::questions_for(fewer);
            let q = &qs[q_idx.min(qs.len() - 1)];
            draw_panel(&self.tex, 132.0, 244.0, 760.0, 410.0, skin);
            draw_wrapped_rich(q.prompt, 160.0, 294.0, 700.0, 26, 28.0, skin.ink, skin.accent);
            for (i, ch) in q.choices.iter().enumerate() {
                if draw_option_row_skinned(&self.tex, 160.0, 420.0 + i as f32 * 64.0, 700.0, 54.0, ch, sel == i, skin) {
                    hit = Some(i);
                }
            }
            draw_tag_shadow("Keys [1-3] / [Enter] or Click an option", 160.0, 636.0, 11, skin.muted);
        }
        hit
    }

    fn draw_riddle(&self, idx: usize, sel: usize, picked: Option<usize>, skin: &Skin) -> Option<usize> {
        let w = screen_width();
        let r = &script::RIDDLES[idx];
        draw_rectangle(0.0, 0.0, w, screen_height(), Color::new(0.01, 0.01, 0.03, 0.65));

        let pw = 800.0;
        let ph = 560.0;
        let px = (w - pw) * 0.5;
        let py = 76.0;
        draw_panel(&self.tex, px, py, pw, ph, skin);
        quiz_header_skinned(&self.tex, r.title, px + 28.0, py + 24.0, pw - 56.0, skin);
        draw_tag_shadow("ONE TRY ONLY", px + pw - 190.0, py + 40.0, 11, c(255, 130, 150, 255));

        let p_lines = wrapped_line_count(r.prompt, pw - 64.0, 26);
        draw_wrapped_rich(r.prompt, px + 32.0, py + 94.0, pw - 64.0, 26, 28.0, skin.ink, skin.accent);

        let opt_y0 = (py + 106.0 + p_lines as f32 * 28.0).max(py + 210.0);
        let mut hit = None;
        for (i, ch) in r.choices.iter().enumerate() {
            let mut label = ch.to_string();
            if let Some(p) = picked {
                if i == r.correct {
                    label = format!("[CORRECT] {}", ch);
                } else if i == p {
                    label = format!("[YOUR PICK] {}", ch);
                }
            }
            if draw_option_row_skinned(&self.tex, px + 32.0, opt_y0 + i as f32 * 62.0, pw - 64.0, 52.0, &label, picked.unwrap_or(sel) == i, skin) && picked.is_none() {
                hit = Some(i);
            }
        }

        if let Some(p) = picked {
            let ok = p == r.correct;
            let foot = if ok { "Nice work! " } else { "LOCKED. Note the explanation: " };
            draw_wrapped_rich(&format!("{}{}", foot, r.explain), px + 32.0, opt_y0 + 266.0, pw - 64.0, 22, 24.0, skin.accent, skin.ink);
            draw_tag_shadow("Press [Enter] or Click to return", px + pw - 330.0, py + ph - 26.0, 10, skin.muted);
        } else {
            draw_tag_shadow("Keys [1-4] / [Enter] or Click an option", px + 32.0, py + ph - 26.0, 10, skin.muted);
        }
        hit
    }

    fn draw_gate_ui(
        &self,
        picks: &[Option<usize>; 14],
        active_slot: Option<usize>,
        feedback: Option<&(usize, String)>,
        skin: &Skin,
    ) -> Option<(Option<usize>, Option<usize>, bool)> {
        let w = screen_width();
        let pw = 920.0;
        let ph = 636.0;
        let px = (w - pw) * 0.5;
        let py = 42.0;
        draw_panel(&self.tex, px, py, pw, ph, skin);
        quiz_header_skinned(&self.tex, "THE LONE STONE THRONE STORY (14 PUNCTUATION SLOTS)", px + 26.0, py + 22.0, pw - 52.0, skin);

        // Flow verbatim text + clickable inline slot chips
        let x0 = px + 28.0;
        let max_x = px + pw - 28.0;
        let mut x = x0;
        let mut y = py + 88.0;
        let line_h = 28.0;

        let mut next_active = active_slot;
        let (mx, my) = mouse_position();
        let mclick = is_mouse_button_pressed(MouseButton::Left);

        let draw_frag = |frag: &str, x: &mut f32, y: &mut f32| {
            for word in frag.split_inclusive(' ') {
                let ww = dlg_width(word, 22);
                if *x > x0 && *x + ww > max_x {
                    *x = x0;
                    *y += line_h;
                }
                draw_dlg_shadow(word, *x, *y, 22, skin.ink);
                *x += ww;
            }
        };

        for (i, slot) in gate::SLOTS.iter().enumerate() {
            draw_frag(slot.before, &mut x, &mut y);
            if !slot.before.is_empty() && !slot.before.ends_with(' ') && !slot.before.ends_with('\n') {
                x += 3.0;
            }
            let sym = picks[i].and_then(|pi| gate::PALETTE.get(pi)).map(|p| p.0).unwrap_or("?");
            let bw = (dlg_width(sym, 22) + 22.0).max(36.0);
            if x + bw > max_x {
                x = x0;
                y += line_h;
            }
            let by = y - 20.0;
            let sel = next_active == Some(i);
            let hov = mx >= x && mx <= x + bw && my >= by && my <= by + 24.0;
            let fill = if sel { c(46, 130, 210, 255) } else if picks[i].is_some() { c(28, 86, 138, 255) } else { c(24, 58, 108, 255) };
            let bord = if sel || hov { skin.accent } else { c(110, 200, 255, 255) };
            draw_rectangle(x, by, bw, 24.0, fill);
            draw_rectangle_lines(x, by, bw, 24.0, if sel { 2.5 } else { 2.0 }, bord);
            draw_dlg_shadow(sym, x + (bw - dlg_width(sym, 22)) * 0.5, y - 1.0, 22, WHITE);
            if hov && mclick {
                next_active = Some(i);
            }
            x += bw + 4.0;
        }
        draw_frag(gate::TRAILING_TEXT, &mut x, &mut y);

        // Palette bar
        let pal_y = py + ph - 144.0;
        draw_tag_shadow(
            &format!("SELECT PUNCTUATION FOR SLOT #{}:", next_active.map(|i| i + 1).unwrap_or(1)),
            px + 28.0,
            pal_y,
            11,
            skin.accent,
        );
        let mut chosen_pat = None;
        for (pi, (sym, _lbl)) in gate::PALETTE.iter().enumerate() {
            let bx = px + 28.0 + pi as f32 * 70.0;
            let by = pal_y + 12.0;
            if draw_btn(&self.tex, bx, by, 62.0, 42.0, sym, skin, false) {
                chosen_pat = Some(pi);
            }
        }

        if let Some((right, msg)) = feedback {
            draw_dlg_shadow(
                &format!("Score: {}/14: {}", right, msg),
                px + 28.0,
                py + ph - 26.0,
                22,
                skin.accent,
            );
        } else {
            draw_dlg_shadow("Click any glowing box, pick a mark above, then press CHECK GATE.", px + 28.0, py + ph - 26.0, 22, skin.muted);
        }

        let check_now = draw_btn(&self.tex, px + pw - 210.0, py + ph - 68.0, 182.0, 48.0, "CHECK GATE", skin, true);
        Some((next_active, chosen_pat, check_now))
    }

    fn draw_rap_battle(
        &self,
        bar: usize,
        beat_t: f32,
        score: u32,
        notes: &[(f32, usize, bool)],
        sel: usize,
        feedback: Option<&'static str>,
        skin: &Skin,
    ) -> Option<usize> {
        let w = screen_width();
        let mut hit = None;
        let pulse = ((beat_t * 4.0) as usize) % 2;

        // Animated stage characters grooving on the beat
        let tow = if pulse == 0 { &self.tex.towtou } else { &self.tex.walks["towtou"][0] };
        let riv = &self.tex.river_walks[((beat_t * 5.0) as usize) % 4];
        let aa = &self.tex.walks["angler_a"][pulse];
        let ab = &self.tex.walks["angler_b"][(pulse + 1) % 2];
        let ac = &self.tex.walks["angler_c"][pulse];
        let bob = (beat_t * 6.0).sin() * 4.0;

        draw_texture_ex(tow, 84.0, 210.0, WHITE, DrawTextureParams { dest_size: Some(vec2(144.0, 120.0)), ..Default::default() });
        draw_texture_ex(riv, 240.0, 220.0 + bob * 0.5, WHITE, DrawTextureParams { dest_size: Some(vec2(96.0, 114.0)), ..Default::default() });
        draw_texture_ex(aa, 690.0, 220.0 - bob, WHITE, DrawTextureParams { dest_size: Some(vec2(96.0, 114.0)), flip_x: true, ..Default::default() });
        draw_texture_ex(ab, 770.0, 224.0 + bob, WHITE, DrawTextureParams { dest_size: Some(vec2(96.0, 114.0)), flip_x: true, ..Default::default() });
        draw_texture_ex(ac, 850.0, 220.0 - bob, WHITE, DrawTextureParams { dest_size: Some(vec2(96.0, 114.0)), flip_x: true, ..Default::default() });

        draw_panel(&self.tex, 60.0, 24.0, w - 120.0, 86.0, skin);
        draw_texture_ex(&self.tex.ui.note, 82.0, 42.0, WHITE, DrawTextureParams { dest_size: Some(vec2(26.0, 26.0)), ..Default::default() });
        draw_tag_shadow(&format!("SEA ANGLERS CYPHER: BAR {}/{}", bar + 1, cosma::RAP_BARS.len()), 118.0, 62.0, 15, skin.accent);
        draw_tag_shadow(&format!("FLOW SCORE: {}", score), w - 280.0, 62.0, 14, skin.ink);

        // Beat lanes (left)
        draw_panel(&self.tex, 60.0, 350.0, 320.0, 320.0, skin);
        let keys = ["A", "S", "D", "F"];
        for li in 0..4 {
            let lx = 84.0 + li as f32 * 68.0;
            draw_nine_slice(&self.tex, lx, 604.0, 56.0, 44.0, skin.chip, skin.accent);
            draw_tag_shadow(keys[li], lx + 20.0, 632.0, 14, skin.accent);
        }
        for (nt, nl, used) in notes {
            if *used { continue; }
            let dy = (*nt - beat_t) * 180.0;
            if dy > -30.0 && dy < 260.0 {
                let lx = 84.0 + *nl as f32 * 68.0;
                draw_texture_ex(&self.tex.ui.note, lx + 12.0, 608.0 - dy, WHITE, DrawTextureParams { dest_size: Some(vec2(32.0, 32.0)), ..Default::default() });
            }
        }

        // Rhyme selector (right)
        let cur = &cosma::RAP_BARS[bar];
        let right_idx = (bar * 2) % 3;
        draw_panel(&self.tex, 400.0, 350.0, 564.0, 320.0, skin);
        draw_wrapped_rich(cur.prompt, 428.0, 396.0, 510.0, 24, 26.0, skin.ink, skin.accent);
        for i in 0..3 {
            let word = if i == right_idx {
                cur.rhyming_word
            } else {
                cur.decoys[if i > right_idx { i - 1 } else { i }]
            };
            if draw_option_row_skinned(&self.tex, 428.0, 476.0 + i as f32 * 52.0, 510.0, 44.0, &format!("{}) {}", i + 1, word), sel == i, skin) {
                hit = Some(i);
            }
        }
        if let Some(msg) = feedback {
            draw_tag_shadow(msg, 428.0, 650.0, 12, skin.accent);
        }
        hit
    }

    fn draw_maze(&self, depth: u8, q_idx: usize, sel: usize, dir_order: [maze::Dir; 4], skin: &Skin) -> Option<usize> {
        let w = screen_width();
        let q = &maze::QUESTIONS[q_idx];
        let mut hit = None;
        draw_panel(&self.tex, (w - 820.0) * 0.5, 48.0, 820.0, 610.0, skin);
        quiz_header_skinned(&self.tex, "THE GRAVE CAVE MAZE OF CRAZE", 132.0, 74.0, 760.0, skin);

        draw_tag_shadow(
            &format!("TUNNEL DEPTH {}/{}   CHOOSE PASSAGE BY RIDDLE & NPC CLUES", depth, maze::MAZE_TARGET_DEPTH),
            132.0,
            146.0,
            13,
            skin.accent,
        );
        draw_wrapped_rich(q.prompt, 132.0, 198.0, 760.0, 26, 28.0, skin.ink, skin.accent);
        const TAGS: [&str; 4] = ["a)", "b)", "c)", "d)"];
        for (i, d) in dir_order.iter().enumerate() {
            let opt = format!("{} {} Tunnel", TAGS[i], d.label());
            if draw_option_row_skinned(&self.tex, 132.0, 316.0 + i as f32 * 70.0, 760.0, 58.0, &opt, sel == i, skin) {
                hit = Some(i);
            }
        }
        draw_tag_shadow("Keys [1-4], [Enter], or Click carefully! Wrong turns push back silently.", 132.0, 622.0, 11, skin.muted);
        hit
    }

    fn draw_algae_temple(&self, stage: usize, sel: usize, feedback: Option<(bool, &'static str)>, skin: &Skin) -> Option<usize> {
        let w = screen_width();
        let tr = &cosma::ALGAE_TRIALS[stage];
        let mut hit = None;
        draw_panel(&self.tex, 60.0, 52.0, w - 120.0, 606.0, skin);
        quiz_header_skinned(&self.tex, &format!("FIGURATE ALGAE TEMPLE: {}", tr.chamber), 90.0, 78.0, w - 180.0, skin);

        draw_nine_slice(&self.tex, 90.0, 142.0, 250.0, 286.0, skin.chip, skin.border);
        let rows = stage + 2;
        let pulse = (get_time() as f32 * 3.0).sin() * 2.0;
        for r in 0..rows {
            for c_i in 0..=r {
                let gx = 215.0 + (c_i as f32 - r as f32 * 0.5) * 34.0;
                let gy = 185.0 + r as f32 * 38.0 + pulse * 0.5;
                let ico = if r == 0 { &self.tex.ui.pearl } else { &self.tex.ui.gem };
                draw_texture_ex(ico, gx - 13.0, gy - 13.0, WHITE, DrawTextureParams { dest_size: Some(vec2(26.0, 26.0)), ..Default::default() });
            }
        }
        draw_wrapped_col(tr.sequence_hint, 108.0, 356.0, 214.0, 20, 22.0, skin.muted);

        draw_wrapped_rich(tr.riddle_text, 368.0, 174.0, 560.0, 25, 28.0, skin.ink, skin.accent);
        for (i, ch) in tr.choices.iter().enumerate() {
            if draw_option_row_skinned(&self.tex, 368.0, 300.0 + i as f32 * 62.0, 560.0, 52.0, ch, sel == i, skin) && feedback.is_none() {
                hit = Some(i);
            }
        }
        if let Some((good, msg)) = feedback {
            draw_wrapped_rich(msg, 90.0, 586.0, w - 180.0, 24, 26.0, if good { skin.accent } else { c(255, 130, 140, 255) }, skin.ink);
        }
        hit
    }

    fn draw_stars(&self, caught: u32, lives: u8, basket_x: f32, stars: &[(f32, f32, f32, bool)], skin: &Skin) {
        let w = screen_width();
        let t = get_time() as f32;
        crate::art::draw_biome_bg(Biome::Slums, t, 0.0, &self.fx, &self.tex);

        for (sx, sy, _, dying) in stars {
            let spr = if *dying { &self.tex.ui.dying_star } else { &self.tex.ui.star };
            draw_texture_ex(spr, sx - 16.0, sy - 16.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(32.0, 32.0)),
                rotation: t * if *dying { -2.2 } else { 1.6 },
                pivot: Some(vec2(*sx, *sy)),
                ..Default::default()
            });
        }

        draw_nine_slice(&self.tex, basket_x - 56.0, 610.0, 112.0, 36.0, skin.sel, skin.accent);
        let r_spr = &self.tex.river_walks[((t * 8.0) as usize) % 4];
        draw_texture_ex(r_spr, basket_x - 24.0, 554.0, WHITE, DrawTextureParams { dest_size: Some(vec2(48.0, 57.0)), ..Default::default() });

        draw_panel(&self.tex, 60.0, 22.0, w - 120.0, 74.0, skin);
        draw_tag_shadow(&format!("RAINBOW ROAD: CATCH STARS  {}/{}", caught, cosma::STAR_TARGET), 84.0, 62.0, 14, skin.accent);
        for i in 0..3 {
            let ht = if (i as u8) < lives { &self.tex.ui.heart_full } else { &self.tex.ui.heart_empty };
            draw_texture_ex(ht, w - 200.0 + i as f32 * 36.0, 42.0, WHITE, DrawTextureParams { dest_size: Some(vec2(30.0, 30.0)), ..Default::default() });
        }
        draw_dlg_shadow("Arrows / Mouse to Catch Gold Stars (Dodge Dying Stars!)", 84.0, 86.0, 21, skin.muted);
    }

    fn draw_torches(&self, lit: [bool; 3], skin: &Skin) -> Option<usize> {
        let w = screen_width();
        let mut hit = None;
        let flick = 1 + ((get_time() * 8.0) as usize % 2);
        draw_panel(&self.tex, (w - 640.0) * 0.5, 160.0, 640.0, 256.0, skin);
        let head = "CARVE & PLACE THE 3 POEM SLABS";
        draw_tag_shadow(head, (w - tag_width(head, 15)) * 0.5, 210.0, 15, skin.accent);
        for i in 0..3 {
            let tx = (w - 380.0) * 0.5 + i as f32 * 140.0;
            let spr = if lit[i] { &self.tex.props.torch[flick] } else { &self.tex.props.torch[0] };
            draw_texture_ex(spr, tx + 16.0, 232.0, WHITE, DrawTextureParams { dest_size: Some(vec2(64.0, 108.0)), ..Default::default() });
            if draw_btn(&self.tex, tx, 344.0, 96.0, 38.0, if lit[i] { "LIT" } else { "LIGHT" }, skin, lit[i]) {
                hit = Some(i);
            }
        }
        draw_dlg_shadow("Press [Space] / [1][2][3] or Click each Torch!", 270.0, 404.0, 22, skin.ink);
        hit
    }

    fn draw_genie(&self, poem_idx: usize, q_idx: usize, sel: usize, feedback: Option<(bool, &'static str)>, skin: &Skin) -> Option<usize> {
        let w = screen_width();
        let t = get_time() as f32;
        let poem = &cosma::POEMS[poem_idx];
        let q = &poem.questions[q_idx];
        let mut hit = None;

        draw_panel(&self.tex, 36.0, 48.0, 452.0, 620.0, skin);
        draw_tag_shadow(poem.title, 58.0, 88.0, 11, skin.accent);
        draw_dlg_shadow(poem.author, 58.0, 114.0, 20, skin.muted);
        let mut py = 148.0;
        for line in poem.lines {
            if !line.is_empty() {
                draw_dlg_shadow(line, 58.0, py, 20, skin.ink);
            }
            py += 22.0;
        }

        draw_panel(&self.tex, 504.0, 48.0, w - 540.0, 620.0, skin);
        let g_spr = match ((t * 7.0) as usize) % 3 {
            0 => &self.tex.genie,
            1 => &self.tex.walks["genie"][0],
            _ => &self.tex.walks["genie"][1],
        };
        draw_texture_ex(g_spr, 528.0, 74.0 + (t * 3.0).sin() * 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(96.0, 108.0)), ..Default::default() });
        draw_tag_shadow("MAGIC FIRE GENIE", 640.0, 110.0, 15, skin.accent);
        draw_wrapped_rich(q.prompt, 528.0, 210.0, w - 588.0, 24, 26.0, skin.ink, skin.accent);
        for (i, ch) in q.choices.iter().enumerate() {
            if draw_option_row_skinned(&self.tex, 528.0, 290.0 + i as f32 * 74.0, w - 588.0, 62.0, ch, sel == i, skin) && feedback.is_none() {
                hit = Some(i);
            }
        }
        if let Some((good, msg)) = feedback {
            draw_wrapped_rich(msg, 528.0, 606.0, w - 588.0, 22, 24.0, if good { skin.accent } else { c(255, 130, 140, 255) }, skin.ink);
        }
        hit
    }

    fn handle_ui_click(&mut self, pick: usize) {
        let phase = std::mem::replace(&mut self.phase, Phase::Explore);
        match phase {
            Phase::KennyBattle { q_idx, round_dodge, timer, mut hp, mut boss_hp, fewer, mut flawless, heart, bullets, hit_cd, feedback, .. } => {
                if !round_dodge && feedback.is_none() {
                    let qs = kenny_battle::questions_for(fewer);
                    let cur_q = &qs[q_idx.min(qs.len() - 1)];
                    let new_fb = if pick == cur_q.correct {
                        boss_hp = boss_hp.saturating_sub(1);
                        self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                        self.fx.accent(true);
                        Some((true, "Clean strike!!", 0.9))
                    } else {
                        hp = hp.saturating_sub(1);
                        flawless = false;
                        self.audio.play_sfx(&self.audio.sfx.bad, 0.55);
                        self.fx.accent(false);
                        Some((false, "Missed that! Guard up!", 0.9))
                    };
                    self.phase = Phase::KennyBattle { q_idx, round_dodge, timer, hp, boss_hp, fewer, flawless, heart, bullets, hit_cd, sel: pick, feedback: new_fb };
                    return;
                }
                self.phase = Phase::KennyBattle { q_idx, round_dodge, timer, hp, boss_hp, fewer, flawless, heart, bullets, hit_cd, sel: pick, feedback };
            }
            Phase::Riddle { idx, .. } => {
                self.resolve_riddle(idx, pick);
                self.phase = Phase::Riddle { idx, sel: pick, picked: Some(pick) };
            }
            Phase::RapBattle { mut bar, beat_t, mut score, notes, .. } => {
                let right_idx = (bar * 2) % 3;
                if pick == right_idx {
                    score += 250;
                    self.audio.play_sfx(&self.audio.sfx.good, 0.55);
                    bar += 1;
                    if bar >= cosma::RAP_BARS.len() {
                        self.anglers_cleared = true;
                        self.prog.done.insert("anglers");
                        self.audio.ensure_music(MusicTrack::Coral);
                        self.queue_lines(script::ANGLERS_WIN, AfterDialogue::None);
                        return;
                    }
                    self.phase = Phase::RapBattle { bar, beat_t, score, notes, sel: 0, feedback: Some(("Clean rhyme!! Keep flowing!", 0.8)) };
                } else {
                    self.audio.play_sfx(&self.audio.sfx.bad, 0.45);
                    self.phase = Phase::RapBattle { bar, beat_t, score, notes, sel: pick, feedback: Some(("Off-rhythm bar! Match the end rhyme!", 0.9)) };
                }
            }
            Phase::Maze { mut depth, mut q_idx, mut pool_pos, mut order, mut dir_order, .. } => {
                let q = &maze::QUESTIONS[order[q_idx % 8]];
                if dir_order[pick] == q.dir {
                    depth += 1;
                    self.audio.play_sfx(&self.audio.sfx.click, 0.45);
                    if depth >= maze::MAZE_TARGET_DEPTH {
                        self.prog.maze_solved = true;
                        self.audio.play_sfx(&self.audio.sfx.good, 0.7);
                        self.fx.accent(true);
                        self.enter_biome(Biome::Slums, true);
                        return;
                    }
                } else {
                    depth = depth.saturating_sub(1);
                }
                pool_pos = (pool_pos + 1) % 8;
                if pool_pos == 0 {
                    order = maze::shuffled_cycle().0;
                }
                q_idx = pool_pos;
                dir_order = maze::shuffled_dirs();
                self.phase = Phase::Maze { depth, q_idx, sel: 0, pool_pos, order, dir_order };
            }
            Phase::AlgaeTemple { stage, .. } => {
                let tr = &cosma::ALGAE_TRIALS[stage];
                let fb = if pick == tr.correct {
                    self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                    Some((true, tr.infuse_note, 1.3))
                } else {
                    self.audio.play_sfx(&self.audio.sfx.bad, 0.5);
                    Some((false, "Crystal mismatch! Check the figure and poetic device!", 1.1))
                };
                self.phase = Phase::AlgaeTemple { stage, sel: pick, feedback: fb };
            }
            Phase::TorchPlace { mut lit } => {
                if pick < 3 {
                    lit[pick] = true;
                    self.audio.play_sfx(&self.audio.sfx.good, 0.5);
                }
                self.world.torches_lit = lit.iter().filter(|b| **b).count() as u32;
                if lit.iter().all(|b| *b) {
                    self.fx.accent(true);
                    self.phase = Phase::GenieQuiz { poem_idx: 0, q_idx: 0, sel: 0, feedback: None };
                } else {
                    self.phase = Phase::TorchPlace { lit };
                }
            }
            Phase::GenieQuiz { poem_idx, q_idx, .. } => {
                let q = &cosma::POEMS[poem_idx].questions[q_idx];
                let fb = if pick == q.correct {
                    self.audio.play_sfx(&self.audio.sfx.good, 0.6);
                    Some((true, q.feedback, 1.3))
                } else {
                    self.audio.play_sfx(&self.audio.sfx.bad, 0.6);
                    Some((false, "False interpretation! The Genie turns to a fresh poem...", 1.4))
                };
                self.phase = Phase::GenieQuiz { poem_idx, q_idx, sel: pick, feedback: fb };
            }
            other => {
                self.phase = other;
            }
        }
    }

    fn draw_crumble(&self, dist: f32, py: f32, lives: u8, shards: &[(f32, f32, f32, bool)], skin: &Skin) {
        let w = screen_width();
        let t = get_time() as f32;
        crate::art::draw_biome_bg(Biome::Slums, t, dist * 1600.0, &self.fx, &self.tex);
        self.world.draw_ground(&self.tex, dist * 1800.0, 556.0, t);

        // Spinning Portal at destination
        let portal_x = (w - 140.0) * (1.05 - dist).max(0.75) + 40.0;
        draw_texture_ex(&self.tex.props.portal_ring, portal_x, 380.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(140.0, 140.0)),
            rotation: t * 1.4,
            pivot: Some(vec2(portal_x + 70.0, 450.0)),
            ..Default::default()
        });

        // 4-frame Sprinting River & jagged Purple Void Shards
        let r_spr = &self.tex.river_walks[((t * 11.0) as usize) % 4];
        draw_texture_ex(r_spr, 192.0, py - 114.0 + 56.0, WHITE, DrawTextureParams { dest_size: Some(vec2(96.0, 114.0)), ..Default::default() });

        for (sx, sy, _, _) in shards {
            draw_texture_ex(&self.tex.ui.shard, sx - 20.0, sy - 20.0, WHITE, DrawTextureParams {
                dest_size: Some(vec2(40.0, 40.0)),
                rotation: t * 3.0,
                pivot: Some(vec2(*sx, *sy)),
                ..Default::default()
            });
        }

        draw_panel(&self.tex, 60.0, 24.0, w - 120.0, 86.0, skin);
        draw_tag_shadow(&format!("DIMENSION CRUMBLING!! RUN TO THE PORTAL  {}%", (dist * 100.0).min(100.0) as i32), 86.0, 62.0, 14, skin.accent);
        for i in 0..3 {
            let ht = if (i as u8) < lives { &self.tex.ui.heart_full } else { &self.tex.ui.heart_empty };
            draw_texture_ex(ht, w - 200.0 + i as f32 * 36.0, 44.0, WHITE, DrawTextureParams { dest_size: Some(vec2(30.0, 30.0)), ..Default::default() });
        }
        draw_dlg_shadow("Press [Space] / [Up] / Click to Jump over Void Shards!", 86.0, 90.0, 22, skin.ink);
    }

    fn draw_ending(&mut self, skin: &Skin) {
        let w = screen_width();
        draw_rectangle(0.0, 0.0, w, screen_height(), Color::new(0.02, 0.01, 0.04, 0.55));
        draw_panel(&self.tex, 150.0, 74.0, w - 300.0, 560.0, skin);

        if let Some(np) = self.tex.portrait_expr("Nadia", "cured") {
            draw_nine_slice(&self.tex, 190.0, 120.0, 152.0, 152.0, skin.chip, skin.accent);
            draw_texture_ex(np, 198.0, 128.0, WHITE, DrawTextureParams { dest_size: Some(vec2(136.0, 136.0)), ..Default::default() });
        }

        draw_texture_ex(&self.tex.ui.crown, 370.0, 124.0, WHITE, DrawTextureParams { dest_size: Some(vec2(32.0, 32.0)), ..Default::default() });
        draw_tag_shadow("CHRONICLE COMPLETE!", 412.0, 150.0, 20, skin.accent);
        draw_wrapped_rich(
            "Nadia drank the *Elixir* *of* *Literacy* and read every single line aloud without a smudge!",
            370.0,
            194.0,
            460.0,
            24,
            28.0,
            skin.ink,
            skin.accent,
        );

        let stats = format!(
            "Riddles Mastered: {} / {}\nStory Puzzles Cleared: {}/4\nKenny Duel Rank: {}",
            self.prog.total_riddles_right(),
            self.prog.total_riddles_seen(),
            self.prog.gate_solved as u8 + self.prog.maze_solved as u8 + self.prog.algae_solved as u8 + self.prog.rainbow_solved as u8,
            if self.prog.kenny_flawless { "Flawless Swift-Swimmer" } else { "True Survivor" },
        );
        draw_wrapped_rich(&stats, 190.0, 336.0, w - 380.0, 26, 34.0, skin.ink, skin.accent);

        if draw_btn(&self.tex, 250.0, 520.0, 240.0, 54.0, "PLAY AGAIN", skin, true) {
            self.audio.play_sfx(&self.audio.sfx.select, 0.7);
            self.start_new_game();
            return;
        }
        if draw_btn(&self.tex, 530.0, 520.0, 240.0, 54.0, "TITLE SCREEN", skin, false) {
            self.phase = Phase::Title;
            self.audio.ensure_music(MusicTrack::Town);
        }
    }
}
