//! Scene state machine. Overworld-first: walk to NPCs and props, Z to engage.

use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};
use crate::art::{Textures, draw_biome_bg};
use crate::art::fx::Fx;
use crate::audio::AudioSys;
use crate::game::script::{self, Biome};
use crate::game::ui::{DialogueBox, draw_wrapped_rich, draw_wrapped_col, wrapped_line_count, quiz_header, quiz_header_skinned, draw_option_row, draw_option_row_skinned, Skin, draw_tag, draw_dlg_text, dlg_width, tag_width, draw_storybook, draw_vial};

/// Where the Rainbow Road paddle catches: one number, used by both the update
/// rule and the drawing, so they can never drift apart.
const PADDLE_Y: f32 = 505.0;

/// The autoplay harness presses "z" once per tick through this flag, so every
/// phase advances through its real input path instead of a parallel one.
static BOT_ADV: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn bot_adv() -> bool {
    BOT_ADV.load(std::sync::atomic::Ordering::Relaxed)
}

/// Readable name for a punctuation mark, shown under its tile in the gate UI.
fn mark_name(m: &str) -> &'static str {
    match m {
        "\u{2014}" => "em dash",
        "-" => "hyphen",
        "." => "period",
        "," => "comma",
        ":" => "colon",
        ";" => "semicolon",
        "\u{2026}" => "ellipsis",
        "!" => "excl.",
        "?" => "question",
        _ => "mark",
    }
}
use crate::game::world::World;
use crate::game::{Progress, jin_battle, gate, maze, cosma};

#[derive(PartialEq, Clone, Copy)]
pub enum Phase {
    Title,
    /// Walking in a biome overworld.
    Explore,
    /// Cutscene-ish dialogue only (intro, wake-up, guards, kandy, trapdoor...).
    Scene,
    JinMeet,
    JinBattle,
    JinAfter,
    Riddle,
    KandyBridge,
    GatePuzzle,
    MazePuzzle,
    StarCatch,
    Genie,
    Qte,
    /// Act title card shown between biomes.
    NextEpisode,
    /// Swap the overworld to `next_biome` (Aquarius to Cosma).
    ActChange,
    Crumble,
    Ending,
}

pub struct Game {
    phase: Phase,
    biome: Biome,
    progress: Progress,
    tex: Textures,
    audio: AudioSys,
    /// Screen-space effects: post-processing, camera feel, transitions.
    fx: Fx,
    dlg: DialogueBox,
    world: World,

    // flow
    #[allow(dead_code)]
    return_phase: Phase,
    scene_lines: &'static [script::Line],
    scene_i: usize,
    scene_next: Phase,
    intro_i: usize,
    intro_t: f32,
    last_choice: String,

    // jin battle
    jin_step: usize,
    jin_qs: Vec<usize>,
    jin_i: usize,
    jin_correct: u32,
    jin_sel: usize,
    jin_set_reduced: bool,
    jin_done: bool,
    jin_offer: bool,
    jin_losses: u32,
    after_stage: usize,

    // riddles
    riddle_sel: usize,
    riddle_answered: bool,
    riddle_right: Option<bool>,
    /// Which riddle set is active: 0 candy, 1 aqua, 2 cosma.
    riddle_set: u8,

    // gate
    gate_slot: usize,
    gate_sel: usize,
    gate_marks: Vec<Option<&'static str>>,
    gate_options: Vec<&'static str>,
    gate_msg: Option<(String, f32)>,

    // maze
    maze_order: Vec<usize>,
    maze_step: usize,
    maze_sel: usize,
    maze_depth: f32,

    // cosma
    poem: usize,
    star_meter: u32,
    lives: u32,
    slab: u32,
    stars: Vec<(f32, f32, f32, bool)>,
    paddle_pos: f32,
    genie_sel: usize,
    genie_shown: bool,
    cosma_fail: bool,

    kandy_asked: bool,
    kandy_reaction: Option<&'static [script::Line]>,

    // trapdoor / qte
    qte_line: usize,
    trap_t: f32,
    qte_progress: f32,

    // crumble / ending
    crumble_t: f32,
    end_line: usize,

    amb_playing: Option<&'static str>,
    /// Characters revealed last frame, for the typewriter blips.
    dlg_chars: usize,
    fade: f32,
    pub title_sel: usize,
    pub shake_t: f32,
    save_exists: bool,
    next_biome: Biome,
    /// Autoplay QA harness: walks the whole script with the real input paths.
    bot: bool,
    bot_t: f32,
    bot_last: Phase,
    debug_overlay: bool,
    /// Clicked an actor farther than arm's reach: (target x, action once arrived).
    pending_talk: Option<(f32, &'static str)>,
}

const AMB_VOL: f32 = 0.12;

impl Game {
    pub async fn new() -> Self {
        crate::game::ui::ensure_fonts().await;
        let tex = Textures::load().await;
        let audio = crate::audio::load().await;
        let world = World::new(Biome::Candy, &tex);
        let fx = Fx::new(screen_width(), screen_height()).await;
        Self {
            phase: Phase::Title,
            biome: Biome::Candy,
            progress: Progress::new(),
            tex,
            audio,
            fx,
            dlg: DialogueBox::new(),
            world,
            return_phase: Phase::Explore,
            scene_lines: script::INTRO_SCENE,
            scene_i: 0,
            scene_next: Phase::Explore,
            intro_i: 0,
            intro_t: 0.0,
            last_choice: String::new(),
            jin_step: 0,
            jin_qs: Vec::new(),
            jin_i: 0,
            jin_correct: 0,
            jin_sel: 0,
            jin_set_reduced: false,
            jin_done: false,
            jin_offer: false,
            jin_losses: 0,
            after_stage: 0,
            riddle_sel: 0,
            riddle_answered: false,
            riddle_right: None,
            riddle_set: 0,
            gate_slot: 0,
            gate_sel: 0,
            gate_marks: Vec::new(),
            gate_options: Vec::new(),
            gate_msg: None,
            maze_order: Vec::new(),
            maze_step: 0,
            maze_sel: 0,
            maze_depth: 0.0,
            poem: 0,
            star_meter: 0,
            lives: cosma::LIVES,
            slab: 0,
            stars: Vec::new(),
            paddle_pos: 0.5,
            genie_sel: 0,
            genie_shown: false,
            cosma_fail: false,
            kandy_asked: false,
            kandy_reaction: None,
            qte_line: 0,
            trap_t: 0.0,
            qte_progress: 0.0,
            crumble_t: 0.0,
            end_line: 0,
            amb_playing: None,
            dlg_chars: 0,
            fade: 1.0,
            title_sel: 0,
            shake_t: 0.0,
            save_exists: crate::game::save::load().is_some(),
            next_biome: Biome::Aquarius,
            bot: false,
            bot_t: 0.0,
            bot_last: Phase::Title,
            debug_overlay: false,
            pending_talk: None,
        }
    }

    /// Jump straight into a free Candy Land overworld (for --shot debugging).
    pub fn boot_candy(&mut self) {
        self.boot_biome(Biome::Candy);
    }

    /// The last fully composited frame, for debug screenshots.
    pub fn shot_image(&self) -> Image {
        self.fx.capture()
    }

    /// Jump straight into the Aquarius overworld (for --shot-aqua QA).
    pub fn boot_aqua(&mut self) {
        self.boot_biome(Biome::Aquarius);
        self.world.player.0 = 1500.0;
        self.world.cam = self.world.target_cam();
    }

    /// Jump straight into the Cosma void overworld (for --shot-cosma QA).
    pub fn boot_cosma(&mut self) {
        self.boot_biome(Biome::Cosma);
        self.world.player.0 = 620.0;
        self.world.cam = self.world.target_cam();
    }

    fn boot_biome(&mut self, biome: Biome) {
        self.phase = Phase::Explore;
        self.biome = biome;
        self.world = World::new(biome, &self.tex);
        self.fade = 0.0;
        let amb = match biome {
            Biome::Candy => "candy",
            Biome::Aquarius => "aqua",
            Biome::Cosma => "cosma",
            Biome::Home => "house",
        };
        self.amb(amb);
        self.fx.iris_open();
        self.fx.punch(0.035);
    }

    /// Open a demo dialogue over the world so UI can be screenshotted.
    pub fn demo_dialog(&mut self) {
        self.boot_candy();
        self.world.player.0 = 610.0;
        self.dlg.ask(
            "Jin",
            "Howdy! I was just in the forest grabbing licorice. The whole dialogue panel, typewriter and choices get one last look before we ship.",
            &[("Where am I", "where"), ("No", "no")],
            &self.tex,
        );
    }

    /// Start the automated playthrough (`--autoplay`). It does not touch the
    /// player's save file: progress lives in memory and autosaves are skipped.
    pub fn autoplay(&mut self) {
        self.bot = true;
        self.progress = Progress::new();
        self.phase = Phase::Scene;
        self.scene_lines = script::INTRO_SCENE;
        self.scene_next = Phase::Explore;
        self.scene_i = 0;
        self.intro_i = 0;
        self.intro_t = 0.0;
        self.fade = 0.0;
        println!("[autoplay] start");
    }

    /// One autoplay decision: press z, and aim the current puzzle at the right
    /// answer so the harness actually gets through the whole script.
    fn bot_step(&mut self) {
        BOT_ADV.store(true, std::sync::atomic::Ordering::Relaxed);
        self.dlg.force_advance = true;
        match self.phase {
            Phase::Explore => {
                if let Some(a) = self.world.nearest_action() {
                    let blocked = match a {
                        // already done with this: walk past instead of re-triggering
                        "jin" => self.progress.has_horse || self.jin_losses > 0,
                        "clue" => true,
                        "riddle" => self.riddles_done() >= 2,
                        "gate" => self.progress.candy_riddles_done < 2,
                        "cave" => self.progress.aqua_riddles_done < 2,
                        "stars" => self.progress.ingredients >= 3,
                        "rift" => !self.progress.maze_solved,
                        "portal" => self.progress.ingredients < 3,
                        "nadia" => self.progress.ingredients < 3,
                        _ => false,
                    };
                    if blocked {
                        let px = self.world.player.0;
                        self.world.player.0 = (px + 220.0).min(self.world.length - 40.0);
                    } else {
                        self.engage(a);
                    }
                } else {
                    self.world.player.0 = (self.world.player.0 + 80.0).min(self.world.length - 40.0);
                }
                self.world.cam = self.world.target_cam();
            }
            Phase::JinBattle => {
                let set = if self.jin_set_reduced { jin_battle::REDUCED_SET } else { jin_battle::FULL_SET };
                if self.jin_i < self.jin_qs.len() {
                    self.jin_sel = set[self.jin_qs[self.jin_i]].correct;
                }
            }
            Phase::Riddle => {
                let set = self.riddle_set_ref();
                let idx = self.riddles_done() as usize % set.len();
                self.riddle_sel = set[idx].answer;
            }
            Phase::GatePuzzle => {
                let si = self.gate_slot.min(gate::THRONE_GATE.slots.len() - 1);
                let want = gate::THRONE_GATE.slots[si].answer;
                if let Some(pos) = self.gate_options.iter().position(|o| *o == want) {
                    self.gate_sel = pos;
                }
            }
            Phase::MazePuzzle => {
                let n = self.maze_step.min(self.maze_order.len() - 1);
                self.maze_sel = maze::MAZE_QUESTIONS[self.maze_order[n]].dir;
            }
            Phase::StarCatch => {
                // track the lowest live star
                if let Some(s) = self.stars.iter().filter(|s| !s.3)
                    .min_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
                {
                    let want = ((s.0 - 80.0) / 864.0).clamp(0.0, 1.0);
                    self.paddle_pos = (self.paddle_pos + (want - self.paddle_pos).signum() * 0.10).clamp(0.0, 1.0);
                }
            }
            Phase::Genie => {
                if !self.dlg.open && !self.cosma_fail {
                    self.genie_sel = cosma::POEMS[self.poem].answer;
                }
            }
            Phase::Qte => {
                if !self.dlg.open { self.qte_progress = 1.01; }
            }
            _ => {}
        }
    }

