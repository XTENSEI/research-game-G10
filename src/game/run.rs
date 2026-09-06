//! Scene state machine. Phases branch on Progress; every script beat present.

use macroquad::prelude::*;
use macroquad::audio::{play_sound, PlaySoundParams};
use crate::art::{Textures, draw_biome_bg};
use crate::audio::AudioSys;
use crate::game::script::{self, Biome};
use crate::game::ui::{DialogueBox, draw_wrapped, draw_option_row, quiz_header};
use crate::game::{Progress, jin_battle, gate, maze, cosma};

#[derive(PartialEq, Clone, Copy)]
pub enum Phase {
    Intro,
    Wake,
    JinMeet,
    JinBattle,
    JinAfter,
    CandyPuzzles,
    Guards,
    Kandy,
    Trapdoor,
    Aquarius,
    Cosma,
    Crumble,
    Ending,
}

pub struct Game {
    phase: Phase,
    progress: Progress,
    tex: Textures,
    audio: AudioSys,
    dlg: DialogueBox,

    // generic sequence walkers
    intro_i: usize,
    intro_t: f32,
    wake_stage: usize,
    jin_step: usize,
    after_stage: usize,
    last_choice: String,
    guard_line: usize,
    kandy_line: usize,
    kandy_asked: bool,
    kandy_reaction: Option<&'static [script::Line]>,
    qte_line: usize,
    trap_line: usize,
    trap_t: f32,
    qte_active: bool,
    qte_progress: f32,
    end_line: usize,
    crumble_t: f32,

    // jin battle
    jin_qs: Vec<usize>,
    jin_i: usize,
    jin_correct: u32,
    jin_sel: usize,
    jin_set_reduced: bool,
    jin_done: bool,
    jin_offer: bool,
    jin_losses: u32,

    // riddles
    riddle_i: usize,
    riddle_sel: usize,
    riddle_answered: bool,
    riddle_right: Option<bool>,

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
    genie_q: bool,
    genie_sel: usize,
    cosma_fail: bool,

    amb_playing: Option<&'static str>,
}

const AMB_VOL: f32 = 0.12;

