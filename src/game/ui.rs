//! Undertale-style dialogue: bordered box, portrait, typewriter text, choice list.
//! Click anywhere / press Z / Enter to advance. Number keys pick choices.

use macroquad::prelude::*;
use crate::art::Textures;
use std::collections::HashMap;
use std::sync::OnceLock;

const BOX_H: f32 = 190.0;
const BOX_MARGIN: f32 = 20.0;

/// Leak the portrait textures so DialogueBox can hold 'static refs.
static PORTRAITS: OnceLock<HashMap<String, Texture2D>> = OnceLock::new();

fn portraits(tex: &Textures) -> &'static HashMap<String, Texture2D> {
    PORTRAITS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("River".to_string(), tex.river.clone());
        m.insert("Jin".to_string(), tex.jin.clone());
        m.insert("Kandy Kween".to_string(), tex.kandy.clone());
        m.insert("Matt".to_string(), tex.matt.clone());
        m.insert("Magic Fire Genie".to_string(), tex.genie.clone());
        m.insert("Guard 1".to_string(), tex.guard_a.clone());
        m.insert("Guard 2".to_string(), tex.guard_b.clone());
        m.insert("Nadia".to_string(), tex.nadia.clone());
        m.insert("Grandparent".to_string(), tex.grandparent.clone());
        m
    })
}

pub struct DialogueBox {
    pub open: bool,
    speaker: String,
    text: String,
    shown: usize,
    timer: f32,
    portrait: Option<&'static Texture2D>,
    choices: Vec<(String, String)>, // (label, payload)
    choice_sel: usize,
    pub resolved: Option<String>,
    /// Blue frame like the Undertale battle box.
    border: Color,
}

impl DialogueBox {
    pub fn new() -> Self {
        Self {
            open: false,
            speaker: String::new(),
            text: String::new(),
            shown: 0,
            timer: 0.0,
            portrait: None,
            choices: Vec::new(),
            choice_sel: 0,
            resolved: None,
            border: Color::from_rgba(40, 200, 255, 255),
        }
    }

    pub fn say(&mut self, who: &str, text: &str, tex: &Textures) {
        self.open = true;
        self.speaker = who.to_string();
        self.text = text.to_string();
        self.shown = 0;
        self.timer = 0.0;
        self.portrait = portraits(tex).get(who);
        self.choices.clear();
        self.resolved = None;
    }

    pub fn ask(&mut self, who: &str, text: &str, choices: &[(&str, &str)], tex: &Textures) {
        self.say(who, text, tex);
        self.choices = choices.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
        self.choice_sel = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.choices.clear();
    }

    /// Returns Some(payload) when the player finishes the line, or picks a choice payload.
    pub fn update(&mut self, dt: f32) -> Option<String> {
        if !self.open {
            return None;
        }

        // Typewriter: 40 chars/sec, blip every 3rd char handled by scene.
        if self.shown < self.text.len() {
            self.timer += dt;
            let want = (self.timer * 40.0) as usize;
            if want > self.shown {
                self.shown = want.min(self.text.len());
            }
        }

        let done = self.shown >= self.text.len();

        // Skip / advance.
        let advance = is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter)
            || is_mouse_button_pressed(MouseButton::Left);