    /// Apply a loaded save.
    pub fn apply_save(&mut self, d: crate::game::save::SaveData) {
        self.progress.has_horse = d.has_horse;
        crate::game::set_horse(d.has_horse);
        self.progress.candy_riddles_done = d.candy_riddles;
        self.progress.aqua_riddles_done = d.aqua_riddles;
        self.progress.cosma_riddles_done = d.cosma_riddles;
        self.progress.gate_solved = d.gate_solved;
        self.progress.maze_solved = d.maze_solved;
        self.progress.ingredients = d.ingredients;
        self.biome = d.biome.to_biome();
        self.world = World::new(self.biome, &self.tex);
        self.world.player.0 = d.player_x.clamp(60.0, self.world.length - 60.0);
        self.world.cam = self.world.target_cam();
        self.amb_playing = None;
        let amb = match self.biome { Biome::Candy => "candy", Biome::Aquarius => "aqua", Biome::Cosma => "cosma", Biome::Home => "house" };
        self.amb(amb);
        self.phase = Phase::Explore;
        self.fade = 0.0;
    }

    /// Snapshot the current run into SaveData.
    pub fn snapshot(&self) -> crate::game::save::SaveData {
        crate::game::save::SaveData {
            has_horse: self.progress.has_horse,
            candy_riddles: self.progress.candy_riddles_done,
            aqua_riddles: self.progress.aqua_riddles_done,
            cosma_riddles: self.progress.cosma_riddles_done,
            gate_solved: self.progress.gate_solved,
            maze_solved: self.progress.maze_solved,
            ingredients: self.progress.ingredients,
            biome: crate::game::save::BiomeId::from_biome(self.biome),
            player_x: self.world.player.0,
        }
    }

    /// Save and mark progression checkpoints automatically.
    pub fn autosave(&self) {
        if self.bot {
            return; // the harness must never overwrite the player's save
        }
        let _ = crate::game::save::save(&self.snapshot());
    }

    /// Wipe save and reset the run (called from title "new game").
    pub fn reset_run(&mut self) {
        crate::game::save::clear();
        crate::game::set_horse(false);
        self.progress = Progress::new();
        self.biome = Biome::Candy;
        self.world = World::new(Biome::Candy, &self.tex);
        self.phase = Phase::Scene;
        self.scene_lines = script::INTRO_SCENE;
        self.scene_i = 0;
        self.intro_i = 0;
        self.intro_t = 0.0;
        self.amb_playing = None;
        self.fade = 1.0;
        self.jin_done = false;
        self.jin_offer = false;
        self.jin_losses = 0;
        self.jin_set_reduced = false;
        self.kandy_asked = false;
        self.kandy_reaction = None;
        self.end_line = 0;
        self.cosma_fail = false;
        self.poem = 0;
        self.slab = 0;
        self.pending_talk = None;
    }

    fn amb(&mut self, which: &'static str) {
        if self.amb_playing != Some(which) {
            let snd = match which {
                "house" => &self.audio.amb.house,
                "candy" => &self.audio.amb.candy,
                "aqua" => &self.audio.amb.aqua,
                _ => &self.audio.amb.cosma,
            };
            play_sound(snd, PlaySoundParams { looped: true, volume: AMB_VOL });
            self.amb_playing = Some(which);
        }
    }

    /// Typewriter blips, driven from one place: any screen that shows dialogue
    /// gets them, and the pitch-ish variation comes from sounding only every
    /// other character advancing.
    fn typewriter_sfx(&mut self) {
        let shown = self.dlg.chars_shown();
        if shown != self.dlg_chars {
            if shown > self.dlg_chars && shown % 2 == 0 {
                play_sound(&self.audio.sfx.blip, PlaySoundParams { looped: false, volume: 0.09 });
            }
            self.dlg_chars = shown;
        }
    }

    fn adv_pressed() -> bool {
        bot_adv() || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter)
            || is_mouse_button_pressed(MouseButton::Left)
    }

    /// Start a dialogue-only scene run of Lines, then go to scene_next.
    fn play_scene(&mut self, lines: &'static [script::Line], next: Phase) {
        self.scene_lines = lines;
        self.scene_i = 0;
        self.scene_next = next;
        self.phase = Phase::Scene;
        let ln = &lines[0];
        self.dlg.say(ln.who, ln.text, &self.tex);
    }