impl Game {
    pub async fn new() -> Self {
        let tex = Textures::load();
        let audio = crate::audio::load().await;
        Self {
            phase: Phase::Intro,
            progress: Progress::new(),
            tex,
            audio,
            dlg: DialogueBox::new(),
            intro_i: 0,
            intro_t: 0.0,
            wake_stage: 0,
            jin_step: 0,
            after_stage: 0,
            last_choice: String::new(),
            guard_line: 0,
            qte_line: 0,
            kandy_line: 0,
            kandy_asked: false,
            kandy_reaction: None,
            trap_line: 0,
            trap_t: 0.0,
            qte_active: false,
            qte_progress: 0.0,
            end_line: 0,
            crumble_t: 0.0,
            jin_qs: Vec::new(),
            jin_i: 0,
            jin_correct: 0,
            jin_sel: 0,
            jin_set_reduced: false,
            jin_done: false,
            jin_offer: false,
            jin_losses: 0,
            riddle_i: 0,
            riddle_sel: 0,
            riddle_answered: false,
            riddle_right: None,
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
            genie_q: false,
            genie_sel: 0,
            cosma_fail: false,
            amb_playing: None,
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

    pub fn update(&mut self, dt: f32) {
        match self.phase {
            Phase::Intro => self.up_intro(dt),
            Phase::Wake => self.up_wake(),
            Phase::JinMeet => self.up_jin_meet(),
            Phase::JinBattle => self.up_jin_battle(),
            Phase::JinAfter => self.up_jin_after(),
            Phase::CandyPuzzles => self.up_candy_puzzles(),
            Phase::Guards => {
                if !self.dlg.open {
                    self.dlg.say("Guard 1", script::GUARDS[0].text, &self.tex);
                }
                if let Some(_) = self.dlg.update(get_frame_time()) {
                    self.guard_line += 1;
                    if self.guard_line >= script::GUARDS.len() {
                        self.phase = Phase::Kandy;
                    } else {
                        let ln = &script::GUARDS[self.guard_line];
                        self.dlg.say(ln.who, ln.text, &self.tex);
                    }
                }
            }
            Phase::Kandy => self.up_kandy(),
            Phase::Trapdoor => self.up_trapdoor(dt),
            Phase::Aquarius => self.up_aquarius(),
            Phase::Cosma => self.up_cosma(dt),
            Phase::Crumble => self.up_crumble(dt),
            Phase::Ending => self.up_ending(),
        }
    }

    pub fn draw(&self) {
        match self.phase {
            Phase::Intro => self.dr_intro(),
            Phase::Wake | Phase::JinMeet => {
                draw_biome_bg(Biome::Candy, get_time() as f32);
                self.dlg.draw();
            }
            Phase::JinBattle => self.dr_jin_battle(),
            Phase::JinAfter => self.dr_candy_with_dlg(),
            Phase::CandyPuzzles => self.dr_gate(),
            Phase::Guards | Phase::Kandy => self.dr_candy_with_dlg(),
            Phase::Trapdoor => self.dr_trapdoor(),
            Phase::Aquarius => self.dr_aquarius(),
            Phase::Cosma => self.dr_cosma(),
            Phase::Crumble => self.dr_crumble(),
            Phase::Ending => self.dr_ending(),
        }
    }

    fn dr_candy_with_dlg(&self) {
        draw_biome_bg(Biome::Candy, get_time() as f32);
        self.dlg.draw();
    }

    // ---------- INTRO ----------
    fn up_intro(&mut self, dt: f32) {
        self.amb("house");
        self.intro_t += dt;
        if Self::adv_pressed() && self.intro_t > 0.5 {
            self.page_play();
            self.intro_i += 1;
            self.intro_t = 0.0;
            if self.intro_i >= script::INTRO_LINES.len() + 1 {
                self.phase = Phase::Wake;
            }
        }
    }

    fn dr_intro(&self) {
        let w = screen_width();
        let h = screen_height();
        if self.intro_i < script::INTRO_LINES.len() {
            clear_background(Color::from_rgba(24, 16, 20, 255));
            match self.intro_i {
                0..=1 => self.nadia_center(w, h),
                2..=3 => {
                    draw_texture_ex(&self.tex.grandparent, w * 0.5 - 48.0, h * 0.30,
                        WHITE, DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() });
                }
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

    fn nadia_center(&self, w: f32, h: f32) {
        draw_texture_ex(&self.tex.nadia, w * 0.5 - 48.0, h * 0.32,
            WHITE, DrawTextureParams { dest_size: Some(Vec2::new(96.0, 144.0)), ..Default::default() });
    }

    // ---------- WAKE ----------
    fn up_wake(&mut self) {
        self.amb("candy");
        if !self.dlg.open && self.wake_stage == 0 {
            self.dlg.say("River", script::WAKE.text, &self.tex);
        }
        if let Some(_) = self.dlg.update(get_frame_time()) {
            self.wake_stage += 1;
            match self.wake_stage {
                1 => self.dlg.say("River", script::WAKE2.text, &self.tex),
                2 => self.dlg.say("(Narration)", script::LOOK.text, &self.tex),
                _ => self.phase = Phase::JinMeet,
            }
        }
    }

    // ---------- JIN MEET ----------
    fn up_jin_meet(&mut self) {
        if !self.dlg.open && self.jin_step == 0 {
            self.dlg.say("Jin", script::JIN_INTRO[0].text, &self.tex);
        }
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
                3 => {
                    if self.last_choice == "no" {
                        // "Aquarius? or Cosma?" then regardless:
                        self.dlg.say("Jin", script::JIN_WARN[0].text, &self.tex);
                    } else {
                        self.dlg.say("Jin", script::JIN_WARN[0].text, &self.tex);
                    }
                }
                4 => {
                    self.dlg.ask("Jin", script::JIN_ANNOY[0].text, &[
                        ("In what way?", "a"),
                        ("how?", "a"),
                        ("annoying?", "a"),
                    ], &self.tex);
                }
                5 => self.dlg.say("Jin", script::JIN_ANNOY[1].text, &self.tex),
                6 => self.dlg.say("Jin", script::JIN_ANNOY[2].text, &self.tex),
                7 => self.dlg.say("Jin", script::JIN_ANNOY[3].text, &self.tex),
                8 => {
                    self.dlg.ask("Jin", "What do you ask?", &[
                        ("Risk?", "risk"),
                        ("Riddles?", "rd"),
                        ("Questions?", "rd"),
                    ], &self.tex);
                }
                9 => {
                    if self.last_choice == "risk" {
                        self.dlg.say("Jin", script::JIN_RISK[0].text, &self.tex);
                    } else {
                        self.dlg.say("Jin", script::JIN_RIDDLE_DEMO[0].text, &self.tex);
                    }
                }
                10 => self.dlg.say("Jin", script::JIN_RISK[1].text, &self.tex),
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

    // ---------- JIN BATTLE ----------
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
        draw_biome_bg(Biome::Candy, get_time() as f32);
        let w = screen_width();
        if self.jin_done {
            self.dlg.draw();
            return;
        }
        let set = if self.jin_set_reduced { jin_battle::REDUCED_SET } else { jin_battle::FULL_SET };
        let q = &set[self.jin_qs[self.jin_i]];
        quiz_header(
            &format!("JIN'S BATTLE  {} / {}", self.jin_i + 1, self.jin_qs.len()),
            q.question,
        );
        for (i, o) in q.options.iter().enumerate() {
            draw_option_row(i, o, 250.0 + i as f32 * 44.0, i == self.jin_sel, None);
        }
        draw_text("1-4 / arrows + z", w - 220.0, screen_height() - 24.0, 16.0,
            Color::from_rgba(200, 200, 220, 140));
        let _ = w;
    }

    // ---------- CANDY RIDDLES + GATE (before guards) ----------
    fn up_candy_puzzles(&mut self) {
        // Script: at least 1 riddle before the puzzle; we run both riddles then the gate.
        if self.progress.candy_riddles_done < script::CANDY_RIDDLES.len() as u32 {
            self.run_riddles(script::CANDY_RIDDLES, |g| {
                g.progress.candy_riddles_done += 1;
                g.progress.candy_riddles_done
            });
            return;
        }
        if !self.progress.gate_solved {
            self.up_gate();
            return;
        }
        self.phase = Phase::Guards;
    }

    fn up_gate(&mut self) {
        if self.gate_marks.is_empty() {
            self.gate_marks = vec![None; gate::THRONE_GATE.slots.len()];
        }
        // Selection over unfilled slots; pick mark from 5 options.
        let slot = &gate::THRONE_GATE.slots[self.gate_slot.min(gate::THRONE_GATE.slots.len() - 1)];
        let mut opts: Vec<&'static str> = vec![slot.answer];
        opts.extend_from_slice(slot.decoys);
        // deterministic shuffle so the answer isn't always first
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
                    self.gate_msg = Some(("The gate grinds open...".to_string(), 3.0));
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
        draw_biome_bg(Biome::Candy, get_time() as f32);
        let w = screen_width();
        // stone gate backdrop
        draw_rectangle(w * 0.5 - 260.0, 90.0, 520.0, 470.0, Color::from_rgba(90, 80, 95, 255));
        draw_rectangle_lines(w * 0.5 - 260.0, 90.0, 520.0, 470.0, 6.0, Color::from_rgba(60, 52, 66, 255));
        quiz_header("THE LONE STONE STORY OF THRONE GATE", "Fill every box. z/Enter to set the mark. Multiple tries allowed.");

        // passage with slots shown as [ ? ] or filled
        let words: Vec<&str> = gate::THRONE_GATE.passage.split(' ').collect();
        let mut line = String::new();
        let mut y = 170.0;
        let mut word_i = 0usize;
        for slot in gate::THRONE_GATE.slots {
            while word_i <= slot.after_word && word_i < words.len() {
                let candidate = format!("{} {}", line, words[word_i]);
                if measure_text(&candidate, None, 18, 1.0).width > 470.0 {
                    draw_text(&line, w * 0.5 - 240.0, y, 18.0, WHITE);
                    y += 24.0;
                    line = words[word_i].to_string();
                } else {
                    line = if line.is_empty() { words[word_i].to_string() } else { candidate };
                }
                word_i += 1;
            }
            let mark = self.gate_marks.get(self.gate_slot).copied().flatten();
            let shown = mark.unwrap_or("?");
            line = format!("{} [{}]", line, shown);
        }
        while word_i < words.len() {
            let candidate = format!("{} {}", line, words[word_i]);
            if measure_text(&candidate, None, 18, 1.0).width > 470.0 {
                draw_text(&line, w * 0.5 - 240.0, y, 18.0, WHITE);
                y += 24.0;
                line = words[word_i].to_string();
            } else {
                line = if line.is_empty() { words[word_i].to_string() } else { candidate };
            }
            word_i += 1;
        }
        draw_text(&line, w * 0.5 - 240.0, y, 18.0, WHITE);

        // option row for current slot
        if self.gate_slot < gate::THRONE_GATE.slots.len() {
            for (i, o) in self.gate_options.iter().enumerate() {
                draw_option_row(i, o, 560.0 + i as f32 * 26.0, i == self.gate_sel, None);
            }
        }
        if let Some((msg, _)) = &self.gate_msg {
            draw_text(msg, w * 0.5 - 130.0, 70.0, 22.0, YELLOW);
        }
    }

    // ---------- JIN AFTER ----------
    fn up_jin_after(&mut self) {
        if !self.dlg.open && self.after_stage == 0 {
            if self.jin_losses == 0 && self.jin_correct == self.jin_qs.len() as u32 {
                self.progress.has_horse = true;
                self.dlg.say("Jin", script::JIN_PERFECT[0].text, &self.tex);
            } else {
                self.dlg.say("Jin", script::JIN_LOST_ONE[0].text, &self.tex);
            }
        }
        if let Some(_) = self.dlg.update(get_frame_time()) {
            self.after_stage += 1;
            match self.after_stage {
                1 => {
                    self.dlg.say("(Narration)", script::JIN_BOOK_OFFER[0].text, &self.tex)
                }
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
                6 => {
                    self.dlg.say("Jin", script::JIN_BOOK_RETURN[2].text, &self.tex);
                }
                _ => self.phase = Phase::CandyPuzzles,
            }
        }
    }

    // ---------- KANDY ----------
    fn up_kandy(&mut self) {
        if !self.dlg.open && self.kandy_line == 0 {
            self.dlg.say("Kandy Kween", script::KANDY_INTRO[0].text, &self.tex);
        }
        if let Some(_) = self.dlg.update(get_frame_time()) {
            self.kandy_line += 1;
            if self.kandy_line < script::KANDY_INTRO.len() {
                let ln = &script::KANDY_INTRO[self.kandy_line];
                self.dlg.say(ln.who, ln.text, &self.tex);
            } else if !self.kandy_asked {
                self.kandy_asked = true;
                self.dlg.ask("Kandy Kween", "The trials. Be honest:", &[
                    ("I thought they were great", "g"),
                    ("They were really easy", "e"),
                    ("Those puzzles sucked", "s"),
                ], &self.tex);
            } else if self.kandy_reaction.is_none() {
                self.kandy_reaction = Some(match self.last_choice.as_str() {
                    "g" => script::KANDY_GREAT_TEXTS,
                    "e" => script::KANDY_EASY_TEXTS,
                    _ => script::KANDY_SUCKED_TEXTS,
                });
                self.kandy_line = 0;
                let ln = &self.kandy_reaction.unwrap()[0];
                self.dlg.say(ln.who, ln.text, &self.tex);
            } else {
                let texts = self.kandy_reaction.unwrap();
                if self.kandy_line - script::KANDY_INTRO.len() < texts.len() {
                    let idx = self.kandy_line - script::KANDY_INTRO.len();
                    let ln = &texts[idx];
                    self.dlg.say(ln.who, ln.text, &self.tex);
                } else {
                    self.phase = Phase::Trapdoor;
                }
            }
        }
    }

    // ---------- TRAPDOOR + QTE ----------
    fn up_trapdoor(&mut self, dt: f32) {
        if !self.dlg.open && self.trap_line == 0 && !self.qte_active {
            self.splash_play();
            let ln = &script::TRAPDOOR[self.trap_line];
            self.dlg.say(ln.who, ln.text, &self.tex);
        }
        if !self.qte_active {
            if let Some(_) = self.dlg.update(get_frame_time()) {
                self.trap_line += 1;
                if self.trap_line < script::TRAPDOOR.len() {
                    let ln = &script::TRAPDOOR[self.trap_line];
                    self.dlg.say(ln.who, ln.text, &self.tex);
                } else {
                    self.qte_active = true;
                    self.qte_progress = 0.3;
                    self.dlg.close();
                }
            }
        } else {
            // QTE: mash to resist, fail regardless (per script)
            self.trap_t += dt;
            if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
                self.qte_progress += 0.05;
            }
            self.qte_progress -= dt * 0.35;
            if self.qte_progress >= 1.0 || (self.qte_progress <= 0.0 && self.trap_t > 3.0) {
                self.qte_active = false;
                self.trap_t = 0.0;
                self.dlg.say("(Narration)", script::QTE_LINES[2].text, &self.tex);
            }
        }
        if !self.qte_active && self.trap_line >= script::TRAPDOOR.len() && self.dlg.open
            && self.dlg_last_done()
        {
            // advance through QTE_LINES then DRAGGED then Aquarius
            if let Some(_) = self.dlg.update(get_frame_time()) {
                self.qte_line += 1;
                if self.qte_line < script::QTE_LINES.len() {
                    let ln = &script::QTE_LINES[self.qte_line];
                    self.dlg.say(ln.who, ln.text, &self.tex);
                } else if self.qte_line == script::QTE_LINES.len() + script::DRAGGED.len() {
                    self.phase = Phase::Aquarius;
                } else {
                    let idx = self.qte_line - script::QTE_LINES.len();
                    let ln = &script::DRAGGED[idx.min(script::DRAGGED.len() - 1)];
                    self.dlg.say(ln.who, ln.text, &self.tex);
                }
            }
        }
    }

    fn dlg_last_done(&self) -> bool {
        true // update() only fires payload when done; harmless gate
    }

    fn dr_trapdoor(&self) {
        let t = get_time() as f32;
        let mut y = 0.0;
        while y < screen_height() {
            let k = (y / screen_height() * 6.0 + t) % 1.0;
            let c = Color::from_rgba((160.0 * k) as u8, (60.0 * k) as u8, (120.0 * k) as u8, 255);
            draw_rectangle(0.0, y, screen_width(), 24.0, c);
            y += 24.0;
        }
        if self.qte_active {
            let w = screen_width();
            draw_text("MASH! (space / click)", w * 0.5 - 130.0, 200.0, 30.0, YELLOW);
            draw_rectangle(w * 0.5 - 200.0, 240.0, 400.0, 26.0, GRAY);
            draw_rectangle(w * 0.5 - 200.0, 240.0, 400.0 * self.qte_progress.clamp(0.0, 1.0), 26.0, PINK);
        }
        self.dlg.draw();
    }

    // ---------- AQUARIUS ----------
    fn up_aquarius(&mut self) {
        self.amb("aqua");
        if self.progress.aqua_riddles_done < script::AQUARIUS_RIDDLES.len() as u32 {
            self.run_riddles(script::AQUARIUS_RIDDLES, |g| {
                g.progress.aqua_riddles_done += 1;
                g.progress.aqua_riddles_done
            });
            return;
        }
        if !self.progress.maze_solved {
            self.up_maze();
            return;
        }
        self.phase = Phase::Cosma;
    }

    fn run_riddles(&mut self, set: &'static [script::Riddle], bump: fn(&mut Self) -> u32) {
        let r = &set[self.riddle_i];
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
                // one-try only: restart all riddles in this biome
                self.riddle_i = 0;
                self.riddle_answered = false;
                self.riddle_right = None;
                self.riddle_sel = 0;
            } else {
                self.riddle_i += 1;
                self.riddle_answered = false;
                self.riddle_right = None;
                self.riddle_sel = 0;
                bump(self);
                if self.riddle_i >= set.len() {
                    self.riddle_i = 0;
                }
            }
        }
    }

    fn submit_riddle(&mut self, answer: usize) {
        self.riddle_answered = true;
        self.riddle_right = Some(self.riddle_sel == answer);
        if self.riddle_right == Some(true) { self.good_play(); } else { self.bad_play(); }
    }

    fn up_maze(&mut self) {
        if self.maze_order.is_empty() {
            self.maze_order = (0..maze::MAZE_QUESTIONS.len()).collect();
            let mut seed = (get_time() * 1000.0) as u64 | 1;
            for i in (1..self.maze_order.len()).rev() {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let j = ((seed >> 33) as usize) % (i + 1);
                self.maze_order.swap(i, j);
            }
        }
        let n = maze::MAZE_QUESTIONS.len();
        if self.maze_step >= n {
            self.progress.maze_solved = true;
            self.progress.ingredients += 1;
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
                self.bad_play();
                self.maze_step = self.maze_step.saturating_sub(1);
                self.maze_depth = (self.maze_depth - 300.0 / n as f32).max(0.0);
            }
        }
    }

