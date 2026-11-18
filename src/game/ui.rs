//! Dialogue UI. Undertale-style panel restyled to the cream/plum/pink tokens:
//! VT323 for spoken text, Press Start 2P for speaker chips, cream panel with a
//! plum border. Public API and geometry are unchanged for the run loop.

use macroquad::prelude::*;
use crate::art::Textures;
use std::collections::HashMap;
use std::sync::OnceLock;

const BOX_H: f32 = 190.0;
const BOX_MARGIN: f32 = 20.0;

const CREAM: Color = Color::new(1.0, 0.96, 0.91, 0.99);
const CREAM_DIM: Color = Color::new(0.94, 0.84, 0.75, 1.0);
const PLUM: Color = Color::from_rgba(74, 42, 99, 255);
const INK: Color = Color::from_rgba(46, 26, 71, 255);
const INK_SOFT: Color = Color::from_rgba(96, 74, 120, 255);
const PINK: Color = Color::from_rgba(226, 106, 156, 255);
const PINK_PALE: Color = Color::from_rgba(255, 214, 230, 255);

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
        m.insert("Gillbert".to_string(), tex.gill.clone());
        m.insert("Scalleon".to_string(), tex.scal.clone());
        m.insert("Market Lady".to_string(), tex.lady.clone());
        m
    })
}

// --- pixel fonts -----------------------------------------------------------

static F_DIALOG: OnceLock<Font> = OnceLock::new();
static F_TAG: OnceLock<Font> = OnceLock::new();

/// Load the pixel fonts through macroquad's file loader (works natively and on
/// wasm, where std::fs does not exist). Must be awaited once before drawing.
pub async fn ensure_fonts() {
    if F_DIALOG.get().is_some() && F_TAG.get().is_some() {
        return;
    }
    if let Ok(bytes) = macroquad::file::load_file("assets/fonts/VT323-Regular.ttf").await {
        if let Ok(f) = load_ttf_font_from_bytes(&bytes) {
            let _ = F_DIALOG.set(f);
        }
    }
    if let Ok(bytes) = macroquad::file::load_file("assets/fonts/PressStart2P-Regular.ttf").await {
        if let Ok(f) = load_ttf_font_from_bytes(&bytes) {
            let _ = F_TAG.set(f);
        }
    }
    assert!(F_DIALOG.get().is_some(), "VT323 font failed to load");
    assert!(F_TAG.get().is_some(), "Press Start 2P font failed to load");
}

fn dialog_font() -> &'static Font {
    F_DIALOG.get().expect("fonts loaded")
}
fn tag_font() -> &'static Font {
    F_TAG.get().expect("fonts loaded")
}

fn ttext_w(text: &str, font: &Font, size: u16) -> f32 {
    measure_text(text, Some(font), size, 1.0).width
}

fn ttext(text: &str, x: f32, y: f32, font: &Font, size: u16, color: Color) {
    draw_text_ex(text, x, y, TextParams {
        font: Some(font),
        font_size: size,
        color,
        ..Default::default()
    });
}

/// Press Start 2P text (chips, titles, HUD).
pub fn draw_tag(text: &str, x: f32, y: f32, size: u16, color: Color) {
    ttext(text, x, y, tag_font(), size, color);
}

/// VT323 text (narrative, labels).
pub fn draw_dlg_text(text: &str, x: f32, y: f32, size: u16, color: Color) {
    ttext(text, x, y, dialog_font(), size, color);
}

pub fn dlg_width(text: &str, size: u16) -> f32 {
    ttext_w(text, dialog_font(), size)
}

pub fn tag_width(text: &str, size: u16) -> f32 {
    ttext_w(text, tag_font(), size)
}

pub struct DialogueBox {
    pub open: bool,
    speaker: String,
    text: String,
    shown: usize,
    timer: f32,
    portrait: Option<&'static Texture2D>,
    choices: Vec<(String, String)>,
    choice_sel: usize,
    pub resolved: Option<String>,
    /// Set by the autoplay harness: treat this frame as if the advance key was
    /// pressed. Cleared as soon as it is read.
    pub force_advance: bool,
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
            force_advance: false,
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

    pub fn speaker(&self) -> &str {
        &self.speaker
    }

    /// How many characters of the line are currently revealed. The game watches
    /// this to fire typewriter blips, so every screen that shows dialogue gets
    /// the sound without each call site having to opt in.
    pub fn chars_shown(&self) -> usize {
        if self.open { self.shown } else { 0 }
    }