    pub fn update(&mut self, dt_raw: f32) {
        self.fx.tick(dt_raw);
        // Conversations lift the camera so the speaker's body stays visible
        // above the dialogue panel instead of being cut off at the knees.
        let world_scene = self.phase == Phase::Scene
            && self.scene_lines.as_ptr() != script::INTRO_SCENE.as_ptr();
        let talking = world_scene
            || matches!(self.phase, Phase::JinMeet | Phase::JinAfter | Phase::KandyBridge);
        let want_lift = if talking && self.dlg.open { 72.0 } else { 0.0 };
        self.fx.lift += (want_lift - self.fx.lift) * (dt_raw * 9.0).min(1.0);
        // Hit-stop: an impact drops the world into slow-motion for a few
        // frames. It is the cheapest way to make a right answer feel like it
        // landed, and it needs no new state on any puzzle.
        let dt = if self.fx.freeze > 0.0 { dt_raw * 0.07 } else { dt_raw };
        self.fade = (self.fade + dt * 2.0).min(1.0);
        self.shake_t = (self.shake_t - dt).max(0.0);
        if is_key_pressed(KeyCode::F1) {
            self.debug_overlay = !self.debug_overlay;
        }
        // F2: skip straight to the next act (playtest shortcut through the
        // whole script without replaying the earlier biomes).
        if is_key_pressed(KeyCode::F2) {
            let next = match self.biome {
                Biome::Candy => Biome::Aquarius,
                Biome::Aquarius => Biome::Cosma,
                _ => Biome::Home,
            };
            self.enter_biome_act(next);
            self.world.set_hint("Debug: skipped to the next act");
        }
        // F3 cycles biomes (playtest QA): Candy -> Aquarius -> Cosma -> Home
        if is_key_pressed(KeyCode::F3) {
            let next = match self.biome {
                Biome::Candy => Biome::Aquarius,
                Biome::Aquarius => Biome::Cosma,
                Biome::Cosma => Biome::Home,
                Biome::Home => Biome::Candy,
            };
            self.biome = next;
            self.world = World::new(next, &self.tex);
            self.phase = Phase::Explore;
            self.fade = 0.0;
        }
        if self.bot {
            self.bot_t += dt_raw;
            if self.bot_t >= 0.09 {
                self.bot_t = 0.0;
                self.bot_step();
            }
        }
        match self.phase {
            Phase::Title => self.up_title(),
            Phase::Scene => self.up_scene(),
            Phase::Explore => self.up_explore(),
            Phase::JinMeet => self.up_jin_meet(),
            Phase::JinBattle => self.up_jin_battle(),
            Phase::JinAfter => self.up_jin_after(),
            Phase::Riddle => self.up_riddle(),
            Phase::KandyBridge => self.up_kandy_bridge(),
            Phase::GatePuzzle => self.up_gate(),
            Phase::MazePuzzle => self.up_maze(),
            Phase::StarCatch => self.up_star_catch(dt),
            Phase::Genie => self.up_genie(),
            Phase::Qte => self.up_qte(dt),
            Phase::NextEpisode => self.up_next(),
            Phase::ActChange => self.up_act_change(),
            Phase::Crumble => self.up_crumble(dt),
            Phase::Ending => self.up_ending(),
        }
        self.typewriter_sfx();
        if self.bot {
            if self.phase != self.bot_last {
                self.bot_last = self.phase;
                let biome = match self.biome {
                    Biome::Candy => "candy",
                    Biome::Aquarius => "aquarius",
                    Biome::Cosma => "cosma",
                    Biome::Home => "home",
                };
                println!("[autoplay] {}  [{}]  elixir {}/3", self.phase_name(), biome, self.progress.ingredients);
            }
            if self.phase == Phase::Ending && !self.dlg.open && self.end_line >= script::ENDING.len() {
                println!("[autoplay] ENDING REACHED");
                std::process::exit(0);
            }
            BOT_ADV.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn draw(&mut self) {
        // The scene goes into an offscreen buffer first; the effects layer then
        // composites it with bloom, the biome colour grade, a vignette, film
        // grain and the transition iris.
        self.fx.begin();
        match self.phase {
            Phase::Title => self.dr_title(),
            Phase::Scene => {
                if self.scene_lines.as_ptr() == script::INTRO_SCENE.as_ptr() {
                    self.dr_intro();
                } else {
                    self.dr_world_behind();
                    self.dlg.draw();
                }
            }
            Phase::Explore => {
                self.dr_world();
                self.draw_hud();
            }
            Phase::JinMeet | Phase::JinAfter | Phase::KandyBridge => {
                self.dr_world_behind();
                self.dlg.draw();
            }
            Phase::JinBattle => self.dr_jin_battle(),
            Phase::Riddle => self.dr_riddle(),
            Phase::GatePuzzle => self.dr_gate(),
            Phase::MazePuzzle => self.dr_maze(),
            Phase::StarCatch => self.dr_stars(),
            Phase::Genie => self.dr_genie(),
            Phase::Qte => self.dr_qte(),
            Phase::NextEpisode => self.dr_next(),
            Phase::ActChange => {}
            // (ActChange swaps the world in the same frame; nothing to draw.)
            Phase::Crumble => self.dr_crumble(),
            Phase::Ending => self.dr_ending(),
        }
        if self.debug_overlay {
            self.dr_debug();
        }
        self.fx.finish(self.fade);
    }

    fn shake_off(&self) -> f32 {
        if self.shake_t > 0.0 {
            rand::gen_range(-4.0, 4.0) * (self.shake_t / 0.35)
        } else { 0.0 }
    }

    fn dr_world(&self) {
        let t = get_time() as f32;
        draw_biome_bg(self.biome, t, self.world.cam, &self.fx);
        self.world.draw(&self.tex, &self.fx, t, self.shake_off(), !self.dlg.open);
        self.fx.begin_ui();
        self.dlg.draw();
    }

    fn dr_world_behind(&self) {
        let t = get_time() as f32;
        draw_biome_bg(self.biome, t, self.world.cam, &self.fx);
        self.world.draw(&self.tex, &self.fx, t, self.shake_off(), false);
        self.fx.begin_ui();
    }

    fn draw_hud(&self) {
        let w = screen_width();
        let msg = format!("elixir {}/3", self.progress.ingredients);
        let tw = tag_width(&msg, 13) + 26.0;
        draw_rectangle(14.0, 12.0, tw, 30.0, Color::from_rgba(255, 246, 233, 235));
        draw_rectangle_lines(14.0, 12.0, tw, 30.0, 2.0, Color::from_rgba(74, 42, 99, 255));
        draw_tag(&msg, 27.0, 31.0, 13, Color::from_rgba(74, 42, 99, 255));
        // three vials beside the counter: one fills per biome component
        for i in 0..3i32 {
            let fill = (self.progress.ingredients as i32 - i).clamp(0, 1) as f32;
            draw_vial(14.0 + tw + 22.0 + i as f32 * 21.0, 46.0, 34.0, fill);
        }
        if self.progress.has_horse {
            draw_dlg_text("riding the horse", 20.0, 54.0, 22, Color::from_rgba(255, 214, 230, 230));
        }
        if w < 1100.0 && self.world.hint_t > 0.0 {
            let _ = w;
        }
    }

    fn dr_intro(&self) {
        let w = screen_width();
        let h = screen_height();
        // storybook page: plum night around a cream card
        clear_background(Color::from_rgba(26, 14, 43, 255));
        draw_rectangle(0.0, 0.0, w, 120.0, Color::from_rgba(46, 26, 71, 255));
        draw_rectangle(0.0, h - 120.0, w, 120.0, Color::from_rgba(46, 26, 71, 255));
        if self.intro_i < script::INTRO_LINES.len() {
            let cw = 780.0;
            let ch = 390.0;
            let cx = (w - cw) / 2.0;
            let cy = h * 0.5 - ch * 0.5;
            // card
            draw_rectangle(cx - 6.0, cy + 6.0, cw + 12.0, ch + 12.0, Color::from_rgba(226, 106, 156, 90));
            draw_rectangle(cx, cy, cw, ch, Color::from_rgba(255, 246, 233, 255));
            draw_rectangle_lines(cx - 2.0, cy - 2.0, cw + 4.0, ch + 4.0, 4.0, Color::from_rgba(74, 42, 99, 255));
            // picture zone on the card
            let py = cy + 38.0;
            match self.intro_i {
                0..=1 => draw_texture_ex(&self.tex.nadia, w * 0.5 - 57.0, py, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(114.0, 171.0)), ..Default::default() }),
                2..=3 => draw_texture_ex(&self.tex.grandparent, w * 0.5 - 57.0, py, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(114.0, 171.0)), ..Default::default() }),
                _ => {
                    let glow = 0.5 + 0.5 * (get_time() as f32 * 2.0).sin();
                    // the heirloom itself: gilt storybook with a ribbon, not a
                    // red rectangle standing in for "item"
                    draw_storybook(w * 0.5, py + 186.0, 4.8, glow);
                }
            }
            // thin seal between the plate and the caption
            draw_rectangle(w * 0.5 - 130.0, py + 202.0, 260.0, 2.0, Color::from_rgba(255, 214, 230, 255));
            // narration in ink below the picture
            let txt = script::INTRO_LINES[self.intro_i];
            draw_wrapped_col(txt, w * 0.5 - 320.0, py + 238.0, 640.0, 36.0, 34, Color::from_rgba(46, 26, 71, 255));
        } else {
            // final white-to-night page turn
            let a = (1.0 - self.intro_t).clamp(0.0, 1.0);
            draw_rectangle(0.0, 0.0, w, h, Color::new(a, a * 0.97, a * 0.92, 1.0));
        }
        if (get_time() * 2.0) as i32 % 2 == 0 {
            draw_tag("z / click", w - 150.0, h - 34.0, 12, Color::from_rgba(255, 214, 230, 200));
        }
    }

    // ---------- SCENE WALKER ----------
    fn up_scene(&mut self) {
        // intro counts panels; other scenes walk dialogue lines
        if self.scene_lines.as_ptr() == script::INTRO_SCENE.as_ptr() {
            self.amb("house");
            self.intro_t += get_frame_time();
            if Self::adv_pressed() && self.intro_t > 0.4 {
                self.page_play();
                self.intro_i += 1;
                self.intro_t = 0.0;
                if self.intro_i >= script::INTRO_LINES.len() + 1 {
                    // arrive in candy land
                    self.biome = Biome::Candy;
                    self.world = World::new(Biome::Candy, &self.tex);
                    self.fade = 0.0;
                    self.amb("candy");
                    self.play_scene(script::WAKE_SCENE, Phase::Explore);
                }
            }
            return;
        }
        if let Some(p) = self.dlg.update(get_frame_time()) {
            self.scene_i += 1;
            if self.scene_i < self.scene_lines.len() {
                let ln = &self.scene_lines[self.scene_i];
                self.dlg.say(ln.who, ln.text, &self.tex);
            } else {
                self.dlg.close();
                let next = self.scene_next;
                let _ = p;
                match next {
                    Phase::Explore => {
                        self.phase = Phase::Explore;
                        self.fade = 0.0;
                    }
                    Phase::JinMeet => {
                        self.phase = Phase::JinMeet;
                        self.jin_step = 0;
                        self.dlg.say("Jin", script::JIN_INTRO[0].text, &self.tex);
                    }
                    Phase::GatePuzzle => self.enter_gate(),
                    Phase::MazePuzzle => self.enter_maze(),
                    Phase::Qte => {
                        self.phase = Phase::Qte;
                        self.qte_progress = 0.3;
                        self.trap_t = 0.0;
                    }
                    Phase::Crumble => {
                        self.phase = Phase::Crumble;
                        self.crumble_t = 0.0;
                    }
                    Phase::ActChange => self.up_act_change(),
                    _ => self.phase = next,
                }
            }
        }
    }

    // ---------- EXPLORE ----------
    fn up_explore(&mut self) {
        self.world.update(get_frame_time(), false);
        if self.world.step_hit {
            self.world.step_hit = false;
            self.step_play();
        }
        // Esc: back out of a pending click-to-walk, or close a prompt
        if is_key_pressed(KeyCode::Escape) {
            if self.world.click.is_some() {
                self.world.click = None;
                self.ui_back();
            }
        }
        // F5 quicksave
        if is_key_pressed(KeyCode::F5) {
            self.autosave();
            self.world.set_hint("saved.");
        }
        let mdown = is_mouse_button_pressed(MouseButton::Left);
        let kb_adv = Self::adv_pressed();
        let (mx, my) = mouse_position();
        let mut action: Option<&'static str> = None;
        let mut force = false;

        if self.world.interact_cd <= 0.0 && (kb_adv || mdown) {
            // A click can target a person or thing directly (walk there, then talk).
            if mdown {
                match self.world.hit_click(mx, my) {
                    Some((wx, act)) => {
                        if (self.world.player.0 - wx).abs() <= crate::game::world::REACH * 1.5 {
                            action = Some(act);
                            force = true;
                        } else if !kb_adv {
                            self.world.click = Some(wx);
                            self.pending_talk = Some((wx, act));
                        }
                    }
                    None => {
                        if !kb_adv {
                            let tx = (self.world.cam + mx).clamp(60.0, self.world.length - 60.0);
                            self.world.click = Some(tx);
                        }
                    }
                }
            }
            // Keyboard: only interacts when already within reach of something.
            if action.is_none() && kb_adv {
                if let Some(a) = self.world.nearest_action() {
                    action = Some(a);
                    force = true;
                }
            }
        }
        // Arrived at a click-to-talk destination.
        if action.is_none() {
            if let Some((tx, act)) = self.pending_talk {
                if (self.world.player.0 - tx).abs() <= crate::game::world::REACH + 24.0 {
                    self.pending_talk = None;
                    self.world.click = None;
                    action = Some(act);
                    force = true;
                }
            }
        }
        if !force {
            return;
        }
        self.pending_talk = None;
        self.world.interact_cd = 0.35;
        self.engage(action.unwrap());
    }

    /// Resolve an interaction the player triggered by talking to or touching
    /// something. Kept separate from input so the autoplay harness can use the
    /// exact same transitions.
    fn engage(&mut self, action: &'static str) {
        match action {
            "jin" => {
                self.dlg.close();
                self.phase = Phase::JinMeet;
                self.jin_step = 0;
                self.last_choice.clear();
                self.dlg.say("Jin", script::JIN_INTRO[0].text, &self.tex);
            }
            "guards" => {
                self.play_scene(script::GUARDS, Phase::Explore);
                self.progress.done.insert("guards");
                self.world.actors.retain(|a| a.action != "guards");
                self.autosave();
            }
            "kandy" => self.play_scene(script::KANDY_INTRO_SCENE, Phase::KandyBridge),
            "riddle" => self.enter_riddle(),
            "gate" => {
                if self.progress.candy_riddles_done < 2 {
                    self.world.set_hint("The gate ignores you. A riddle stone first.");
                } else {
                    self.enter_gate();
                }
            }
            "cave" => {
                if self.progress.aqua_riddles_done < 2 {
                    self.world.set_hint("Voices whisper: answer the stones first.");
                } else {
                    self.enter_maze();
                }
            }
            "clue" => {
                // NPC clue from nearest actor label
                let label = self.world.nearest_info().map(|(l, _, _)| l).unwrap_or("");
                let lines: &'static [script::Line] = match label {
                    "Gillbert" => script::CLUE_GILL,
                    "Scalleon" => script::CLUE_SCAL,
                    _ => script::CLUE_LADY,
                };
                self.play_scene(lines, Phase::Explore);
            }
            "stars" => {
                if self.progress.cosma_riddles_done < 2 {
                    self.world.set_hint("The torches sleep. The riddle stones call first.");
                } else {
                    self.enter_stars();
                }
            }
            "nadia" => {
                self.end_line = 0;
                self.dlg.close();
                self.phase = Phase::Ending;
                self.fade = 0.0;
            }
            "rift" => {
                if self.progress.maze_solved {
                    self.next_biome = Biome::Cosma;
                    self.play_scene(script::COSMA_ARRIVE, Phase::ActChange);
                } else {
                    self.world.set_hint("The rift is sealed. The Grave Cave must open first.");
                }
            }
            "portal" => {
                if self.progress.ingredients >= 3 {
                    self.play_scene(script::PORTAL_SCENE, Phase::Crumble);
                } else {
                    self.world.set_hint("The stone circle is dark. Light all three torches.");
                }
            }
            _ => {}
        }
    }

    fn enter_riddle(&mut self) {
        self.riddle_set = match self.biome {
            Biome::Candy => 0,
            Biome::Aquarius => 1,
            Biome::Cosma | Biome::Home => 2,
        };
        self.riddle_sel = 0;
        self.riddle_answered = false;
        self.riddle_right = None;
        self.phase = Phase::Riddle;
        self.ask_play();
    }

    fn riddle_set_ref(&self) -> &'static [script::Riddle] {
        match self.riddle_set {
            0 => script::CANDY_RIDDLES,
            1 => script::AQUARIUS_RIDDLES,
            _ => script::COSMA_RIDDLES,
        }
    }

    fn riddles_done(&self) -> u32 {
        match self.riddle_set {
            0 => self.progress.candy_riddles_done,
            1 => self.progress.aqua_riddles_done,
            _ => self.progress.cosma_riddles_done,
        }
    }

    fn bump_riddles(&mut self) {
        match self.riddle_set {
            0 => self.progress.candy_riddles_done += 1,
            1 => self.progress.aqua_riddles_done += 1,
            _ => self.progress.cosma_riddles_done += 1,
        }
    }

    // ---------- RIDDLE ----------
    fn up_riddle(&mut self) {
        let set = self.riddle_set_ref();
        let idx = self.riddles_done() as usize % set.len();
        let r = &set[idx];
        if !self.riddle_answered {
            let before = self.riddle_sel;
            if is_key_pressed(KeyCode::Up) { self.riddle_sel = self.riddle_sel.saturating_sub(1); }
            if is_key_pressed(KeyCode::Down) { self.riddle_sel = (self.riddle_sel + 1).min(3); }
            if self.riddle_sel != before { self.ui_move(); }
            for i in 0..4 {
                let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, _ => KeyCode::Key4 };
                if is_key_pressed(k) {
                    self.riddle_sel = i;
                    self.submit_riddle(r.answer);
                }
            }
            if Self::adv_pressed() {
                self.submit_riddle(r.answer);
            }
        } else if Self::adv_pressed() {
            if self.riddle_right == Some(false) {
                self.riddle_answered = false;
                self.riddle_right = None;
                self.riddle_sel = 0;
            } else {
                self.bump_riddles();
                self.riddle_answered = false;
                self.riddle_right = None;
                self.riddle_sel = 0;
                self.phase = Phase::Explore;
                self.fade = 0.0;
                self.world.set_hint("A stone hums, satisfied.");
            }
        }
    }

    fn submit_riddle(&mut self, answer: usize) {
        self.riddle_answered = true;
        self.riddle_right = Some(self.riddle_sel == answer);
        let px = self.world.player.0;
        let py = self.world.player.1 + 20.0;
        if self.riddle_right == Some(true) {
            self.good_play();
            // burst for the spark, ring for the shockwave: together they read as
            // one event with a direction rather than a puff of pixels
            self.world.burst(px, py, 2, 14);
            self.world.ring(px, py, 2, 16, 210.0);
        } else {
            self.bad_play();
            self.shake_t = 0.35;
            self.world.burst(px, py, 0, 10);
            self.world.ring(px, py, 0, 12, 150.0);
        }
        self.autosave();
    }

    fn dr_riddle(&self) {
        self.dr_world_behind();
        let set = self.riddle_set_ref();
        let idx = self.riddles_done() as usize % set.len();
        let r = &set[idx];
        let (title, skin) = match self.riddle_set {
            0 => ("CANDY LAND RIDDLE", Skin::Plum),
            1 => ("AQUARIUS RIDDLE", Skin::PearlSand),
            _ => ("COSMA RIDDLE", Skin::NeonSlab),
        };
        quiz_header_skinned(title, "", skin);
        let w = screen_width();
        // The riddle is a story passage: give it its own card so four lines of
        // text and the options never fight for the same band of the screen.
        let qlines = wrapped_line_count(r.question, 732.0, 27) as f32;
        let qh = qlines * 30.0 + 34.0;
        let qx = w * 0.5 - 390.0;
        let qy = 144.0;
        draw_rectangle(qx, qy, 780.0, qh, Color::from_rgba(30, 15, 48, 190));
        draw_rectangle_lines(qx, qy, 780.0, qh, 1.5, skin.edge());
        draw_wrapped_rich(r.question, qx + 24.0, qy + 32.0, 732.0, 30.0, 27,
            Color::from_rgba(244, 240, 250, 255), skin.edge());
        let row0 = (qy + qh + 34.0).max(268.0);
        for (i, o) in r.options.iter().enumerate() {
            draw_option_row_skinned(i, o, row0 + i as f32 * 44.0,
                i == self.riddle_sel,
                if self.riddle_answered { Some(i != r.answer) } else { None },
                skin);
        }
        if self.riddle_answered {
            let right = self.riddle_right == Some(true);
            let msg_y = row0 + 4.0 * 44.0 + 22.0;
            draw_text(
                if right { "Correct! z to return" } else { "Wrong. One try only. z restarts this stone's riddles" },
                w * 0.5 - 180.0, msg_y, 24.0,
                if right { GREEN } else { RED });
        }
    }

    // ---------- JIN ----------
    fn up_jin_meet(&mut self) {
        if let Some(p) = self.dlg.update(get_frame_time()) {
            if !p.is_empty() { self.last_choice = p.clone(); }
            match self.jin_step {
                0 => self.dlg.say("Jin", script::JIN_INTRO[1].text, &self.tex),
                1 => {
                    self.dlg.ask("Jin", script::JIN_INTRO[2].text, &[
                        ("Where am I", "w"),
                        ("Where are we", "w"),
                        ("No", "no"),
                    ], &self.tex);
                }
                2 => {
                    if self.last_choice == "no" {
                        self.dlg.say("Jin", script::JIN_NO[0].text, &self.tex);
                    } else {
                        self.dlg.say("Jin", script::JIN_WHERE[0].text, &self.tex);
                    }
                }
                3 => self.dlg.say("Jin", script::JIN_NO[1].text, &self.tex),
                4 => self.dlg.say("Jin", script::JIN_WARN[0].text, &self.tex),
                5 => {
                    self.dlg.ask("Jin", script::JIN_ANNOY[0].text, &[
                        ("In what way?", "a"),
                        ("how?", "a"),
                        ("annoying?", "a"),
                    ], &self.tex);
                }
                6 => self.dlg.say("Jin", script::JIN_ANNOY[1].text, &self.tex),
                7 => self.dlg.say("Jin", script::JIN_ANNOY[2].text, &self.tex),
                8 => self.dlg.say("Jin", script::JIN_ANNOY[3].text, &self.tex),
                9 => {
                    self.dlg.ask("Jin", "What do you ask?", &[
                        ("Risk?", "risk"),
                        ("Riddles?", "rd"),
                        ("Questions?", "rd"),
                    ], &self.tex);
                }
                10 => {
                    if self.last_choice == "risk" {
                        self.dlg.say("Jin", script::JIN_RISK[0].text, &self.tex);
                    } else {
                        self.dlg.say("Jin", script::JIN_RIDDLE_DEMO[0].text, &self.tex);
                    }
                }
                11 => self.dlg.say("Jin", script::JIN_RISK[1].text, &self.tex),
                _ => {
                    self.phase = Phase::JinBattle;
                    self.jin_qs = (0..jin_battle::FULL_SET.len()).collect();
                    self.jin_i = 0;
                    self.jin_correct = 0;
                    self.jin_done = false;
                    self.jin_offer = false;
                    return;
                }
            }
            self.jin_step += 1;
        }
    }

    fn up_jin_battle(&mut self) {
        if self.jin_done {
            if self.jin_offer {
                if let Some(p) = self.dlg.update(get_frame_time()) {
                    self.jin_offer = false;
                    if p == "retry" {
                        self.jin_set_reduced = true;
                        self.jin_qs = (0..jin_battle::REDUCED_SET.len()).collect();
                        self.jin_i = 0;
                        self.jin_correct = 0;
                        self.jin_done = false;
                    } else {
                        self.phase = Phase::JinMeet;
                        self.jin_step = 0;
                        self.dlg.say("Jin", script::JIN_INTRO[0].text, &self.tex);
                    }
                }
                return;
            }
            if let Some(_) = self.dlg.update(get_frame_time()) {
                if self.jin_correct == 0 && !self.jin_set_reduced {
                    self.jin_offer = true;
                    self.dlg.ask("Jin", "Challenge again? (fewer questions)", &[
                        ("Challenge him", "retry"),
                        ("Give up", "give"),
                    ], &self.tex);
                } else {
                    self.phase = Phase::JinAfter;
                    self.after_stage = 0;
                    self.dlg.close();
                }
            }
            return;
        }

        let set = if self.jin_set_reduced { jin_battle::REDUCED_SET } else { jin_battle::FULL_SET };
        if self.jin_i >= self.jin_qs.len() {
            self.jin_done = true;
            if self.jin_correct == 0 {
                self.jin_losses += 1;
                self.dlg.say("Jin", script::JIN_LOSE[0].text, &self.tex);
            } else {
                self.dlg.say("Jin", script::JIN_WIN[0].text, &self.tex);
            }
            return;
        }

        let q = &set[self.jin_qs[self.jin_i]];
        if is_key_pressed(KeyCode::Up) { self.jin_sel = self.jin_sel.saturating_sub(1); }
        if is_key_pressed(KeyCode::Down) { self.jin_sel = (self.jin_sel + 1).min(3); }
        for i in 0..4 {
            let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, _ => KeyCode::Key4 };
            if is_key_pressed(k) { self.jin_sel = i; self.jin_submit(q.correct); }
        }
        if Self::adv_pressed() {
            self.jin_submit(q.correct);
        }
    }

    fn jin_submit(&mut self, correct: usize) {
        if self.jin_sel as usize == correct {
            self.good_play();
            self.jin_correct += 1;
            self.world.burst(self.world.player.0, self.world.player.1 + 20.0, 2, 10);
        } else {
            self.bad_play();
            self.shake_t = 0.3;
        }
        self.jin_i += 1;
        self.jin_sel = 0;
    }

    fn dr_jin_battle(&self) {
        self.dr_world_behind();
        let w = screen_width();
        // answering the last question leaves jin_i one past the end on the same
        // frame, before jin_done is set: guard the index, not just the flag
        if self.jin_done || self.jin_i >= self.jin_qs.len() {
            self.dlg.draw();
            return;
        }
        let set = if self.jin_set_reduced { jin_battle::REDUCED_SET } else { jin_battle::FULL_SET };
        let q = &set[self.jin_qs[self.jin_i]];
        quiz_header(
            &format!("JIN'S BATTLE  {}/{}", self.jin_i + 1, self.jin_qs.len()),
            q.question,
        );
        for (i, o) in q.options.iter().enumerate() {
            draw_option_row(i, o, 250.0 + i as f32 * 44.0, i == self.jin_sel, None);
        }
        draw_text("1-4 / arrows + z", w - 220.0, screen_height() - 24.0, 16.0,
            Color::from_rgba(200, 200, 220, 140));
    }

    fn up_jin_after(&mut self) {
        if !self.dlg.open && self.after_stage == 0 {
            if self.jin_losses == 0 && self.jin_correct == self.jin_qs.len() as u32 {
                self.progress.has_horse = true;
                crate::game::set_horse(true);
                self.autosave();
                self.dlg.say("Jin", script::JIN_PERFECT[0].text, &self.tex);
            } else {
                self.dlg.say("Jin", script::JIN_LOST_ONE[0].text, &self.tex);
            }
        }
        if let Some(p) = self.dlg.update(get_frame_time()) {
            if !p.is_empty() { self.last_choice = p.clone(); }
            self.after_stage += 1;
            match self.after_stage {
                1 => self.dlg.say("(Narration)", script::JIN_BOOK_OFFER[0].text, &self.tex),
                2 => self.dlg.say("(Narration)", script::JIN_BOOK_OFFER[1].text, &self.tex),
                3 => {
                    self.dlg.ask("River", script::JIN_BOOK_RETURN[0].text, &[
                        ("Where did you get this book?", "b1"),
                        ("Where did you get that book?", "b1"),
                        ("That's my book.", "b2"),
                    ], &self.tex);
                }
                4 => {
                    if self.last_choice == "b2" {
                        self.dlg.say("Jin", script::JIN_BOOK_MINE[0].text, &self.tex);
                    } else {
                        self.dlg.say("Jin", script::JIN_BOOK_THIS[0].text, &self.tex);
                    }
                }
                5 => self.dlg.say("River", script::JIN_BOOK_RETURN[1].text, &self.tex),
                6 => self.dlg.say("Jin", script::JIN_BOOK_RETURN[2].text, &self.tex),
                _ => {
                    self.dlg.close();
                    self.phase = Phase::Explore;
                    self.fade = 0.0;
                    self.world.set_hint("Head right: riddle stones, then the Throne Gate.");
                }
            }
        }
    }

    // ---------- GATE ----------
    fn enter_gate(&mut self) {
        self.ask_play();
        self.gate_marks.clear();
        self.gate_slot = 0;
        self.gate_sel = 0;
        self.phase = Phase::GatePuzzle;
    }

    fn up_gate(&mut self) {
        if self.gate_marks.is_empty() {
            self.gate_marks = vec![None; gate::THRONE_GATE.slots.len()];
        }
        let slot = &gate::THRONE_GATE.slots[self.gate_slot.min(gate::THRONE_GATE.slots.len() - 1)];
        let mut opts: Vec<&'static str> = vec![slot.answer];
        opts.extend_from_slice(slot.decoys);
        let rot = self.gate_slot % opts.len();
        opts.rotate_left(rot);
        self.gate_options = opts;

        let gprev = self.gate_sel;
        if is_key_pressed(KeyCode::Up) { self.gate_sel = self.gate_sel.saturating_sub(1); }
        if is_key_pressed(KeyCode::Down) { self.gate_sel = (self.gate_sel + 1).min(4); }
        if self.gate_sel != gprev { self.ui_click(); }
        for i in 0..5 {
            let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, 3 => KeyCode::Key4, _ => KeyCode::Key5 };
            if is_key_pressed(k) { self.gate_sel = i; }
        }
        if Self::adv_pressed() {
            let chosen = self.gate_options[self.gate_sel.min(4)];
            self.gate_marks[self.gate_slot] = Some(chosen);
            if chosen == slot.answer {
                self.good_play();
                self.gate_slot += 1;
                self.gate_sel = 0;
                if self.gate_slot >= gate::THRONE_GATE.slots.len() {
                    self.progress.gate_solved = true;
                    self.progress.ingredients += 1;
                    self.reward_play();
                    self.progress.done.insert("gate");
                    self.world.set_hint("The Throne Gate grinds open!");
                    self.world.burst(2050.0, 470.0, 2, 30);
                    self.autosave();
                    self.phase = Phase::Explore;
                    self.fade = 0.0;
                    // remove gate prop so the path is clear
                    self.world.props.retain(|p| p.id != "gate");
                }
            } else {
                self.bad_play();
                self.gate_msg = Some(("Wrong mark. The stone stays silent.".to_string(), 1.5));
                self.gate_marks[self.gate_slot] = None;
            }
        }
        if let Some((_, t)) = &mut self.gate_msg {
            *t -= get_frame_time();
            if *t <= 0.0 { self.gate_msg = None; }
        }
    }

    /// The Throne Gate page: the passage typeset on a stone tablet with a
    /// punctuation box in every slot, and the five candidate marks below it.
    fn dr_gate(&self) {
        self.dr_world_behind();
        let w = screen_width();
        let h = screen_height();
        let skin = Skin::Plum;
        quiz_header_skinned("THE LONE STONE STORY OF THRONE GATE",
            "Fill every box. 1-5 pick a mark, z sets it. Tries allowed.", skin);

        let px = 46.0;
        let pw = w - 92.0;
        let py = 152.0;
        let ph = h - 292.0;
        draw_rectangle(px, py, pw, ph, Color::from_rgba(72, 64, 84, 244));
        draw_rectangle_lines(px, py, pw, ph, 3.0, Color::from_rgba(132, 118, 148, 255));
        draw_rectangle(px + 8.0, py + 8.0, pw - 16.0, 2.0, Color::from_rgba(170, 154, 186, 80));

        let size: u16 = 22;
        let lh = 24.0;
        let word_gap = dlg_width(" ", size);
        let x0 = px + 26.0;
        let xmax = px + pw - 26.0;
        let mut x = x0;
        let mut y = py + 34.0;
        let words: Vec<&str> = gate::THRONE_GATE.passage.split(' ').collect();
        let mut wi = 0usize;
        let body = Color::from_rgba(230, 220, 240, 255);
        let put_word = |word: &str, x: &mut f32, y: &mut f32| {
            let ww = dlg_width(word, size);
            if *x + ww > xmax && *x > x0 {
                *x = x0;
                *y += lh;
            }
            draw_dlg_text(word, *x, *y, size, body);
            *x += ww + word_gap;
        };
        for (si, slot) in gate::THRONE_GATE.slots.iter().enumerate() {
            while wi <= slot.after_word && wi < words.len() {
                put_word(words[wi], &mut x, &mut y);
                wi += 1;
            }
            if y > py + ph - 34.0 {
                break;
            }
            let cw = 26.0;
            if x + cw > xmax {
                x = x0;
                y += lh;
            }
            let hot = si == self.gate_slot;
            let mark = self.gate_marks.get(si).copied().flatten().unwrap_or("?");
            let fill = if hot {
                Color::from_rgba(226, 106, 156, 255)
            } else if mark == "?" {
                Color::from_rgba(48, 40, 58, 255)
            } else {
                Color::from_rgba(104, 88, 120, 255)
            };
            draw_rectangle(x, y - 19.0, cw, 25.0, fill);
            draw_rectangle_lines(x, y - 19.0, cw, 25.0, 1.5,
                if hot { Color::from_rgba(255, 246, 233, 255) } else { Color::from_rgba(178, 162, 196, 170) });
            let mw = dlg_width(mark, 21);
            draw_dlg_text(mark, x + (cw - mw) / 2.0, y + 2.0, 21,
                if mark == "?" { Color::from_rgba(150, 140, 164, 255) } else { WHITE });
            x += cw + word_gap;
        }
        while wi < words.len() && y <= py + ph - 20.0 {
            put_word(words[wi], &mut x, &mut y);
            wi += 1;
        }

        // candidate marks, as engraved tiles with names (punctuation is small)
        let done = self.gate_marks.iter().filter(|m| m.is_some()).count();
        draw_dlg_text(&format!("{}/{} marks", done, gate::THRONE_GATE.slots.len()),
            px + pw - 118.0, py + ph - 16.0, 22, Color::from_rgba(200, 188, 214, 220));
        if self.gate_slot < gate::THRONE_GATE.slots.len() {
            let tw = 152.0;
            let gap = 12.0;
            let total = self.gate_options.len() as f32 * tw + (self.gate_options.len() as f32 - 1.0) * gap;
            let tx0 = w * 0.5 - total * 0.5;
            for (i, o) in self.gate_options.iter().enumerate() {
                let tx = tx0 + i as f32 * (tw + gap);
                let ty = h - 150.0;
                let sel = i == self.gate_sel;
                draw_rectangle(tx, ty, tw, 66.0, if sel { Color::from_rgba(226, 106, 156, 255) } else { Color::from_rgba(48, 40, 58, 245) });
                draw_rectangle_lines(tx, ty, tw, 66.0, 2.0,
                    if sel { Color::from_rgba(255, 246, 233, 255) } else { Color::from_rgba(120, 104, 140, 200) });
                let sw = dlg_width(o, 30);
                draw_dlg_text(o, tx + (tw - sw) / 2.0, ty + 38.0, 30, WHITE);
                let nm = mark_name(o);
                let nw = dlg_width(nm, 16);
                draw_dlg_text(nm, tx + (tw - nw) / 2.0, ty + 58.0, 16,
                    Color::from_rgba(236, 224, 240, 220));
                if sel {
                    draw_dlg_text(">", tx - 22.0, ty + 40.0, 30, Color::from_rgba(255, 214, 230, 255));
                }
            }
            draw_dlg_text("1-5 pick   z set   arrows move", w * 0.5 - 130.0, h - 66.0, 20,
                Color::from_rgba(214, 200, 228, 200));
        }
        if let Some((msg, _)) = &self.gate_msg {
            let mw = dlg_width(msg, 22);
            draw_dlg_text(msg, w * 0.5 - mw / 2.0, 206.0, 22, Color::from_rgba(255, 224, 120, 255));
        }
    }

    // ---------- MAZE ----------
    fn enter_maze(&mut self) {
        self.ask_play();
        self.maze_order = (0..maze::MAZE_QUESTIONS.len()).collect();
        let mut seed = (get_time() * 1000.0) as u64 | 1;
        for i in (1..self.maze_order.len()).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = ((seed >> 33) as usize) % (i + 1);
            self.maze_order.swap(i, j);
        }
        self.maze_step = 0;
        self.maze_depth = 0.0;
        self.phase = Phase::MazePuzzle;
    }

    fn up_maze(&mut self) {
        let n = maze::MAZE_QUESTIONS.len();
        if self.maze_step >= n {
            self.progress.maze_solved = true;
            self.progress.ingredients += 1;
            self.reward_play();
            self.progress.done.insert("maze");
            self.world.set_hint("The cave opens to the far current!");
            self.world.burst(self.world.player.0, self.world.player.1 + 20.0, 1, 24);
            self.autosave();
            self.phase = Phase::Explore;
            self.fade = 0.0;
            self.world.props.retain(|p| p.id != "cave");
            return;
        }
        let q = &maze::MAZE_QUESTIONS[self.maze_order[self.maze_step]];
        let mprev = self.maze_sel;
        if is_key_pressed(KeyCode::Up) { self.maze_sel = (self.maze_sel + 3) % 4; }
        if is_key_pressed(KeyCode::Down) { self.maze_sel = (self.maze_sel + 1) % 4; }
        if self.maze_sel != mprev { self.ui_click(); }
        for i in 0..4 {
            let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, _ => KeyCode::Key4 };
            if is_key_pressed(k) { self.maze_sel = i; }
        }
        if Self::adv_pressed() {
            if self.maze_sel == q.dir {
                self.good_play();
                self.maze_step += 1;
                self.maze_depth += 300.0 / n as f32;
            } else {
                // silently sent back one step (per script)
                self.maze_step = self.maze_step.saturating_sub(1);
                self.maze_depth = (self.maze_depth - 300.0 / n as f32).max(0.0);
                self.bad_play();
            }
        }
    }

    fn dr_maze(&self) {
        self.dr_world_behind();
        let w = screen_width();
        // cave darkness
        draw_rectangle(0.0, 0.0, w, screen_height(), Color::from_rgba(6, 10, 20, 235));
        let h = screen_height();
        let skin = Skin::PearlSand;
        quiz_header_skinned("THE GRAVE CAVE MAZE OF CRAZE",
            "Choose the path. Wrong: silently sent back one step.", skin);
        let step = self.maze_step.min(self.maze_order.len().saturating_sub(1));
        let q = &maze::MAZE_QUESTIONS[self.maze_order[step]];
        // question card
        let qx = w * 0.5 - 390.0;
        draw_rectangle(qx, 160.0, 780.0, 96.0, Color::from_rgba(10, 26, 40, 215));
        draw_rectangle_lines(qx, 160.0, 780.0, 96.0, 1.5, skin.edge());
        draw_wrapped_rich(q.question, qx + 26.0, 202.0, 728.0, 30.0, 27,
            Color::from_rgba(242, 250, 253, 255), Color::from_rgba(140, 225, 235, 255));
        let row0 = 292.0;
        for i in 0..4 {
            draw_option_row_skinned(i, maze::DIR_NAMES[i], row0 + i as f32 * 44.0, i == self.maze_sel, None, skin);
        }
        // depth meter: how far into the cave the path has gone
        let mx = w - 52.0;
        draw_rectangle(mx, 170.0, 12.0, 300.0, Color::from_rgba(255, 255, 255, 36));
        let fill = self.maze_depth.max(6.0);
        draw_rectangle(mx, 470.0 - fill, 12.0, fill,
            Color::from_rgba(140, 225, 235, 255));
        // clue card, kept inside the framed stage
        // sits inside the framed stage, clear of the frame's bottom rule
        let cy = h - 248.0;
        draw_rectangle(qx, cy, 780.0, 110.0, Color::from_rgba(8, 20, 32, 205));
        draw_rectangle_lines(qx, cy, 780.0, 110.0, 1.0, Color::from_rgba(140, 225, 235, 70));
        draw_dlg_text("remember the voices", qx + 24.0, cy + 26.0, 22, Color::from_rgba(140, 225, 235, 230));
        draw_wrapped_rich(maze::MAZE_CLUES[1].text, qx + 24.0, cy + 54.0, 700.0, 24.0, 22,
            Color::from_rgba(214, 238, 244, 235), Color::from_rgba(140, 225, 235, 255));
        draw_wrapped_rich(maze::MAZE_CLUES[3].text, qx + 24.0, cy + 82.0, 700.0, 24.0, 22,
            Color::from_rgba(214, 238, 244, 235), Color::from_rgba(140, 225, 235, 255));
    }

    // ---------- COSMA ----------
    fn enter_stars(&mut self) {
        self.ask_play();
        self.stars.clear();
        self.star_meter = 0;
        self.lives = cosma::LIVES;
        self.phase = Phase::StarCatch;
    }

    fn up_star_catch(&mut self, dt: f32) {
        if rand::gen_range(0.0, 1.0) < dt * 2.4 {
            let dying = rand::gen_range(0.0, 1.0) < 0.28;
            self.stars.push((rand::gen_range(30.0, 994.0), -20.0, rand::gen_range(130.0, 250.0), dying));
        }
        for s in self.stars.iter_mut() {
            s.1 += s.2 * dt;
        }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { self.paddle_pos -= dt * 0.95; }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { self.paddle_pos += dt * 0.95; }
        self.paddle_pos = self.paddle_pos.clamp(0.0, 1.0);

        let px = self.paddle_x();
        let mut caught: Option<bool> = None;
        self.stars.retain(|s| {
            let near = (s.0 - px).abs() < 50.0 && s.1 > PADDLE_Y - 22.0 && s.1 < PADDLE_Y + 18.0;
            if near && caught.is_none() {
                caught = Some(!s.3);
                return false;
            }
            s.1 <= 760.0
        });
        if let Some(good) = caught {
            if good {
                self.good_play();
                self.star_meter += 1;
            } else {
                self.bad_play();
                self.lives = self.lives.saturating_sub(1);
                self.star_meter = self.star_meter.saturating_sub(cosma::DYING_PENALTY);
            }
        }
        if self.lives == 0 {
            self.cosma_fail = true;
            self.stars.clear();
            self.phase = Phase::Genie; // fail screen handled in genie draw pre-state
            return;
        }
        if self.star_meter >= cosma::STAR_TARGET {
            self.star_meter = 0;
            self.slab += 1;
            self.good_play();
            if self.slab >= cosma::SLABS {
                self.phase = Phase::Genie;
                self.world.torches_lit = cosma::SLABS;
            } else {
                self.world.set_hint(&format!("Slab {}/{} carved! Next podium.", self.slab, cosma::SLABS));
            }
        }
    }

    fn paddle_x(&self) -> f32 {
        80.0 + self.paddle_pos * 864.0
    }

    fn dr_stars(&self) {
        self.dr_world_behind();
        let w = screen_width();
        let h = screen_height();
        let p = &cosma::POEMS[self.poem];
        quiz_header_skinned("THE RAINBOW ROAD",
            &format!("Catch the lit stars, dodge the dying ones. Lives {}   Slab {}/{}",
                self.lives, self.slab + 1, cosma::SLABS), Skin::NeonSlab);

        // play band, with a marked catch line instead of a bare paddle
        let band_top = 152.0;
        let band_bot = 552.0;
        draw_rectangle(28.0, band_top, w - 56.0, band_bot - band_top, Color::from_rgba(8, 4, 20, 170));
        draw_rectangle_lines(28.0, band_top, w - 56.0, band_bot - band_top, 1.5, Color::from_rgba(110, 230, 255, 60));
        let pulse = 0.35 + 0.25 * (get_time() as f32 * 3.0).sin();
        draw_rectangle(28.0, PADDLE_Y - 12.0, w - 56.0, 1.5, Color::new(0.43, 0.90, 1.0, pulse));

        // the poem being carved, as a slab card on the right
        let cx = w - 400.0;
        let cy = band_top + 14.0;
        draw_rectangle(cx, cy, 356.0, 132.0, Color::from_rgba(16, 10, 32, 225));
        draw_rectangle_lines(cx, cy, 356.0, 132.0, 1.0, Color::from_rgba(150, 110, 255, 90));
        draw_dlg_text("the slab so far", cx + 16.0, cy + 24.0, 18, Color::from_rgba(160, 200, 255, 210));
        for (i, line) in p.lines.iter().enumerate() {
            let lit = i < self.slab as usize;
            draw_dlg_text(line, cx + 16.0, cy + 50.0 + i as f32 * 21.0, 18,
                if lit { Color::from_rgba(246, 244, 255, 255) } else { Color::from_rgba(210, 210, 235, 80) });
        }

        for s in &self.stars {
            let tex: &Texture2D = if s.3 { &self.tex.dying_star } else { &self.tex.star };
            // dying stars read as red-hot, not just a different sprite
            let tint = if s.3 { Color::from_rgba(255, 150, 150, 255) } else { WHITE };
            draw_texture_ex(tex, s.0 - 16.0, s.1 - 16.0, tint,
                DrawTextureParams { dest_size: Some(Vec2::new(32.0, 32.0)), ..Default::default() });
        }
        let px = self.paddle_x();
        draw_rectangle(px - 50.0, PADDLE_Y, 100.0, 16.0, Color::from_rgba(226, 106, 156, 255));
        draw_rectangle(px - 50.0, PADDLE_Y, 100.0, 4.0, Color::from_rgba(255, 226, 240, 255));
        draw_rectangle(px - 12.0, PADDLE_Y + 16.0, 24.0, 8.0, Color::from_rgba(140, 225, 235, 200));

        // slab meter, inside the frame
        let my = h - 146.0;
        draw_dlg_text("slab progress", 40.0, my - 10.0, 20, Color::from_rgba(200, 220, 255, 220));
        draw_rectangle(40.0, my, 360.0, 16.0, Color::from_rgba(255, 255, 255, 36));
        draw_rectangle(40.0, my, 360.0 * (self.star_meter as f32 / cosma::STAR_TARGET as f32), 16.0,
            Color::from_rgba(255, 226, 120, 255));
        draw_rectangle_lines(40.0, my, 360.0, 16.0, 1.5, Color::from_rgba(255, 255, 255, 70));
        draw_dlg_text(&format!("{}/{}", self.star_meter, cosma::STAR_TARGET), 410.0, my + 14.0, 20,
            Color::from_rgba(226, 226, 240, 220));
    }

    fn up_genie(&mut self) {
        if self.cosma_fail {
            if Self::adv_pressed() {
                self.cosma_fail = false;
                self.poem = (self.poem + 1) % cosma::POEMS.len();
                self.slab = 0;
                self.lives = cosma::LIVES;
                self.phase = Phase::StarCatch;
                self.fade = 0.0;
            }
            return;
        }
        if !self.genie_shown {
            self.genie_shown = true;
            let p = &cosma::POEMS[self.poem];
            self.dlg.say("Magic Fire Genie", &format!("The torches burn. {}", p.question), &self.tex);
        }
        if self.dlg.open {
            let _ = self.dlg.update(get_frame_time());
        } else {
            let p = &cosma::POEMS[self.poem];
            if is_key_pressed(KeyCode::Up) { self.genie_sel = self.genie_sel.saturating_sub(1); }
            if is_key_pressed(KeyCode::Down) { self.genie_sel = (self.genie_sel + 1).min(3); }
            if Self::adv_pressed() {
                if self.genie_sel == p.answer {
                    self.good_play();
                    self.progress.ingredients += 1;
                    self.reward_play();
                    self.progress.done.insert("portal");
                    self.phase = Phase::Explore;
                    self.fade = 0.0;
                    self.world.set_hint("A component hums in your hand. Reach the stone circle!");
                    self.world.burst(self.world.player.0, self.world.player.1 + 20.0, 3, 30);
                    self.autosave();
                } else {
                    self.bad_play();
                    self.cosma_fail = true;
                }
            }
        }
    }

    fn dr_genie(&self) {
        self.dr_world_behind();
        let w = screen_width();
        if self.cosma_fail {
            draw_rectangle(0.0, 0.0, w, screen_height(), Color::from_rgba(0, 0, 0, 200));
            draw_text("The stars scatter...", w * 0.5 - 110.0, 300.0, 30.0, RED);
            draw_text("z: try again (a new poem will form)", w * 0.5 - 170.0, 350.0, 22.0, GRAY);
            return;
        }
        if self.dlg.open {
            self.dlg.draw();
            return;
        }
        let p = &cosma::POEMS[self.poem];
        quiz_header_skinned("GENIE'S QUESTION", p.question, Skin::NeonSlab);
        for (i, o) in p.options.iter().enumerate() {
            draw_option_row_skinned(i, o, 300.0 + i as f32 * 44.0, i == self.genie_sel, None, Skin::NeonSlab);
        }
    }

    // ---------- KANDY BRIDGE ----------
    fn up_kandy_bridge(&mut self) {
        // The intro scene just finished: close it and ask the question.
        if !self.kandy_asked {
            self.scene_i = 0;
            self.kandy_asked = true;
            self.dlg.ask("Kandy Kween", "The trials. Be honest:", &[
                ("I thought they were great", "g"),
                ("They were really easy", "e"),
                ("Those puzzles sucked", "s"),
            ], &self.tex);
            return;
        }
        if let Some(p) = self.dlg.update(get_frame_time()) {
            if !p.is_empty() { self.last_choice = p.clone(); }
            if self.kandy_reaction.is_none() {
                let texts: &'static [script::Line] = match self.last_choice.as_str() {
                    "g" => script::KANDY_GREAT_TEXTS,
                    "e" => script::KANDY_EASY_TEXTS,
                    _ => script::KANDY_SUCKED_TEXTS,
                };
                self.kandy_reaction = Some(texts);
                self.scene_lines = texts;
                self.scene_i = 0;
                let ln = &texts[0];
                self.dlg.say(ln.who, ln.text, &self.tex);
                return;
            }
            // reaction finished: narration then trapdoor QTE
            self.play_scene(script::TRAPDOOR, Phase::Qte);
        }
    }

    // ---------- QTE ----------
    fn up_qte(&mut self, dt: f32) {
        if self.dlg.open {
            // narration after failing/resisting
            if let Some(_) = self.dlg.update(get_frame_time()) {
                self.qte_line += 1;
                if self.qte_line < script::QTE_LINES.len() {
                    let ln = &script::QTE_LINES[self.qte_line];
                    self.dlg.say(ln.who, ln.text, &self.tex);
                } else if self.qte_line < script::QTE_LINES.len() + script::DRAGGED.len() {
                    let idx = self.qte_line - script::QTE_LINES.len();
                    let ln = &script::DRAGGED[idx];
                    self.dlg.say(ln.who, ln.text, &self.tex);
                } else {
                    // the drag ends: teaser card gates Aquarius (LITERACY_FULL unlocks)
                    self.fade = 0.0;
                    self.dlg.close();
                    self.phase = Phase::NextEpisode;
                }
            }
            return;
        }
        // mash phase
        self.trap_t += dt;
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
            self.qte_progress += 0.05;
        }
        self.qte_progress -= dt * 0.35;
        if self.qte_progress >= 1.0 || (self.qte_progress <= 0.0 && self.trap_t > 3.0) {
            self.splash_play();
            self.dlg.say("(Narration)", script::QTE_LINES[0].text, &self.tex);
        }
    }

    fn dr_qte(&self) {
        let t = get_time() as f32;
        let mut y = 0.0;
        while y < screen_height() {
            let k = (y / screen_height() * 6.0 + t) % 1.0;
            let c = Color::from_rgba((160.0 * k) as u8, (60.0 * k) as u8, (120.0 * k) as u8, 255);
            draw_rectangle(0.0, y, screen_width(), 24.0, c);
            y += 24.0;
        }
        if !self.dlg.open {
            draw_text("MASH! (space / click)", screen_width() * 0.5 - 130.0, 200.0, 30.0, YELLOW);
            draw_rectangle(screen_width() * 0.5 - 200.0, 240.0, 400.0, 26.0, GRAY);
            draw_rectangle(screen_width() * 0.5 - 200.0, 240.0, 400.0 * self.qte_progress.clamp(0.0, 1.0), 26.0, PINK);
        }
        self.dlg.draw();
    }

    // ---------- CRUMBLE + ENDING ----------
    fn up_crumble(&mut self, dt: f32) {
        self.crumble_t += dt;
        if Self::adv_pressed() && self.crumble_t > 1.5 {
            // run back to an explorable home
            self.biome = Biome::Home;
            self.world = World::new(Biome::Home, &self.tex);
            self.fade = 0.0;
            self.amb("house");
            self.phase = Phase::Explore;
            self.autosave();
        }
    }

    fn dr_crumble(&self) {
        clear_background(Color::from_rgba(10, 6, 16, 255));
        let t = self.crumble_t;
        let mut seed = 99u64;
        for _ in 0..50 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = ((seed >> 33) % 1024) as f32;
            let y = (((seed >> 20) % 720) as f32 + t * 140.0) % 720.0;
            draw_rectangle(x, y, 6.0, 6.0, Color::from_rgba(255, 90, 110, 200));
        }
        // player running silhouette shakes
        let shake = (t * 30.0).sin() * 3.0;
        draw_texture_ex(&self.tex.river, screen_width() * 0.5 - 24.0 + shake, 430.0,
            WHITE, DrawTextureParams { dest_size: Some(Vec2::new(48.0, 72.0)), ..Default::default() });
        draw_text("The dimension is crumbling. RUN!", screen_width() * 0.5 - 190.0, 340.0, 30.0, RED);
        draw_text("z to run home", screen_width() * 0.5 - 60.0, 560.0, 18.0, GRAY);
    }

    fn up_ending(&mut self) {
        if !self.dlg.open && self.end_line == 0 {
            self.amb("house");
            let ln = &script::ENDING[0];
            self.dlg.say(ln.who, ln.text, &self.tex);
            return;
        }
        if self.dlg.update(get_frame_time()).is_some() {
            self.end_line += 1;
            if self.end_line < script::ENDING.len() {
                let ln = &script::ENDING[self.end_line];
                self.dlg.say(ln.who, ln.text, &self.tex);
            } else {
                // Stay on the last frame: the room, the two of them, done.
                self.dlg.close();
            }
        }
    }

    fn dr_ending(&self) {
        let w = screen_width();
        let h = screen_height();
        // Nadia's room at night, lit only by the hearth glow.
        for i in 0..40 {
            let f = i as f32 / 40.0;
            let c = Color::new(0.16 - 0.10 * f, 0.10 - 0.06 * f, 0.14 - 0.08 * f, 1.0);
            draw_rectangle(0.0, f * h, w, h / 40.0 + 1.0, c);
        }
        let glow = 0.5 + 0.5 * (get_time() as f32 * 1.4).sin();
        // hearth light: a soft, wide pool rather than a hard brown disc
        self.fx.glow_rect(w * 0.5 + 250.0, 300.0, 560.0 + glow * 40.0, 520.0,
            Color::new(1.0, 0.56, 0.26, 0.13 + glow * 0.03));
        draw_ellipse(w * 0.5 + 20.0, 492.0, 250.0, 22.0, 0.0, Color::from_rgba(120, 62, 70, 90));
        // everything has to sit above the dialogue panel (top edge 510)
        draw_rectangle(0.0, 500.0, w, h - 500.0, Color::from_rgba(48, 30, 38, 255));
        draw_rectangle(0.0, 498.0, w, 3.0, Color::from_rgba(70, 44, 52, 255));
        // Nadia's bed, then the two of them beside it
        draw_rectangle(w * 0.5 - 230.0, 392.0, 260.0, 106.0, Color::from_rgba(74, 48, 62, 255));
        draw_rectangle(w * 0.5 - 230.0, 392.0, 260.0, 14.0, Color::from_rgba(255, 176, 200, 255));
        draw_rectangle(w * 0.5 - 216.0, 408.0, 62.0, 22.0, Color::from_rgba(250, 246, 238, 255));
        draw_rectangle(w * 0.5 - 230.0, 486.0, 260.0, 12.0, Color::from_rgba(52, 32, 44, 255));
        draw_texture_ex(&self.tex.river, w * 0.5 - 40.0, 340.0, WHITE,
            DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() });
        let breathe = (get_time() as f32 * 2.0).sin() * 2.0;
        draw_texture_ex(&self.tex.nadia, w * 0.5 - 210.0, 372.0 + breathe, WHITE,
            DrawTextureParams { dest_size: Some(Vec2::new(78.0, 117.0)), ..Default::default() });
        self.dlg.draw();
    }
}