        if !self.choices.is_empty() && done {
            // Choice navigation.
            if is_key_pressed(KeyCode::Up) { self.choice_sel = self.choice_sel.saturating_sub(1); }
            if is_key_pressed(KeyCode::Down) {
                self.choice_sel = (self.choice_sel + 1).min(self.choices.len() - 1);
            }
            if is_key_pressed(KeyCode::Key1) { self.choice_sel = 0; }
            if is_key_pressed(KeyCode::Key2) && self.choices.len() > 1 { self.choice_sel = 1; }
            if is_key_pressed(KeyCode::Key3) && self.choices.len() > 2 { self.choice_sel = 2; }
            if is_key_pressed(KeyCode::Key4) && self.choices.len() > 3 { self.choice_sel = 3; }

            // Mouse hover pick.
            let (mx, my) = mouse_position();
            for (i, (_, _)) in self.choices.iter().enumerate() {
                let r = self.choice_rect(i);
                if r.contains(Vec2::new(mx, my)) {
                    self.choice_sel = i;
                    if is_mouse_button_pressed(MouseButton::Left) {
                        let payload = self.choices[i].1.clone();
                        self.close();
                        return Some(payload);
                    }
                }
            }

            if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) {
                let payload = self.choices[self.choice_sel].1.clone();
                self.close();
                return Some(payload);
            }
            return None;
        }

        if done && advance {
            self.close();
            return Some(String::new());
        }
        if !done && advance {
            self.shown = self.text.len(); // skip typewriter
        }
        None
    }

    fn choice_rect(&self, i: usize) -> Rect {
        let w = screen_width();
        let x = w * 0.5 - 260.0 + 30.0;
        let y = screen_height() - BOX_H - BOX_MARGIN + 60.0 + i as f32 * 30.0;
        Rect::new(x, y, 460.0, 26.0)
    }

    pub fn draw(&self) {
        if !self.open {
            return;
        }
        let w = screen_width();
        let h = screen_height();
        let box_y = h - BOX_H - BOX_MARGIN;
        let box_w = w - BOX_MARGIN * 2.0;

        // Outer black + animated border.
        draw_rectangle(BOX_MARGIN - 4.0, box_y - 4.0, box_w + 8.0, BOX_H + 8.0, BLACK);
        let pulse = 0.6 + 0.4 * (get_time() as f32 * 3.0).sin();
        draw_rectangle_lines(BOX_MARGIN - 4.0, box_y - 4.0, box_w + 8.0, BOX_H + 8.0,
            3.0, Color::new(self.border.r, self.border.g, self.border.b, pulse));
        draw_rectangle(BOX_MARGIN, box_y, box_w, BOX_H, Color::from_rgba(8, 8, 14, 245));

        // Portrait.
        let mut text_x = BOX_MARGIN + 24.0;
        if let Some(p) = self.portrait {
            let pw = 140.0;
            let ph = 150.0;
            draw_texture_ex(&p, BOX_MARGIN + 18.0, box_y + 20.0, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(pw, ph)), ..Default::default() });
            text_x += pw + 20.0;
        }

        // Speaker tag.
        if !self.speaker.is_empty() && self.speaker != "(Narration)" {
            let tag_w = measure_text(&self.speaker, None, 20, 1.0).width + 24.0;
            draw_rectangle(text_x, box_y - 14.0, tag_w, 28.0, Color::from_rgba(255, 105, 180, 255));
            draw_text(&self.speaker, text_x + 12.0, box_y + 5.0, 20.0, Color::from_rgba(30, 10, 25, 255));
        }

        // Body text with word wrap + typewriter.
        let visible = &self.text[..self.shown.min(self.text.len())];
        draw_wrapped(visible, text_x, box_y + 34.0, w - text_x - 40.0, 24.0);

        // Choices.
        for (i, (label, _)) in self.choices.iter().enumerate() {
            let r = self.choice_rect(i);
            let sel = i == self.choice_sel;
            if sel {
                draw_rectangle(r.x - 10.0, r.y - 2.0, r.w + 20.0, r.h, Color::from_rgba(255, 105, 180, 60));
                draw_text(">", r.x - 22.0, r.y + 18.0, 22.0, PINK);
            }
            draw_text(label, r.x, r.y + 18.0, 20.0,
                if sel { WHITE } else { Color::from_rgba(200, 200, 210, 255) });
        }

        // Advance hint.
        let done = self.shown >= self.text.len();
        if done && self.choices.is_empty() {
            let blink = (get_time() * 2.0) as i32 % 2 == 0;
            if blink {
                draw_text("z / click", w - 130.0, box_y + BOX_H - 12.0, 16.0,
                    Color::from_rgba(120, 200, 255, 200));
            }
        }
    }
}

/// Simple greedy word wrap.
pub fn draw_wrapped(text: &str, x: f32, mut y: f32, max_w: f32, line_h: f32) {
    let size = 20;
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
            if measure_text(&candidate, None, size, 1.0).width > max_w && !line.is_empty() {
                draw_text(&line, x, y, size as f32, WHITE);
                y += line_h;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        draw_text(&line, x, y, size as f32, WHITE);
        y += line_h;
    }
}

/// Centered quiz option row (for riddles + battles).
pub fn draw_option_row(i: usize, label: &str, y: f32, selected: bool, disabled: Option<bool>) {
    let w = screen_width();
    let x = w * 0.5 - 300.0;
    let color = match disabled {
        Some(true) => Color::from_rgba(120, 120, 130, 255),
        _ => if selected { WHITE } else { Color::from_rgba(190, 190, 200, 255) },
    };
    if selected {
        draw_rectangle(x - 12.0, y - 22.0, 620.0, 30.0, Color::from_rgba(255, 105, 180, 50));
        draw_text(">", x - 26.0, y, 22.0, PINK);
    }
    let tag = format!("{}. {}", i + 1, label);
    draw_text(&tag, x, y, 20.0, color);
}

pub fn quiz_header(title: &str, sub: &str) {
    let w = screen_width();
    draw_text(title, w * 0.5 - measure_text(title, None, 34, 1.0).width * 0.5, 70.0, 34.0, PINK);
    draw_wrapped(sub, w * 0.5 - 330.0, 110.0, 660.0, 26.0);
}