    /// Returns Some(payload) when the player finishes the line, or picks a choice payload.
    pub fn update(&mut self, dt: f32) -> Option<String> {
        if !self.open {
            return None;
        }

        if self.shown < self.text.len() {
            self.timer += dt;
            let want = (self.timer * 46.0) as usize;
            if want > self.shown {
                self.shown = want.min(self.text.len());
            }
        }

        let done = self.shown >= self.text.len();
        let advance = self.force_advance || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter)
            || is_mouse_button_pressed(MouseButton::Left);
        self.force_advance = false;

        if !self.choices.is_empty() && done {
            if is_key_pressed(KeyCode::Up) { self.choice_sel = self.choice_sel.saturating_sub(1); }
            if is_key_pressed(KeyCode::Down) {
                self.choice_sel = (self.choice_sel + 1).min(self.choices.len() - 1);
            }
            if is_key_pressed(KeyCode::Key1) { self.choice_sel = 0; }
            if is_key_pressed(KeyCode::Key2) && self.choices.len() > 1 { self.choice_sel = 1; }
            if is_key_pressed(KeyCode::Key3) && self.choices.len() > 2 { self.choice_sel = 2; }
            if is_key_pressed(KeyCode::Key4) && self.choices.len() > 3 { self.choice_sel = 3; }

            let (mx, my) = mouse_position();
            for (i, _) in self.choices.iter().enumerate() {
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

            // `advance` already folds in the harness flag, the same as the
            // non-choice path, so an automated run can actually pick an option
            if advance {
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
            self.shown = self.text.len();
        }
        None
    }

    fn choice_rect(&self, i: usize) -> Rect {
        let w = screen_width();
        let x = w * 0.5 - 260.0 + 30.0;
        let y = screen_height() - BOX_H - BOX_MARGIN + 64.0 + i as f32 * 32.0;
        Rect::new(x, y, 460.0, 28.0)
    }

    pub fn draw(&self) {
        if !self.open {
            return;
        }
        let w = screen_width();
        let h = screen_height();
        let box_y = h - BOX_H - BOX_MARGIN;
        let box_w = w - BOX_MARGIN * 2.0;
        let f = dialog_font();

        // soft drop shadow, cream panel, plum border
        draw_rectangle(BOX_MARGIN - 5.0, box_y - 3.0, box_w + 10.0, BOX_H + 10.0, Color::from_rgba(46, 26, 71, 70));
        let pulse = 0.75 + 0.25 * (get_time() as f32 * 2.4).sin();
        draw_rectangle(BOX_MARGIN - 4.0, box_y - 4.0, box_w + 8.0, BOX_H + 8.0,
            Color::new(PLUM.r, PLUM.g, PLUM.b, pulse));
        draw_rectangle(BOX_MARGIN, box_y, box_w, BOX_H, CREAM);
        draw_rectangle(BOX_MARGIN + 4.0, box_y + 4.0, box_w - 8.0, 2.0, PINK_PALE);

        // Portrait (keeps its 2:3 aspect, no stretching).
        let mut text_x = BOX_MARGIN + 28.0;
        if let Some(p) = self.portrait {
            let ph = 150.0;
            let pw = ph * 2.0 / 3.0;
            draw_texture_ex(p, BOX_MARGIN + 26.0, box_y + 22.0, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(pw, ph)), ..Default::default() });
            // soft accent panel behind the portrait
            draw_rectangle_lines(BOX_MARGIN + 22.0, box_y + 18.0, pw + 8.0, ph + 8.0, 2.0, CREAM_DIM);
            text_x += pw + 34.0;
        }

        // Speaker chip (Press Start 2P).
        if !self.speaker.is_empty() && self.speaker != "(Narration)" {
            let tag = self.speaker.as_str();
            let tf = tag_font();
            let size: u16 = 15;
            let tw = ttext_w(tag, tf, size);
            let chip_w = tw + 30.0;
            let chip_y = box_y - 18.0;
            draw_rectangle(BOX_MARGIN + 8.0, chip_y, chip_w, 30.0, PINK);
            draw_rectangle_lines(BOX_MARGIN + 8.0, chip_y, chip_w, 30.0, 2.0, PLUM);
            ttext(tag, BOX_MARGIN + 23.0, chip_y + 22.0, tf, size, CREAM);
        }

