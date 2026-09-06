//! Scene state machine. Overworld-first: walk to NPCs and props, Z to engage.

use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};
use crate::art::{Textures, draw_biome_bg};
use crate::audio::AudioSys;
use crate::game::script::{self, Biome};
use crate::game::ui::{DialogueBox, draw_wrapped, draw_option_row, quiz_header};
use crate::game::world::World;
use crate::game::{Progress, jin_battle, gate, maze, cosma};

#[derive(PartialEq, Clone, Copy)]
pub enum Phase {
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
    Crumble,
    Ending,
}

pub struct Game {
    phase: Phase,
    biome: Biome,
    progress: Progress,
    tex: Textures,
    audio: AudioSys,
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
    fade: f32,
}

const AMB_VOL: f32 = 0.12;

impl Game {
    pub async fn new() -> Self {
        let tex = Textures::load();
        let audio = crate::audio::load().await;
        Self {
            phase: Phase::Scene,
            biome: Biome::Candy,
            progress: Progress::new(),
            tex,
            audio,
            dlg: DialogueBox::new(),
            world: World::new(Biome::Candy),
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
            fade: 1.0,
        }
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

    fn adv_pressed() -> bool {
        is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter)
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

    pub fn update(&mut self, dt: f32) {
        self.fade = (self.fade + dt * 2.0).min(1.0);
        match self.phase {
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
            Phase::Crumble => self.up_crumble(dt),
            Phase::Ending => self.up_ending(),
        }
    }

    pub fn draw(&self) {
        match self.phase {
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
            Phase::Crumble => self.dr_crumble(),
            Phase::Ending => self.dr_ending(),
        }
        // fade-in veil for scene transitions
        if self.fade < 1.0 {
            let a = (1.0 - self.fade) * 0.9;
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, a));
        }
    }

    fn dr_world(&self) {
        draw_biome_bg(self.biome, get_time() as f32);
        self.world.draw(&self.tex, get_time() as f32);
        self.dlg.draw();
    }

    fn dr_world_behind(&self) {
        draw_biome_bg(self.biome, get_time() as f32);
        self.world.draw(&self.tex, get_time() as f32);
    }

    fn draw_hud(&self) {
        // ingredient counter
        let msg = format!("Elixir components: {}/3", self.progress.ingredients);
        draw_rectangle(14.0, 12.0, measure_text(&msg, None, 16, 1.0).width + 20.0, 26.0,
            Color::from_rgba(0, 0, 0, 150));
        draw_text(&msg, 24.0, 30.0, 16.0, Color::from_rgba(255, 230, 120, 255));
        if self.progress.has_horse {
            draw_text("horse: yes", 24.0, 56.0, 14.0, Color::from_rgba(200, 255, 200, 200));
        }
    }

    fn dr_intro(&self) {
        let w = screen_width();
        let h = screen_height();
        if self.intro_i < script::INTRO_LINES.len() {
            clear_background(Color::from_rgba(24, 16, 20, 255));
            match self.intro_i {
                0..=1 => draw_texture_ex(&self.tex.nadia, w * 0.5 - 48.0, h * 0.32, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() }),
                2..=3 => draw_texture_ex(&self.tex.grandparent, w * 0.5 - 48.0, h * 0.30, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() }),
                _ => {
                    let glow = 0.5 + 0.5 * (get_time() as f32 * 2.0).sin();
                    draw_circle(w * 0.5, h * 0.42, 60.0 + glow * 14.0,
                        Color::from_rgba(255, 240, 180, (40.0 + glow * 40.0) as u8));
                    draw_rectangle(w * 0.5 - 34.0, h * 0.42 - 44.0, 68.0, 88.0,
                        Color::from_rgba(140, 40, 40, 255));
                    draw_rectangle(w * 0.5 - 4.0, h * 0.42 - 44.0, 8.0, 88.0,
                        Color::from_rgba(90, 20, 20, 255));
                }
            }
            draw_wrapped(script::INTRO_LINES[self.intro_i], w * 0.5 - 300.0, h * 0.72, 600.0, 30.0);
        } else {
            let a = (1.0 - self.intro_t).clamp(0.0, 1.0);
            clear_background(Color::new(1.0, 1.0, 1.0, a));
        }
        if (get_time() * 2.0) as i32 % 2 == 0 {
            draw_text("z / click", w - 130.0, h - 24.0, 16.0, Color::from_rgba(255, 200, 220, 160));
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
                    self.world = World::new(Biome::Candy);
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
                    _ => self.phase = next,
                }
            }
        }
    }

    // ---------- EXPLORE ----------
    fn up_explore(&mut self) {
        self.world.update(get_frame_time(), false);
        if !Self::adv_pressed() {
            return;
        }
        let Some((dist, action)) = self.world.nearest() else { return };
        let _ = dist;
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
                let label = self.world.nearest_label().unwrap_or("");
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
            Biome::Cosma => 2,
        };
        self.riddle_sel = 0;
        self.riddle_answered = false;
        self.riddle_right = None;
        self.phase = Phase::Riddle;
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
            if is_key_pressed(KeyCode::Up) { self.riddle_sel = self.riddle_sel.saturating_sub(1); }
            if is_key_pressed(KeyCode::Down) { self.riddle_sel = (self.riddle_sel + 1).min(3); }
            for i in 0..4 {
                let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, _ => KeyCode::Key4 };
                if is_key_pressed(k) {
                    self.riddle_sel = i;
                    self.submit_riddle(r.answer);
                }
            }
            if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) {
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
        if self.riddle_right == Some(true) { self.good_play(); } else { self.bad_play(); }
    }

    fn dr_riddle(&self) {
        self.dr_world_behind();
        let set = self.riddle_set_ref();
        let idx = self.riddles_done() as usize % set.len();
        let r = &set[idx];
        let title = match self.riddle_set { 0 => "CANDY LAND RIDDLE", 1 => "AQUARIUS RIDDLE", _ => "COSMA RIDDLE" };
        quiz_header(title, r.question);
        let w = screen_width();
        for (i, o) in r.options.iter().enumerate() {
            draw_option_row(i, o, 250.0 + i as f32 * 44.0,
                i == self.riddle_sel,
                if self.riddle_answered { Some(i != r.answer) } else { None });
        }
        if self.riddle_answered {
            let right = self.riddle_right == Some(true);
            draw_text(
                if right { "Correct! z to return" } else { "Wrong. One try only. z restarts this stone's riddles" },
                w * 0.5 - 180.0, 470.0, 24.0,
                if right { GREEN } else { RED });
        }
    }

    // ---------- JIN ----------
    fn up_jin_meet(&mut self) {
        if let Some(_) = self.dlg.update(get_frame_time()) {
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
        if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) {
            self.jin_submit(q.correct);
        }
    }

    fn jin_submit(&mut self, correct: usize) {
        if self.jin_sel as usize == correct {
            self.good_play();
            self.jin_correct += 1;
        } else {
            self.bad_play();
        }
        self.jin_i += 1;
        self.jin_sel = 0;
    }

    fn dr_jin_battle(&self) {
        self.dr_world_behind();
        let w = screen_width();
        if self.jin_done {
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
                self.dlg.say("Jin", script::JIN_PERFECT[0].text, &self.tex);
            } else {
                self.dlg.say("Jin", script::JIN_LOST_ONE[0].text, &self.tex);
            }
        }
        if let Some(_) = self.dlg.update(get_frame_time()) {
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

        if is_key_pressed(KeyCode::Up) { self.gate_sel = self.gate_sel.saturating_sub(1); }
        if is_key_pressed(KeyCode::Down) { self.gate_sel = (self.gate_sel + 1).min(4); }
        for i in 0..5 {
            let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, 3 => KeyCode::Key4, _ => KeyCode::Key5 };
            if is_key_pressed(k) { self.gate_sel = i; }
        }
        if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) {
            let chosen = self.gate_options[self.gate_sel.min(4)];
            self.gate_marks[self.gate_slot] = Some(chosen);
            if chosen == slot.answer {
                self.good_play();
                self.gate_slot += 1;
                self.gate_sel = 0;
                if self.gate_slot >= gate::THRONE_GATE.slots.len() {
                    self.progress.gate_solved = true;
                    self.progress.ingredients += 1;
                    self.progress.done.insert("gate");
                    self.world.set_hint("The Throne Gate grinds open!");
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

    fn dr_gate(&self) {
        self.dr_world_behind();
        let w = screen_width();
        draw_rectangle(w * 0.5 - 260.0, 90.0, 520.0, 460.0, Color::from_rgba(90, 80, 95, 235));
        draw_rectangle_lines(w * 0.5 - 260.0, 90.0, 520.0, 460.0, 6.0, Color::from_rgba(60, 52, 66, 255));
        quiz_header("THE LONE STONE STORY OF THRONE GATE",
            "Fill every box. z/Enter to set the mark. Tries allowed.");

        let words: Vec<&str> = gate::THRONE_GATE.passage.split(' ').collect();
        let mut line = String::new();
        let mut y = 170.0;
        let mut word_i = 0usize;
        for (si, slot) in gate::THRONE_GATE.slots.iter().enumerate() {
            while word_i <= slot.after_word && word_i < words.len() {
                let candidate = format!("{} {}", line, words[word_i]);
                if measure_text(&candidate, None, 17, 1.0).width > 470.0 {
                    draw_text(&line, w * 0.5 - 240.0, y, 17.0, WHITE);
                    y += 23.0;
                    line = words[word_i].to_string();
                } else {
                    line = if line.is_empty() { words[word_i].to_string() } else { candidate };
                }
                word_i += 1;
            }
            let mark = self.gate_marks.get(si).copied().flatten().unwrap_or("?");
            let hot = si == self.gate_slot;
            let seg = format!(" [{}]", mark);
            let hot_c = if hot { YELLOW } else { WHITE };
            // draw the bracketed mark inline-ish: simpler to append colored per line end
            let candidate = format!("{}{}", line, seg);
            if measure_text(&candidate, None, 17, 1.0).width > 470.0 {
                draw_text(&line, w * 0.5 - 240.0, y, 17.0, WHITE);
                y += 23.0;
                draw_text(&seg, w * 0.5 - 240.0, y, 17.0, hot_c);
                line = String::new();
                y += 23.0;
            } else {
                line = candidate;
                // draw so far with the mark colored: approximation: draw base white then mark on top
                // keep simple: store pending and render at line end
            }
            let _ = hot_c;
        }
        while word_i < words.len() {
            let candidate = format!("{} {}", line, words[word_i]);
            if measure_text(&candidate, None, 17, 1.0).width > 470.0 {
                draw_text(&line, w * 0.5 - 240.0, y, 17.0, WHITE);
                y += 23.0;
                line = words[word_i].to_string();
            } else {
                line = if line.is_empty() { words[word_i].to_string() } else { candidate };
            }
            word_i += 1;
        }
        draw_text(&line, w * 0.5 - 240.0, y, 17.0, WHITE);

        if self.gate_slot < gate::THRONE_GATE.slots.len() {
            for (i, o) in self.gate_options.iter().enumerate() {
                draw_option_row(i, o, 565.0 + i as f32 * 26.0, i == self.gate_sel, None);
            }
        }
        if let Some((msg, _)) = &self.gate_msg {
            draw_text(msg, w * 0.5 - 160.0, 70.0, 22.0, YELLOW);
        }
        // progress chips
        let done = self.gate_marks.iter().filter(|m| m.is_some()).count();
        draw_text(&format!("{}/{} marks", done, gate::THRONE_GATE.slots.len()), w - 150.0, 40.0, 18.0, WHITE);
    }

    // ---------- MAZE ----------
    fn enter_maze(&mut self) {
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
            self.progress.done.insert("maze");
            self.world.set_hint("The cave opens to the far current!");
            self.phase = Phase::Explore;
            self.fade = 0.0;
            self.world.props.retain(|p| p.id != "cave");
            return;
        }
        let q = &maze::MAZE_QUESTIONS[self.maze_order[self.maze_step]];
        if is_key_pressed(KeyCode::Up) { self.maze_sel = (self.maze_sel + 3) % 4; }
        if is_key_pressed(KeyCode::Down) { self.maze_sel = (self.maze_sel + 1) % 4; }
        for i in 0..4 {
            let k = match i { 0 => KeyCode::Key1, 1 => KeyCode::Key2, 2 => KeyCode::Key3, _ => KeyCode::Key4 };
            if is_key_pressed(k) { self.maze_sel = i; }
        }
        if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) {
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
        quiz_header("THE GRAVE CAVE MAZE OF CRAZE", "Choose the path. Wrong: you are silently sent back.");
        let step = self.maze_step.min(self.maze_order.len().saturating_sub(1));
        let q = &maze::MAZE_QUESTIONS[self.maze_order[step]];
        draw_wrapped(q.question, w * 0.5 - 300.0, 220.0, 600.0, 28.0);
        for i in 0..4 {
            draw_option_row(i, maze::DIR_NAMES[i], 340.0 + i as f32 * 44.0, i == self.maze_sel, None);
        }
        draw_rectangle(w - 44.0, 200.0, 14.0, 300.0, Color::from_rgba(255, 255, 255, 40));
        draw_rectangle(w - 44.0, 500.0 - self.maze_depth, 14.0, self.maze_depth.max(6.0), PINK);
        // clue reminders (from NPCs)
        draw_text("Remember the voices:", 40.0, 590.0, 18.0, Color::from_rgba(180, 220, 255, 220));
        draw_wrapped(maze::MAZE_CLUES[1].text, 40.0, 615.0, 420.0, 22.0);
        draw_wrapped(maze::MAZE_CLUES[3].text, 40.0, 665.0, 420.0, 22.0);
    }

    // ---------- COSMA ----------
    fn enter_stars(&mut self) {
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
            let near = (s.0 - px).abs() < 50.0 && s.1 > 545.0 && s.1 < 585.0;
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
        60.0 + self.paddle_pos * 904.0
    }

    fn dr_stars(&self) {
        self.dr_world_behind();
        let _w = screen_width();
        let p = &cosma::POEMS[self.poem];
        quiz_header("THE RAINBOW ROAD",
            &format!("Catch stars, avoid dying stars. Lives: {}. Slab {}/{}",
                self.lives, self.slab + 1, cosma::SLABS));
        for (i, line) in p.lines.iter().enumerate() {
            let c = if i < self.slab as usize { WHITE } else { Color::from_rgba(255, 255, 255, 70) };
            draw_text(line, 40.0, 160.0 + i as f32 * 24.0, 18.0, c);
        }
        for s in &self.stars {
            let tex: &Texture2D = if s.3 { &self.tex.dying_star } else { &self.tex.star };
            draw_texture_ex(tex, s.0 - 16.0, s.1 - 16.0, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(32.0, 32.0)), ..Default::default() });
        }
        let px = self.paddle_x();
        draw_rectangle(px - 46.0, 570.0, 92.0, 14.0, PINK);
        draw_rectangle(40.0, 640.0, 400.0, 20.0, GRAY);
        draw_rectangle(40.0, 640.0, 400.0 * (self.star_meter as f32 / cosma::STAR_TARGET as f32), 20.0, YELLOW);
    }

    fn up_genie(&mut self) {
        if self.cosma_fail {
            if is_key_pressed(KeyCode::Z) {
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
            if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) {
                if self.genie_sel == p.answer {
                    self.good_play();
                    self.progress.ingredients += 1;
                    self.progress.done.insert("portal");
                    self.phase = Phase::Explore;
                    self.fade = 0.0;
                    self.world.set_hint("A component hums in your hand. Reach the stone circle!");
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
        quiz_header("GENIE'S QUESTION", p.question);
        for (i, o) in p.options.iter().enumerate() {
            draw_option_row(i, o, 300.0 + i as f32 * 44.0, i == self.genie_sel, None);
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
            let _ = p;
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
                    // arrive in Aquarius
                    self.biome = Biome::Aquarius;
                    self.world = World::new(Biome::Aquarius);
                    self.fade = 0.0;
                    self.amb("aqua");
                    self.dlg.close();
                    self.phase = Phase::Explore;
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
            self.phase = Phase::Ending;
            self.fade = 0.0;
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
            self.dlg.say("Nadia", "...River? I... I can read it. I can read the whole thing!", &self.tex);
        }
        if let Some(_) = self.dlg.update(get_frame_time()) {
            self.end_line += 1;
            match self.end_line {
                1 => self.dlg.say("River", "Welcome back, Nadia.", &self.tex),
                _ => self.dlg.close(),
            }
        }
    }

    fn dr_ending(&self) {
        clear_background(Color::from_rgba(24, 16, 20, 255));
        draw_texture_ex(&self.tex.nadia, screen_width() * 0.5 - 140.0, 160.0,
            WHITE, DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() });
        draw_texture_ex(&self.tex.river, screen_width() * 0.5 + 44.0, 160.0,
            WHITE, DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() });
        draw_text("yehey", screen_width() * 0.5 - 32.0, 380.0, 30.0, PINK);
        self.dlg.draw();
    }
}

impl Game {
    fn page_play(&self) {
        play_sound(&self.audio.sfx.page, PlaySoundParams { looped: false, volume: 0.5 });
    }
    fn good_play(&self) {
        play_sound(&self.audio.sfx.good, PlaySoundParams { looped: false, volume: 0.5 });
    }
    fn bad_play(&self) {
        play_sound(&self.audio.sfx.bad, PlaySoundParams { looped: false, volume: 0.5 });
    }
    fn splash_play(&self) {
        play_sound(&self.audio.sfx.splash, PlaySoundParams { looped: false, volume: 0.5 });
    }
}