impl Game {
    fn up_title(&mut self) {
        self.amb("house");
        let w = screen_width();
        let opts = if self.save_exists { 2 } else { 1 };
        if is_key_pressed(KeyCode::Up) {
            let before = self.title_sel;
            self.title_sel = self.title_sel.saturating_sub(1);
            if self.title_sel != before { self.ui_move(); }
        }
        if is_key_pressed(KeyCode::Down) {
            let before = self.title_sel;
            self.title_sel = (self.title_sel + 1).min(opts - 1);
            if self.title_sel != before { self.ui_move(); }
        }
        if self.title_sel >= opts { self.title_sel = opts - 1; }

        // Hover picks the row under the cursor.
        let (mx, my) = mouse_position();
        let mut hover: Option<usize> = None;
        let mut y = 470.0;
        let entries: [(&str, bool); 2] = [("continue", self.save_exists), ("new game", true)];
        for (i, (label, enabled)) in entries.iter().enumerate() {
            if !*enabled { y += 56.0; continue; }
            let lw = tag_width(label, 18) + 56.0;
            let x = w * 0.5 - lw / 2.0;
            if my >= y - 6.0 && my <= y + 42.0 && mx >= x - 10.0 && mx <= x + lw + 10.0 {
                hover = Some(i);
            }
            y += 56.0;
        }
        if let Some(h) = hover {
            self.title_sel = h;
        }
        let click_fire = hover.is_some() && is_mouse_button_pressed(MouseButton::Left);
        if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) || click_fire {
            self.good_play();
            if self.save_exists && self.title_sel == 0 {
                if let Some(d) = crate::game::save::load() {
                    self.apply_save(d);
                    return;
                }
            }
            self.reset_run();
        }
    }

    fn dr_title(&self) {
        let w = screen_width();
        let h = screen_height();
        // deep plum gradient sky
        let top = Color::from_rgba(74, 42, 99, 255);
        let bot = Color::from_rgba(26, 14, 43, 255);
        for i in 0..48 {
            let f = i as f32 / 48.0;
            let c = Color::new(top.r + (bot.r - top.r) * f, top.g + (bot.g - top.g) * f, top.b + (bot.b - top.b) * f, 1.0);
            draw_rectangle(0.0, i as f32 * h / 48.0, w, h / 48.0 + 1.0, c);
        }
        // drifting pink sparkles
        let t = get_time() as f32;
        let mut seed = 12u64;
        for _ in 0..46 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let x = ((seed >> 33) % 1024) as f32;
            let y = (((seed >> 20) % 720) as f32 + t * (14.0 + (seed % 12) as f32)) % h;
            let tw = 0.4 + 0.6 * (t * 2.0 + (seed % 8) as f32).sin().abs();
            draw_rectangle(x, y, 2.0, 2.0, Color::new(1.0, 0.72, 0.84, tw * 0.8));
        }
        // heirloom book glowing behind the title block
        let glow = 0.5 + 0.5 * (get_time() as f32 * 1.6).sin();
        draw_circle(w * 0.5, 112.0, 104.0 + glow * 14.0, Color::from_rgba(255, 224, 140, (26.0 + glow * 34.0) as u8));
        // the heirloom itself is the title mark
        draw_storybook(w * 0.5, 178.0, 2.9, glow);
        // title in Press Start, cream on plum; hard-capped so it can never
        // overflow on backends whose glyph rendering outgrows the metrics
        let title = "GIFT OF EDUCATION";
        let ts: u16 = 26;
        let tw = tag_width(title, ts);
        draw_tag(title, w * 0.5 - tw / 2.0, 244.0, ts, Color::from_rgba(255, 246, 233, 255));
        draw_rectangle(w * 0.5 - tw * 0.25, 272.0, tw * 0.5, 3.0, Color::from_rgba(226, 106, 156, 255));
        let sub = "a literacy rpg";
        let sw = dlg_width(sub, 30);
        draw_dlg_text(sub, w * 0.5 - sw / 2.0, 296.0, 30, Color::from_rgba(255, 214, 230, 220));
        // characters standing either side of the menu, with contact shadows so
        // they sit on the same floor as the menu
        for (tx, cx2) in [(&self.tex.nadia, w * 0.5 - 380.0), (&self.tex.river, w * 0.5 + 290.0)] {
            draw_ellipse(cx2 + 45.0, 573.0, 30.0, 6.0, 0.0, Color::from_rgba(16, 8, 26, 90));
            draw_texture_ex(tx, cx2, 436.0, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(90.0, 135.0)), ..Default::default() });
        }
        // menu chips
        let items: [(&str, bool); 2] = [("continue", self.save_exists), ("new game", true)];
        let mut y = 470.0;
        for (i, (label, enabled)) in items.iter().enumerate() {
            if !enabled { y += 56.0; continue; }
            let sel = i == self.title_sel;
            let lw = tag_width(label, 18) + 56.0;
            let x = w * 0.5 - lw / 2.0;
            if sel {
                draw_rectangle(x, y, lw, 36.0, Color::from_rgba(226, 106, 156, 255));
                draw_rectangle_lines(x - 2.0, y - 2.0, lw + 4.0, 40.0, 2.0, Color::from_rgba(255, 246, 233, 255));
                draw_tag(label, w * 0.5 - tag_width(label, 18) / 2.0, y + 24.0, 18, Color::from_rgba(46, 26, 71, 255));
            } else {
                draw_tag(label, w * 0.5 - tag_width(label, 18) / 2.0, y + 24.0, 18, Color::from_rgba(200, 186, 214, 255));
            }
            y += 56.0;
        }
        let ctrl = "wasd move  z confirm  1-4 pick  f1 debug";
        let cw2 = dlg_width(ctrl, 24);
        draw_dlg_text(ctrl, w * 0.5 - cw2 / 2.0, h - 34.0, 24, Color::from_rgba(200, 186, 214, 150));
    }    fn page_play(&mut self) {
        play_sound(&self.audio.sfx.page, PlaySoundParams { looped: false, volume: 0.5 });
    }

    /// Right answer: bright flash, a small punch-in and a few frames of
    /// hit-stop, so the confirmation reads as a hit rather than a label swap.
    fn good_play(&mut self) {
        play_sound(&self.audio.sfx.good, PlaySoundParams { looped: false, volume: 0.5 });
        self.fx.accent(true);
    }

    /// Wrong answer: red flash, camera kick, longer hit-stop.
    fn bad_play(&mut self) {
        play_sound(&self.audio.sfx.bad, PlaySoundParams { looped: false, volume: 0.5 });
        self.fx.accent(false);
    }

    /// Footsteps. The surface follows the biome, and the two variants alternate
    /// with a small volume jitter so a walk does not read as one looped sample.
    fn step_play(&mut self) {
        let n = self.audio.sfx.steps.len();
        if n < 8 {
            return;
        }
        // riding swaps the whole set for a harder, woodier contact
        let base = match (self.biome, crate::game::ride_unlocked()) {
            (Biome::Candy, true) => 6,
            (Biome::Candy, false) => 0,
            (Biome::Aquarius, _) => 2,
            (Biome::Cosma, _) => 4,
            (Biome::Home, _) => 6,
        };
        let idx = (base + (rand::gen_range(0.0, 2.0) as usize)) % n;
        let vol = if crate::game::ride_unlocked() && self.biome == Biome::Candy {
            0.30 + rand::gen_range(0.0, 0.08)
        } else {
            0.22 + rand::gen_range(0.0, 0.09)
        };
        play_sound(&self.audio.sfx.steps[idx], PlaySoundParams { looped: false, volume: vol });
    }

    /// A generic UI blip for prompt advances and menu moves.
    fn ui_click(&mut self) {
        play_sound(&self.audio.sfx.click, PlaySoundParams { looped: false, volume: 0.35 });
        self.fx.punch(0.006);
    }

    fn ui_move(&mut self) {
        play_sound(&self.audio.sfx.select, PlaySoundParams { looped: false, volume: 0.28 });
    }

    fn ui_back(&mut self) {
        play_sound(&self.audio.sfx.back, PlaySoundParams { looped: false, volume: 0.30 });
    }

    /// Awarded when an elixir ingredient is banked.
    fn reward_play(&mut self) {
        play_sound(&self.audio.sfx.coin, PlaySoundParams { looped: false, volume: 0.45 });
    }

    /// A question is about to be asked.
    fn ask_play(&mut self) {
        play_sound(&self.audio.sfx.question, PlaySoundParams { looped: false, volume: 0.38 });
    }

    /// The trapdoor plunge: a full camera dunk in water.
    fn splash_play(&mut self) {
        play_sound(&self.audio.sfx.splash, PlaySoundParams { looped: false, volume: 0.5 });
        self.fx.shake_for(11.0, 0.5);
        self.fx.punch(0.05);
        self.fx.hit_stop(0.16);
        self.fx.flash(Color::new(0.62, 0.86, 1.0, 1.0), 0.34);
    }
}