    fn dr_aquarius(&self) {
        draw_biome_bg(Biome::Aquarius, get_time() as f32);
        let w = screen_width();
        if self.progress.aqua_riddles_done < script::AQUARIUS_RIDDLES.len() as u32 {
            let r = &script::AQUARIUS_RIDDLES[self.riddle_i];
            quiz_header("AQUARIUS RIDDLE", r.question);
            for (i, o) in r.options.iter().enumerate() {
                draw_option_row(i, o, 250.0 + i as f32 * 44.0,
                    i == self.riddle_sel,
                    if self.riddle_answered { Some(i != r.answer) } else { None });
            }
            if self.riddle_answered {
                let right = self.riddle_right == Some(true);
                draw_text(
                    if right { "Correct! z to continue" } else { "Wrong. One try only. z restarts the riddles" },
                    w * 0.5 - 160.0, 470.0, 24.0,
                    if right { GREEN } else { RED });
            }
            self.dlg.draw();
            return;
        }
        if !self.progress.maze_solved {
            quiz_header("THE GRAVE CAVE MAZE OF CRAZE", "Choose the path. Fail: silently sent back.");
            let step = self.maze_step.min(self.maze_order.len().saturating_sub(1));
            let q = &maze::MAZE_QUESTIONS[self.maze_order[step]];
            draw_wrapped(q.question, w * 0.5 - 300.0, 220.0, 600.0, 28.0);
            for i in 0..4 {
                draw_option_row(i, maze::DIR_NAMES[i], 340.0 + i as f32 * 44.0, i == self.maze_sel, None);
            }
            draw_rectangle(w - 44.0, 200.0, 14.0, 300.0, Color::from_rgba(255, 255, 255, 40));
            draw_rectangle(w - 44.0, 500.0 - self.maze_depth, 14.0, self.maze_depth.max(6.0), PINK);
            // clue reminders
            draw_wrapped(maze::MAZE_CLUES[1].text, 40.0, 600.0, 400.0, 22.0);
            draw_wrapped(maze::MAZE_CLUES[3].text, 40.0, 650.0, 400.0, 22.0);
        }
    }