        // Body text (VT323), typewriter + wrap in plum ink.
        let visible = &self.text[..self.shown.min(self.text.len())];
        draw_wrapped_col(visible, text_x, box_y + 40.0, w - text_x - 56.0, 34.0, 32, INK);

        // Choices
        for (i, (label, _)) in self.choices.iter().enumerate() {
            let r = self.choice_rect(i);
            let sel = i == self.choice_sel;
            if sel {
                draw_rectangle(r.x - 12.0, r.y - 4.0, r.w + 24.0, 34.0, PINK_PALE);
                ttext(">", r.x - 28.0, r.y + 30.0, f, 32, PINK);
            }
            ttext(label, r.x, r.y + 30.0, f, 32,
                if sel { INK } else { INK_SOFT });
        }

        // Advance hint
        let done = self.shown >= self.text.len();
        if done && self.choices.is_empty() {
            let blink = (get_time() * 2.0) as i32 % 2 == 0;
            if blink {
                ttext("z / click", w - 150.0, box_y + BOX_H - 14.0, f, 26, INK_SOFT);
            }
        }
    }
}

/// Line count a text will occupy after greedy wrap (mirrors draw_wrapped_col).
pub fn wrapped_line_count(text: &str, max_w: f32, size: u16) -> usize {
    let f = dialog_font();
    let mut count = 0usize;
    for para in text.split('\n') {
        let mut line = String::new();
        count += 1;
        for word in para.split(' ') {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
            if ttext_w(&candidate, f, size) > max_w && !line.is_empty() {
                count += 1;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
    }
    count
}

/// Greedy word wrap in the dialogue font, colored.
pub fn draw_wrapped_col(text: &str, x: f32, mut y: f32, max_w: f32, line_h: f32, size: u16, color: Color) {
    let f = dialog_font();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
            if ttext_w(&candidate, f, size) > max_w && !line.is_empty() {
                ttext(&line, x, y, f, size, color);
                y += line_h;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        ttext(&line, x, y, f, size, color);
        y += line_h;
    }
}

/// Diegetic quiz skins. Every quiz screen shares the dialogue tokens (VT323
/// body, Press Start 2P title chip, framed stage) but wears its biome's
/// materials: Candy keeps the plum veil, Aquarius dresses as pearl-and-sand,
/// Cosma sits on a neon circuit slab.
#[derive(Clone, Copy, PartialEq)]
pub enum Skin {
    Plum,
    PearlSand,
    NeonSlab,
}

impl Skin {
    fn veil(self) -> Color {
        match self {
            Skin::Plum => Color::from_rgba(36, 18, 56, 210),
            Skin::PearlSand => Color::from_rgba(7, 22, 36, 218),
            Skin::NeonSlab => Color::from_rgba(10, 6, 22, 228),
        }
    }
    /// (outer frame, inner frame)
    fn frame(self) -> (Color, Color) {
        match self {
            Skin::Plum => (Color::from_rgba(255, 214, 230, 110), Color::from_rgba(226, 106, 156, 70)),
            Skin::PearlSand => (Color::from_rgba(226, 240, 246, 150), Color::from_rgba(130, 214, 226, 80)),
            Skin::NeonSlab => (Color::from_rgba(150, 110, 255, 170), Color::from_rgba(110, 230, 255, 90)),
        }
    }
    /// title chip (fill, border, text)
    fn chip(self) -> (Color, Color, Color) {
        match self {
            Skin::Plum => (PINK, PLUM, CREAM),
            Skin::PearlSand => (
                Color::from_rgba(235, 242, 246, 255),
                Color::from_rgba(58, 118, 138, 255),
                Color::from_rgba(14, 48, 66, 255),
            ),
            Skin::NeonSlab => (
                Color::from_rgba(24, 16, 44, 255),
                Color::from_rgba(110, 230, 255, 255),
                Color::from_rgba(238, 246, 255, 255),
            ),
        }
    }
    /// selection band behind the chosen row
    fn hi(self) -> Color {
        match self {
            Skin::Plum => Color::from_rgba(226, 106, 156, 70),
            Skin::PearlSand => Color::from_rgba(190, 230, 240, 55),
            Skin::NeonSlab => Color::from_rgba(150, 110, 255, 60),
        }
    }
    fn caret(self) -> Color {
        match self {
            Skin::Plum => PINK,
            Skin::PearlSand => Color::from_rgba(140, 225, 235, 255),
            Skin::NeonSlab => Color::from_rgba(110, 230, 255, 255),
        }
    }
    /// (selected text, unselected text, disabled text, subtitle text)
    fn text(self) -> (Color, Color, Color, Color) {
        match self {
            Skin::Plum => (WHITE, Color::from_rgba(200, 200, 210, 255), Color::from_rgba(130, 130, 140, 255), Color::from_rgba(255, 236, 244, 255)),
            Skin::PearlSand => (
                Color::from_rgba(242, 250, 253, 255),
                Color::from_rgba(168, 198, 212, 255),
                Color::from_rgba(96, 118, 132, 255),
                Color::from_rgba(214, 238, 244, 255),
            ),
            Skin::NeonSlab => (
                Color::from_rgba(238, 246, 255, 255),
                Color::from_rgba(170, 182, 222, 255),
                Color::from_rgba(100, 108, 140, 255),
                Color::from_rgba(218, 230, 255, 255),
            ),
        }
    }
    pub fn edge(self) -> Color {
        self.frame().0
    }
    fn rule(self) -> Color {
        match self {
            Skin::Plum => Color::from_rgba(226, 106, 156, 150),
            Skin::PearlSand => Color::from_rgba(140, 225, 235, 140),
            Skin::NeonSlab => Color::from_rgba(110, 230, 255, 160),
        }
    }
}

/// Centered quiz option row (riddles + battles), in the given biome skin.
/// Long options auto-shrink so they never overflow the frame edge.
pub fn draw_option_row_skinned(i: usize, label: &str, y: f32, selected: bool, disabled: Option<bool>, skin: Skin) {
    let w = screen_width();
    let x = w * 0.5 - 300.0;
    let f = dialog_font();
    let (sel_c, dim_c, off_c, _) = skin.text();
    let color = match disabled {
        Some(true) => off_c,
        _ => if selected { sel_c } else { dim_c },
    };
    let tag = format!("{}. {}", i + 1, label);
    let mut size: u16 = 34;
    let tw0 = ttext_w(&tag, f, size);
    if tw0 > 640.0 {
        size = ((size as f32 * 640.0 / tw0) as u16).max(20);
    }
    if selected {
        draw_rectangle(x - 12.0, y - 24.0, 620.0, 34.0, skin.hi());
        ttext(">", x - 30.0, y + 2.0, f, 34, skin.caret());
    }
    ttext(&tag, x, y + 2.0, f, size, color);
}

/// Centered quiz option row, default plum skin.
pub fn draw_option_row(i: usize, label: &str, y: f32, selected: bool, disabled: Option<bool>) {
    draw_option_row_skinned(i, label, y, selected, disabled, Skin::Plum);
}

pub fn quiz_header(title: &str, sub: &str) {
    quiz_header_skinned(title, sub, Skin::Plum);
}

pub fn quiz_header_skinned(title: &str, sub: &str, skin: Skin) {
    let w = screen_width();
    let h = screen_height();
    let (f_out, f_in) = skin.frame();
    let (c_fill, c_border, c_text) = skin.chip();
    let (_, _, _, sub_c) = skin.text();
    // biome veil + hairline frame: every quiz screen sits on its biome's stage.
    // The frame starts below the subtitle so text never sits on the rule.
    draw_rectangle(0.0, 0.0, w, h, skin.veil());
    let fx = 24.0;
    let fy = 152.0;
    let fw = w - 48.0;
    let fh = h - 282.0;
    draw_rectangle_lines(fx, fy, fw, fh, 2.0, f_out);
    draw_rectangle_lines(fx + 6.0, fy + 6.0, fw - 12.0, fh - 12.0, 1.0, f_in);
    // signature details, one per skin, kept quiet:
    if skin == Skin::PearlSand {
        // a string of sea-pearls threaded along the top rule
        let ry = 153.0;
        for k in 0..9 {
            let px = w * 0.5 - 144.0 + k as f32 * 36.0;
            let gl = 0.8 + 0.2 * (get_time() as f32 * 1.4 + k as f32).sin();
            draw_circle(px, ry, 2.4, Color::new(0.94, 0.97, 0.99, gl));
            draw_circle(px - 0.8, ry - 0.8, 0.9, Color::from_rgba(255, 255, 255, 230));
        }
    } else if skin == Skin::NeonSlab {
        // circuit corner ticks, like the Neon Node panels inlaid in the walkway
        let c = Color::from_rgba(110, 230, 255, 220);
        let tick = 14.0;
        for (cx, cy, dx, dy) in [
            (fx, fy, 1.0, 1.0),
            (fx + fw, fy, -1.0, 1.0),
            (fx, fy + fh, 1.0, -1.0),
            (fx + fw, fy + fh, -1.0, -1.0),
        ] {
            let hx = if dx > 0.0 { cx } else { cx - tick };
            let hy = if dy > 0.0 { cy } else { cy - tick };
            draw_rectangle(hx, cy - 1.5, tick, 3.0, c);
            draw_rectangle(cx - 1.5, hy, 3.0, tick, c);
        }
    }
    // title: Press Start 2P chip when it fits, otherwise cream VT323
    let ts = tag_width(title, 18);
    let f = dialog_font();
    if ts < w - 240.0 {
        let cw = ts + 52.0;
        let cy = 34.0;
        let cx = w * 0.5 - cw / 2.0;
        draw_rectangle(cx, cy, cw, 38.0, c_fill);
        draw_rectangle_lines(cx - 2.0, cy - 2.0, cw + 4.0, 42.0, 2.0, c_border);
        draw_tag(title, w * 0.5 - ts / 2.0, cy + 27.0, 18, c_text);
    } else {
        ttext(title, w * 0.5 - ttext_w(title, f, 40) * 0.5, 60.0, f, 40, CREAM);
    }
    // rule + subtitle in the skin's tones
    draw_rectangle(w * 0.5 - 150.0, 96.0, 300.0, 2.0, skin.rule());
    if !sub.is_empty() {
        let sl = wrapped_line_count(sub, 760.0, 25) as f32;
        draw_wrapped_col(sub, w * 0.5 - 380.0, 124.0, 760.0, 25.0, 25, sub_c);
        let _ = sl;
    }
}

/// Like `draw_wrapped_col` but treats `*word*` as emphasis (accent colour) and
/// prints the word without its asterisks. The maze questions carry the
/// script's own emphasis markers, which used to show up literally.
pub fn draw_wrapped_rich(text: &str, x0: f32, mut y: f32, max_w: f32, line_h: f32, size: u16, base: Color, accent: Color) {
    let f = dialog_font();
    let max_x = x0 + max_w;
    let space = ttext_w(" ", f, size);
    for para in text.split('\n') {
        let mut x = x0;
        for word in para.split(' ') {
            // emphasis can wrap punctuation (`*west*,`), so strip the markers
            // wherever they sit instead of only on a bare *word*
            let emph = word.contains('*');
            let clean: String = word.chars().filter(|ch| *ch != '*').collect();
            let ww = ttext_w(&clean, f, size);
            if x + ww > max_x && x > x0 {
                x = x0;
                y += line_h;
            }
            ttext(&clean, x, y, f, size, if emph { accent } else { base });
            x += ww + space;
        }
        y += line_h;
    }
}

// --- item art --------------------------------------------------------------
// Everything here is drawn on a small integer grid (a "pixel" is `k` screen
// pixels) so it keeps the hard edges of the pixel fonts at any size instead of
// turning into a flat vector rectangle. The old gift was a red draw_rectangle.

/// The Family Heirloom storybook. `bottom` is the base of the book.
/// The grid is 32 wide by 40 tall.
pub fn draw_storybook(cx: f32, bottom: f32, k: f32, glow: f32) {
    let left = cx - 16.0 * k;
    let top = bottom - 40.0 * k;
    let r = |x: f32, y: f32, w: f32, h: f32, c: Color| {
        draw_rectangle(left + x * k, top + y * k, w * k, h * k, c);
    };
    let gold = Color::from_rgba(238, 193, 100, 255);
    let gold_lo = Color::from_rgba(170, 124, 48, 255);
    let leather = Color::from_rgba(82, 40, 116, 255);
    let leather_hi = Color::from_rgba(112, 60, 154, 255);
    let leather_lo = Color::from_rgba(52, 22, 78, 255);
    let page = Color::from_rgba(251, 240, 214, 255);
    let page_lo = Color::from_rgba(212, 190, 152, 255);

    // warm halo so the book reads as the hero object, not a UI sprite
    draw_circle(cx + 2.0 * k, bottom - 20.0 * k, 24.0 * k,
        Color::new(1.0, 0.84, 0.48, 0.05 + glow * 0.09));

    // page block sits proud of the cover on the right, with page edges
    r(9.0, 5.0, 20.0, 32.0, page);
    r(9.0, 5.0, 20.0, 1.0, page_lo);
    for i in 0..5 {
        r(11.0, 9.0 + i as f32 * 5.5, 16.0, 0.8, page_lo);
    }
    r(28.0, 5.0, 1.0, 32.0, page_lo);

    // covers
    r(4.0, 2.0, 23.0, 36.0, leather_lo);
    r(4.0, 4.0, 20.0, 33.0, leather);
    // spine with raised bands
    r(6.0, 4.0, 4.0, 33.0, leather_hi);
    r(6.0, 4.0, 1.0, 33.0, Color::from_rgba(140, 84, 184, 255));
    r(6.0, 6.0, 4.0, 2.0, gold_lo);
    r(6.0, 32.0, 4.0, 2.0, gold_lo);

    // gilt frame and emblem
    r(13.0, 8.0, 1.0, 25.0, gold_lo);
    r(25.0, 8.0, 1.0, 25.0, gold_lo);
    r(13.0, 8.0, 13.0, 1.0, gold_lo);
    r(13.0, 32.0, 13.0, 1.0, gold_lo);
    draw_poly(cx + 4.0 * k, top + 19.0 * k, 4, 5.0 * k, 0.78, gold);
    draw_poly(cx + 4.0 * k, top + 19.0 * k, 4, 2.4 * k, 0.78, leather);
    draw_circle(cx + 4.0 * k, top + 19.0 * k, 0.9 * k, gold);

    // clasp and ribbon bookmark
    r(26.0, 17.0, 4.0, 6.0, gold);
    r(26.0, 19.0, 4.0, 2.0, gold_lo);
    r(19.0, 36.0, 3.0, 4.0, Color::from_rgba(214, 74, 132, 255));
    r(19.0, 36.0, 3.0, 1.0, Color::from_rgba(246, 130, 176, 255));
}

/// One of the three elixir vials in the HUD. `filled` is 0..1.
/// The grid is 10 wide by 20 tall.
pub fn draw_vial(cx: f32, bottom: f32, h: f32, filled: f32) {
    let k = h / 20.0;
    let top = bottom - h;
    let left = cx - 5.0 * k;
    let glass = Color::from_rgba(240, 246, 252, 225);
    let glass_lo = Color::from_rgba(150, 128, 176, 200);
    let f = filled.clamp(0.0, 1.0);

    // cork
    draw_rectangle(left + 3.0 * k, top, 4.0 * k, 3.0 * k, Color::from_rgba(158, 108, 66, 255));
    draw_rectangle(left + 3.0 * k, top, 4.0 * k, 1.0 * k, Color::from_rgba(196, 142, 92, 255));
    // neck + body silhouette
    draw_rectangle(left + 2.0 * k, top + 3.0 * k, 6.0 * k, 3.0 * k, glass);
    draw_rectangle(left + 1.0 * k, top + 6.0 * k, 8.0 * k, 11.0 * k, glass);
    draw_rectangle(left + 2.0 * k, top + 17.0 * k, 6.0 * k, 1.0 * k, glass);
    draw_rectangle(left + 3.0 * k, top + 18.0 * k, 4.0 * k, 1.0 * k, glass_lo);
    draw_rectangle(left, top + 6.0 * k, 1.0 * k, 11.0 * k, glass_lo);
    draw_rectangle(left + 9.0 * k, top + 6.0 * k, 1.0 * k, 11.0 * k, glass_lo);

    // glowing contents, filled from the base up
    if f > 0.01 {
        let body_top = top + 6.0 * k;
        let body_h = 13.0 * k * f;
        let y = top + 19.0 * k - body_h;
        draw_rectangle(left + 1.0 * k, y, 8.0 * k, body_h, Color::from_rgba(255, 150, 200, 235));
        draw_rectangle(left + 1.0 * k, y, 8.0 * k, 1.0 * k, Color::from_rgba(255, 214, 236, 255));
        draw_rectangle(left + 1.0 * k, y, 2.0 * k, body_h, Color::from_rgba(255, 196, 226, 235));
        // a spark near the surface
        draw_circle(left + 6.0 * k, y + 2.0 * k, 0.8 * k, Color::from_rgba(255, 246, 252, 220));
        let _ = body_top;
    }
    // specular stripe down the glass
    draw_rectangle(left + 2.5 * k, top + 7.0 * k, 1.0 * k, 8.0 * k, Color::from_rgba(255, 255, 255, 190));
}