impl Game {
    /// Enter a biome as a scripted act: swap the overworld and play its
    /// arrival lines before handing control back to the player.
    fn enter_biome_act(&mut self, biome: Biome) {
        self.boot_biome(biome);
        match biome {
            Biome::Aquarius => self.play_scene(script::AQUA_WAKE, Phase::Explore),
            Biome::Cosma => self.play_scene(script::COSMA_ARRIVE, Phase::Explore),
            _ => self.phase = Phase::Explore,
        }
    }

    /// The act card is the only gap between the plunge and Aquarius: diving is
    /// automatic, the whole script is playable in one run.
    fn up_next(&mut self) {
        if Self::adv_pressed() {
            self.enter_biome_act(Biome::Aquarius);
        }
    }

    /// Aquarius to Cosma: swap the overworld the moment the rift scene ends.
    fn up_act_change(&mut self) {
        let b = self.next_biome;
        self.enter_biome_act(b);
    }

    /// Aquarius teaser card after the trapdoor plunge.
    fn dr_next(&self) {
        let w = screen_width();
        let h = screen_height();
        let t = get_time() as f32;
        // deep sea gradient + drifting light rays
        for i in 0..48 {
            let f = i as f32 / 48.0;
            let c = Color::new(0.07 + 0.2 * f, 0.28 + 0.12 * f, 0.42 + 0.06 * f, 1.0);
            draw_rectangle(0.0, i as f32 * h / 48.0, w, h / 48.0 + 1.0, c);
        }
        for i in 0..4 {
            let x = 140.0 + i as f32 * 260.0 + (t * 0.5 + i as f32).sin() * 30.0;
            draw_rectangle(x, 0.0, 70.0, h, Color::new(0.7, 0.86, 1.0, 0.05));
        }
        for i in 0..22 {
            let x = (i as f32 * 137.0 + t * 26.0) % (w + 40.0) - 20.0;
            let y = h - ((t * 40.0 + i as f32 * 60.0) % h);
            draw_circle_lines(x, y, 2.0 + (i % 3) as f32, 1.2, Color::new(0.7, 0.9, 1.0, 0.4));
        }
        // teaser text
        let head = "AQUARIUS";
        let hw = tag_width(head, 46);
        draw_tag(head, w * 0.5 - hw / 2.0, 210.0, 46, Color::from_rgba(214, 240, 255, 255));
        let sub = "the grave cave maze of craze";
        let sw = dlg_width(sub, 34);
        draw_dlg_text(sub, w * 0.5 - sw / 2.0, 300.0, 34, Color::from_rgba(150, 210, 235, 255));
        draw_rectangle(w * 0.5 - 160.0, 344.0, 320.0, 2.0, Color::from_rgba(120, 190, 220, 120));
        let body = "river sinks into a world under the skywater.";
        let bw = dlg_width(body, 30);
        draw_dlg_text(body, w * 0.5 - bw / 2.0, 366.0, 30, Color::from_rgba(180, 214, 235, 220));
        let hint = "z: dive into aquarius";
        let hintw = dlg_width(hint, 26);
        draw_dlg_text(hint, w * 0.5 - hintw / 2.0, 470.0, 26, Color::from_rgba(214, 240, 255, 200));
        // river sinking at the bottom edge
        let bob = (t * 2.0).sin() * 8.0;
        draw_texture_ex(&self.tex.river, w * 0.5 - 51.0, h - 150.0 + bob, WHITE,
            DrawTextureParams { dest_size: Some(Vec2::new(102.0, 153.0)), ..Default::default() });
    }