    // ---------- COSMA ----------
    fn up_cosma(&mut self, dt: f32) {
        self.amb("cosma");
        if self.progress.cosma_riddles_done < script::COSMA_RIDDLES.len() as u32 {
            self.run_riddles(script::COSMA_RIDDLES, |g| {
                g.progress.cosma_riddles_done += 1;
                g.progress.cosma_riddles_done
            });
            return;
        }
        if self.cosma_fail {
            if is_key_pressed(KeyCode::Z) {
                self.cosma_fail = false;
                self.poem = (self.poem + 1) % cosma::POEMS.len();
                self.star_meter = 0;
                self.lives = cosma::LIVES;
                self.slab = 0;
                self.genie_q = false;
                self.stars.clear();
            }
            return;
        }
        if self.slab < cosma::SLABS {
            self.up_star_catch(dt);
        } else if !self.genie_q {
            self.genie_q = true;
            let p = &cosma::POEMS[self.poem];
            self.dlg.say("Magic Fire Genie", &format!("The torches burn. {}", p.question), &self.tex);
        } else {
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
                        self.phase = Phase::Crumble;
                    } else {
                        self.bad_play();
                        self.cosma_fail = true;
                    }
                }
            }
        }
    }

    fn up_star_catch(&mut self, dt: f32) {
        if rand::gen_range(0.0, 1.0) < dt * 2.2 {
            let dying = rand::gen_range(0.0, 1.0) < 0.28;
            self.stars.push((rand::gen_range(30.0, 994.0), -20.0, rand::gen_range(120.0, 240.0), dying));
        }
        for s in self.stars.iter_mut() {
            s.1 += s.2 * dt;
        }
        // paddle: A/D or arrows
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { self.paddle_pos -= dt * 0.9; }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { self.paddle_pos += dt * 0.9; }
        self.paddle_pos = self.paddle_pos.clamp(0.0, 1.0);

        let px = self.paddle_x();
        let mut caught: Option<bool> = None; // Some(true)=good star, Some(false)=dying
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
            return;
        }
        if self.star_meter >= cosma::STAR_TARGET {
            self.star_meter = 0;
            self.slab += 1;
            self.good_play();
        }
    }

    fn paddle_x(&self) -> f32 {
        60.0 + self.paddle_pos * 904.0
    }

    fn dr_cosma(&self) {
        draw_biome_bg(Biome::Cosma, get_time() as f32);
        let w = screen_width();

        if self.progress.cosma_riddles_done < script::COSMA_RIDDLES.len() as u32 {
            let r = &script::COSMA_RIDDLES[self.riddle_i];
            quiz_header("COSMA RIDDLE", r.question);
            for (i, o) in r.options.iter().enumerate() {
                draw_option_row(i, o, 250.0 + i as f32 * 44.0, i == self.riddle_sel,
                    if self.riddle_answered { Some(i != r.answer) } else { None });
            }
            if self.riddle_answered {
                let right = self.riddle_right == Some(true);
                draw_text(
                    if right { "Correct! z to continue" } else { "Wrong. One try only. z restarts the riddles" },
                    w * 0.5 - 160.0, 470.0, 24.0,
                    if right { GREEN } else { RED });
            }
            return;
        }
        if self.cosma_fail {
            clear_background(BLACK);
            draw_text("The stars scatter...", w * 0.5 - 110.0, 300.0, 30.0, RED);
            draw_text("z: try again (a new poem will form)", w * 0.5 - 170.0, 350.0, 22.0, GRAY);
            return;
        }
        let p = &cosma::POEMS[self.poem];
        if self.slab < cosma::SLABS {
            quiz_header("THE RAINBOW ROAD",
                &format!("Catch falling stars, avoid dying stars. Lives: {}. Slab {}/{}",
                    self.lives, self.slab + 1, cosma::SLABS));
            for (i, line) in p.lines.iter().enumerate() {
                let c = if i < self.slab as usize { WHITE } else { Color::from_rgba(255, 255, 255, 60) };
                draw_text(line, 40.0, 160.0 + i as f32 * 24.0, 18.0, c);
            }
            for s in &self.stars {
                let tex: &Texture2D = if s.3 { &self.tex.dying_star } else { &self.tex.star };
                draw_texture_ex(&tex, s.0 - 16.0, s.1 - 16.0, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(32.0, 32.0)), ..Default::default() });
            }
            let px = self.paddle_x();
            draw_rectangle(px - 46.0, 570.0, 92.0, 14.0, PINK);
            draw_rectangle(40.0, 640.0, 400.0, 20.0, GRAY);
            draw_rectangle(40.0, 640.0, 400.0 * (self.star_meter as f32 / cosma::STAR_TARGET as f32), 20.0, YELLOW);
        } else if self.genie_q {
            if self.dlg.open {
                self.dlg.draw();
            } else {
                quiz_header("GENIE'S QUESTION", p.question);
                for (i, o) in p.options.iter().enumerate() {
                    draw_option_row(i, o, 300.0 + i as f32 * 44.0, i == self.genie_sel, None);
                }
            }
        }
    }

    // ---------- CRUMBLE + ENDING ----------
    fn up_crumble(&mut self, dt: f32) {
        self.crumble_t += dt;
        if Self::adv_pressed() && self.crumble_t > 1.0 {
            self.phase = Phase::Ending;
        }
    }

    fn dr_crumble(&self) {
        clear_background(Color::from_rgba(10, 6, 16, 255));
        let t = self.crumble_t;
        let mut seed = 99u64;
        for _ in 0..40 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = ((seed >> 33) % 1024) as f32;
            let y = (((seed >> 20) % 720) as f32 + t * 120.0) % 720.0;
            draw_rectangle(x, y, 6.0, 6.0, Color::from_rgba(255, 90, 110, 200));
        }
        draw_text("The dimension is crumbling. RUN.", screen_width() * 0.5 - 190.0, 340.0, 30.0, RED);
    }

    fn up_ending(&mut self) {
        if !self.dlg.open && self.end_line == 0 {
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