    /// Boot a specific screen for QA screenshots (`--shot=<name>`).
    /// Readable pose of the player for the movement and animation tests.
    pub fn qa_pose(&self) -> String {
        format!(
            "x={:7.1} y={:6.1} feet={:6.1} drawn_bottom={:6.1} facing={:+.0} moving={} swim={}",
            self.world.player.0,
            self.world.player.1,
            self.world.player.1 + 72.0,
            self.world.drawn_bottom.get(),
            self.world.facing,
            self.world.moving,
            self.biome == Biome::Aquarius,
        )
    }

    /// Test hooks: hold a direction, teleport, or set a click-to-walk target.
    pub fn qa_dir(&mut self, dx: f32, dy: f32) {
        self.world.bot_dir = dx;
        self.world.bot_y = dy;
    }
    pub fn qa_teleport(&mut self, x: f32) {
        self.world.player.0 = x;
        self.world.click = None;
    }
    pub fn qa_click(&mut self, x: f32) {
        self.world.click = Some(x);
    }
    pub fn qa_player(&self) -> (f32, f32) {
        self.world.player
    }
    pub fn qa_world_len(&self) -> f32 {
        self.world.length
    }
    pub fn qa_click_target(&self) -> Option<f32> {
        self.world.click
    }
    pub fn qa_biome_name(&self) -> &'static str {
        match self.biome {
            Biome::Candy => "candy",
            Biome::Aquarius => "aquarius",
            Biome::Cosma => "cosma",
            Biome::Home => "home",
        }
    }

    pub fn qa_boot(&mut self, name: &str) {
        match name {
            "walk" => {
                self.boot_candy();
                self.world.player.0 = 400.0;
                self.world.bot_dir = 1.0;
            }
            "idle" => {
                self.boot_candy();
                self.world.player.0 = 400.0;
            }
            "swim" => {
                self.boot_aqua();
                self.world.player.0 = 1200.0;
                self.world.bot_dir = 1.0;
            }
            "title" => self.phase = Phase::Title,
            "intro" => {
                self.phase = Phase::Scene;
                self.scene_lines = script::INTRO_SCENE;
                self.scene_next = Phase::Explore;
                self.intro_i = 2;
            }
            "gift" => {
                self.phase = Phase::Scene;
                self.scene_lines = script::INTRO_SCENE;
                self.scene_next = Phase::Explore;
                self.intro_i = 4;
            }
            "candy" | "candy_world" => self.boot_candy(),
            "jin_meet" => {
                self.boot_candy();
                self.world.player.0 = 640.0;
                self.world.cam = self.world.target_cam();
                self.jin_step = 1;
                self.dlg.ask("Jin", script::JIN_INTRO[2].text, &[
                    ("Where am I", "w"),
                    ("Where are we", "w"),
                    ("No", "no"),
                ], &self.tex);
                self.world.set_hint("Head right: riddle stones, then the Throne Gate.");
                self.phase = Phase::JinMeet;
            }
            "jin_battle" => {
                self.boot_candy();
                self.jin_qs = (0..jin_battle::FULL_SET.len()).collect();
                self.jin_i = 0;
                self.jin_correct = 0;
                self.phase = Phase::JinBattle;
            }
            "riddle" => {
                self.boot_candy();
                self.enter_riddle();
            }
            "gate" => {
                self.boot_candy();
                self.progress.candy_riddles_done = 2;
                self.enter_gate();
            }
            "guards" => {
                self.boot_candy();
                self.world.player.0 = 2380.0;
                self.world.cam = self.world.target_cam();
                self.play_scene(script::GUARDS, Phase::Explore);
            }
            "kandy" => {
                self.boot_candy();
                self.world.player.0 = 3150.0;
                self.world.cam = self.world.target_cam();
                self.play_scene(script::KANDY_INTRO_SCENE, Phase::KandyBridge);
            }
            "qte" => {
                self.boot_candy();
                self.phase = Phase::Qte;
                self.qte_progress = 0.45;
            }
            "actcard" => {
                self.boot_candy();
                self.phase = Phase::NextEpisode;
            }
            "aqua" | "aqua_world" => self.boot_aqua(),
            "aqua_wake" => {
                self.boot_biome(Biome::Aquarius);
                self.play_scene(script::AQUA_WAKE, Phase::Explore);
            }
            "maze" => {
                self.boot_aqua();
                self.progress.aqua_riddles_done = 2;
                self.enter_maze();
            }
            "cosma" | "cosma_world" => self.boot_cosma(),
            "stars" => {
                self.boot_cosma();
                self.progress.cosma_riddles_done = 2;
                self.enter_stars();
            }
            "genie" => {
                self.boot_cosma();
                self.world.torches_lit = cosma::SLABS;
                self.phase = Phase::Genie;
            }
            "crumble" => {
                self.boot_cosma();
                self.phase = Phase::Crumble;
                self.crumble_t = 1.0;
            }
            "home" => self.boot_biome(Biome::Home),
            "ending" => {
                self.boot_biome(Biome::Home);
                self.phase = Phase::Ending;
                self.end_line = 0;
            }
            other => {
                eprintln!("unknown --shot={} (title,intro,candy,jin_meet,jin_battle,riddle,gate,guards,kandy,qte,actcard,aqua,maze,cosma,stars,genie,crumble,home,ending)", other);
                self.phase = Phase::Title;
            }
        }
    }

    fn phase_name(&self) -> &'static str {
        match self.phase {
            Phase::Title => "title",
            Phase::Explore => "explore",
            Phase::Scene => "scene",
            Phase::JinMeet => "jin meet",
            Phase::JinBattle => "jin battle",
            Phase::JinAfter => "jin after",
            Phase::Riddle => "riddle",
            Phase::KandyBridge => "kandy bridge",
            Phase::GatePuzzle => "throne gate",
            Phase::MazePuzzle => "cave maze",
            Phase::StarCatch => "rainbow road",
            Phase::Genie => "genie",
            Phase::Qte => "qte plunge",
            Phase::NextEpisode => "act card",
            Phase::ActChange => "act change",
            Phase::Crumble => "crumble",
            Phase::Ending => "ending",
        }
    }

    fn dr_debug(&self) {
        let w = screen_width();
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("phase: {}", self.phase_name()));
        lines.push(format!("biome: {}", match self.biome {
            Biome::Candy => "candy", Biome::Aquarius => "aquarius",
            Biome::Cosma => "cosma", Biome::Home => "home",
        }));
        if self.phase == Phase::Explore || self.phase == Phase::Scene {
            lines.push(format!("pos: {:.0} / {:.0}", self.world.player.0, self.world.length));
        }
        let goal = if self.dlg.open {
            format!("speaking: {}", self.dlg_speaker_name())
        } else if self.phase == Phase::Explore {
            match self.world.nearest_info() {
                Some((l, _, _)) => format!("near: {}", l),
                None => format!("go: x{:.0}", self.world.next_target_x()),
            }
        } else {
            String::new()
        };
        if !goal.is_empty() {
            lines.push(goal);
        }
        lines.push(format!(
            "candy riddles {}/2  gate {}  ing {}/3  horse {}",
            self.progress.candy_riddles_done,
            if self.progress.gate_solved { "open" } else { "locked" },
            self.progress.ingredients,
            if self.progress.has_horse { "yes" } else { "no" },
        ));
        lines.push("f1: hide".to_string());
        let lh = 20.0;
        let box_w = 340.0;
        let box_h = lines.len() as f32 * lh + 16.0;
        let x = w - box_w - 12.0;
        let y = 12.0;
        draw_rectangle(x, y, box_w, box_h, Color::new(0.06, 0.05, 0.1, 0.85));
        draw_rectangle_lines(x, y, box_w, box_h, 2.0, Color::from_rgba(226, 106, 156, 160));
        let mut ly = y + 20.0;
        for ln in &lines {
            draw_dlg_text(ln, x + 12.0, ly, 20, Color::from_rgba(255, 246, 233, 235));
            ly += lh;
        }
    }

    fn dlg_speaker_name(&self) -> &str {
        self.dlg.speaker()
    }
}
