//! Rich pixel-art RPG UI: Kenney Fantasy 9-slice borders, 48x48 animated bust
//! portraits, pixel drop-shadow text helpers, Field Notes modal, and Pause UI.

use std::sync::OnceLock;
use macroquad::prelude::*;

use crate::art::Textures;
use crate::game::script::Biome;
use crate::game::Progress;

const BOX_H: f32 = 192.0;
const BOX_MARGIN: f32 = 22.0;

// --- pixel fonts -----------------------------------------------------------

static F_DIALOG: OnceLock<Font> = OnceLock::new();
static F_TAG: OnceLock<Font> = OnceLock::new();

pub async fn ensure_fonts() {
    if F_DIALOG.get().is_some() && F_TAG.get().is_some() {
        return;
    }
    if let Ok(bytes) = crate::embedded::load_bytes("assets/fonts/VT323-Regular.ttf").await {
        if let Ok(mut f) = load_ttf_font_from_bytes(&bytes) {
            prewarm_font(&mut f, &[20, 22, 24, 26, 28]);
            let _ = F_DIALOG.set(f);
        }
    }
    if let Ok(bytes) = crate::embedded::load_bytes("assets/fonts/PressStart2P-Regular.ttf").await {
        if let Ok(mut f) = load_ttf_font_from_bytes(&bytes) {
            prewarm_font(&mut f, &[10, 11, 12, 13, 14, 15, 16, 18, 20, 28]);
            let _ = F_TAG.set(f);
        }
    }
    assert!(F_DIALOG.get().is_some(), "VT323 font failed to load");
    assert!(F_TAG.get().is_some(), "Press Start 2P font failed to load");
}

fn prewarm_font(font: &mut Font, sizes: &[u16]) {
    let chars: Vec<char> = (' '..='~').chain("—…“”’•".chars()).collect();
    for &sz in sizes {
        font.populate_font_cache(&chars, sz);
    }
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
    draw_text_ex(
        text,
        x.round(),
        y.round(),
        TextParams {
            font: Some(font),
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

pub fn draw_tag_shadow(text: &str, x: f32, y: f32, size: u16, color: Color) {
    ttext(text, x + 2.0, y + 2.0, tag_font(), size, Color::new(0.03, 0.02, 0.07, color.a * 0.85));
    ttext(text, x, y, tag_font(), size, color);
}

pub fn draw_dlg_shadow(text: &str, x: f32, y: f32, size: u16, color: Color) {
    ttext(text, x + 2.0, y + 2.0, dialog_font(), size, Color::new(0.03, 0.02, 0.07, color.a * 0.85));
    ttext(text, x, y, dialog_font(), size, color);
}

pub fn dlg_width(text: &str, size: u16) -> f32 {
    ttext_w(text, dialog_font(), size)
}

pub fn tag_width(text: &str, size: u16) -> f32 {
    ttext_w(text, tag_font(), size)
}

// --- Biome UI Palettes & 9-slice Fantasy UI Borders -------------------------

#[derive(Clone, Copy)]
pub struct Skin {
    pub bg: Color,
    pub fill: Color,
    pub border: Color,
    pub ink: Color,
    pub muted: Color,
    pub accent: Color,
    pub sel: Color,
    pub chip: Color,
    pub chip_ink: Color,
}

impl Skin {
    pub fn for_biome(b: Biome) -> Self {
        match b {
            Biome::Kelp => Self {
                bg: Color::from_rgba(12, 34, 42, 238),
                fill: Color::from_rgba(18, 48, 56, 240),
                border: Color::from_rgba(104, 232, 184, 255),
                ink: Color::from_rgba(238, 250, 244, 255),
                muted: Color::from_rgba(154, 206, 192, 255),
                accent: Color::from_rgba(255, 218, 102, 255),
                sel: Color::from_rgba(32, 82, 86, 245),
                chip: Color::from_rgba(28, 76, 82, 255),
                chip_ink: Color::from_rgba(146, 246, 206, 255),
            },
            Biome::Coral => Self {
                bg: Color::from_rgba(12, 28, 64, 238),
                fill: Color::from_rgba(16, 40, 84, 240),
                border: Color::from_rgba(106, 224, 252, 255),
                ink: Color::from_rgba(240, 248, 255, 255),
                muted: Color::from_rgba(156, 198, 236, 255),
                accent: Color::from_rgba(255, 190, 96, 255),
                sel: Color::from_rgba(32, 72, 134, 245),
                chip: Color::from_rgba(24, 62, 114, 255),
                chip_ink: Color::from_rgba(136, 236, 255, 255),
            },
            Biome::Slums => Self {
                bg: Color::from_rgba(16, 12, 36, 242),
                fill: Color::from_rgba(22, 18, 48, 244),
                border: Color::from_rgba(156, 124, 246, 255),
                ink: Color::from_rgba(240, 236, 255, 255),
                muted: Color::from_rgba(162, 152, 210, 255),
                accent: Color::from_rgba(110, 244, 240, 255),
                sel: Color::from_rgba(46, 34, 88, 245),
                chip: Color::from_rgba(36, 26, 70, 255),
                chip_ink: Color::from_rgba(122, 246, 242, 255),
            },
            Biome::Home => Self {
                bg: Color::from_rgba(34, 20, 40, 238),
                fill: Color::from_rgba(46, 28, 52, 242),
                border: Color::from_rgba(240, 190, 110, 255),
                ink: Color::from_rgba(252, 244, 232, 255),
                muted: Color::from_rgba(208, 178, 156, 255),
                accent: Color::from_rgba(255, 212, 112, 255),
                sel: Color::from_rgba(78, 46, 80, 245),
                chip: Color::from_rgba(64, 38, 66, 255),
                chip_ink: Color::from_rgba(255, 224, 152, 255),
            },
        }
    }
}

/// Draw crisp 9-slice panel using Kenney Fantasy UI Borders (`48x48` 16px grid).
pub fn draw_nine_slice(tex: &Textures, x: f32, y: f32, w: f32, h: f32, fill_tint: Color, border_tint: Color) {
    let x = x.round();
    let y = y.round();
    let w = w.round().max(28.0);
    let h = h.round().max(28.0);
    let c = 14.0f32.min(w * 0.35).min(h * 0.35);
    let s = 16.0;

    let draw_cell = |t: &Texture2D, sx: f32, sy: f32, dx: f32, dy: f32, dw: f32, dh: f32, col: Color| {
        if dw <= 0.0 || dh <= 0.0 {
            return;
        }
        draw_texture_ex(t, dx, dy, col, DrawTextureParams {
            source: Some(Rect::new(sx, sy, s, s)),
            dest_size: Some(vec2(dw, dh)),
            ..Default::default()
        });
    };

    // Subtle ground drop shadow
    draw_rectangle(x + 4.0, y + 6.0, w, h, Color::new(0.01, 0.01, 0.03, 0.45));

    // Inner fill quad
    draw_rectangle(x + 6.0, y + 6.0, w - 12.0, h - 12.0, fill_tint);
    draw_texture_ex(&tex.ui.panel_fill, x + 8.0, y + 8.0, Color::new(fill_tint.r * 1.15, fill_tint.g * 1.15, fill_tint.b * 1.15, 0.25), DrawTextureParams {
        dest_size: Some(vec2(w - 16.0, h - 16.0)),
        ..Default::default()
    });

    // 9-slice border frame
    let b = &tex.ui.panel_border;
    let mw = (w - c * 2.0).max(0.0);
    let mh = (h - c * 2.0).max(0.0);
    // Corners
    draw_cell(b, 0.0, 0.0, x, y, c, c, border_tint);
    draw_cell(b, 32.0, 0.0, x + w - c, y, c, c, border_tint);
    draw_cell(b, 0.0, 32.0, x, y + h - c, c, c, border_tint);
    draw_cell(b, 32.0, 32.0, x + w - c, y + h - c, c, c, border_tint);
    // Edges
    draw_cell(b, 16.0, 0.0, x + c, y, mw, c, border_tint);
    draw_cell(b, 16.0, 32.0, x + c, y + h - c, mw, c, border_tint);
    draw_cell(b, 0.0, 16.0, x, y + c, c, mh, border_tint);
    draw_cell(b, 32.0, 16.0, x + w - c, y + c, c, mh, border_tint);
}

pub fn draw_panel(tex: &Textures, x: f32, y: f32, w: f32, h: f32, skin: &Skin) {
    draw_nine_slice(tex, x, y, w, h, skin.bg, skin.border);
}

/// Interactive pill/panel button with hover/press feedback. Returns true if clicked this frame.
pub fn draw_btn(tex: &Textures, x: f32, y: f32, w: f32, h: f32, label: &str, skin: &Skin, active: bool) -> bool {
    let (mx, my) = mouse_position();
    let hov = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let down = hov && is_mouse_button_down(MouseButton::Left);
    let dy = if down { 1.5 } else if hov { -1.0 } else { 0.0 };
    let fill = if active || hov { skin.sel } else { skin.chip };
    let border = if active || hov { skin.accent } else { skin.border };
    draw_nine_slice(tex, x, y + dy, w, h, fill, border);
    let size = if tag_width(label, 12) <= w - 16.0 { 12 } else { 10 };
    let tw = tag_width(label, size);
    let col = if active || hov { skin.accent } else { skin.ink };
    draw_tag_shadow(label, x + (w - tw) * 0.5, y + dy + h * 0.5 + (size as f32 * 0.42), size, col);
    hov && is_mouse_button_pressed(MouseButton::Left)
}

/// Interactive square icon button. Returns true if clicked this frame.
pub fn draw_icon_only_btn(
    tex: &Textures,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ico: &Texture2D,
    skin: &Skin,
    active: bool,
) -> bool {
    let (mx, my) = mouse_position();
    let hov = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let down = hov && is_mouse_button_down(MouseButton::Left);
    let dy = if down { 1.5 } else if hov { -1.0 } else { 0.0 };
    let fill = if active || hov { skin.sel } else { skin.chip };
    let border = if active || hov { skin.accent } else { skin.border };
    draw_nine_slice(tex, x, y + dy, w, h, fill, border);
    draw_texture_ex(ico, x + (w - 22.0) * 0.5, y + dy + (h - 22.0) * 0.5, WHITE, DrawTextureParams {
        dest_size: Some(vec2(22.0, 22.0)),
        ..Default::default()
    });
    hov && is_mouse_button_pressed(MouseButton::Left)
}

/// Interactive pill/panel button featuring a pixel icon cleanly padded beside its label.
pub fn draw_btn_icon(
    tex: &Textures,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ico: &Texture2D,
    label: &str,
    skin: &Skin,
    active: bool,
) -> bool {
    let (mx, my) = mouse_position();
    let hov = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let down = hov && is_mouse_button_down(MouseButton::Left);
    let dy = if down { 1.5 } else if hov { -1.0 } else { 0.0 };
    let fill = if active || hov { skin.sel } else { skin.chip };
    let border = if active || hov { skin.accent } else { skin.border };
    draw_nine_slice(tex, x, y + dy, w, h, fill, border);
    let size = if tag_width(label, 12) + 26.0 <= w - 16.0 { 12 } else { 10 };
    let tw = tag_width(label, size);
    let total = (18.0 + 8.0 + tw).min(w - 14.0);
    let x0 = (x + (w - total) * 0.5).max(x + 8.0);
    draw_texture_ex(ico, x0, y + dy + (h - 18.0) * 0.5, WHITE, DrawTextureParams {
        dest_size: Some(vec2(18.0, 18.0)),
        ..Default::default()
    });
    let col = if active || hov { skin.accent } else { skin.ink };
    draw_tag_shadow(label, x0 + 26.0, y + dy + h * 0.5 + (size as f32 * 0.42), size, col);
    hov && is_mouse_button_pressed(MouseButton::Left)
}

// --- DialogueBox (Undertale/Stardew style bust portrait + rich text) -------

pub struct DialogueBox {
    pub open: bool,
    speaker: String,
    expr: String,
    text: String,
    shown: usize,
    timer: f32,
    open_t: f32,
    portrait: Option<Texture2D>,
    choices: Vec<(String, String)>,
    choice_sel: usize,
    pub force_advance: bool,
}

impl DialogueBox {
    pub fn new() -> Self {
        Self {
            open: false,
            speaker: String::new(),
            expr: "normal".to_string(),
            text: String::new(),
            shown: 0,
            timer: 0.0,
            open_t: 0.0,
            portrait: None,
            choices: Vec::new(),
            choice_sel: 0,
            force_advance: false,
        }
    }

    pub fn say(&mut self, who: &str, text: &str, tex: &Textures) {
        self.say_expr(who, "normal", text, tex);
    }

    pub fn say_expr(&mut self, who: &str, expr: &str, text: &str, tex: &Textures) {
        if !self.open {
            self.open_t = 0.0;
        }
        self.open = true;
        self.speaker = who.to_string();
        self.expr = expr.to_string();
        self.text = text.to_string();
        self.shown = 0;
        self.timer = 0.0;
        self.portrait = tex.portrait_expr(who, expr).cloned();
        self.choices.clear();
    }

    pub fn ask(&mut self, who: &str, text: &str, choices: &[(&str, &str)], tex: &Textures) {
        self.ask_expr(who, "normal", text, choices, tex);
    }

    pub fn ask_expr(&mut self, who: &str, expr: &str, text: &str, choices: &[(&str, &str)], tex: &Textures) {
        self.say_expr(who, expr, text, tex);
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

    pub fn chars_shown(&self) -> usize {
        if self.open { self.shown } else { 0 }
    }

    pub fn is_finished(&self) -> bool {
        self.open && self.shown >= self.text.len()
    }

    pub fn update(&mut self, dt: f32) -> Option<String> {
        if !self.open {
            return None;
        }
        self.open_t = (self.open_t + dt * 7.5).min(1.0);
        if self.shown < self.text.len() {
            self.timer += dt;
            let want = (self.timer * 52.0) as usize;
            if want > self.shown {
                self.shown = want.min(self.text.len());
            }
        }
        let done = self.shown >= self.text.len();
        let forced = self.force_advance;
        let advance = forced
            || is_key_pressed(KeyCode::Z)
            || is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || is_mouse_button_pressed(MouseButton::Left);
        self.force_advance = false;

        if !self.choices.is_empty() && done {
            if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                self.choice_sel = self.choice_sel.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                self.choice_sel = (self.choice_sel + 1).min(self.choices.len() - 1);
            }
            for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].iter().enumerate() {
                if is_key_pressed(*key) && k < self.choices.len() {
                    self.choice_sel = k;
                }
            }
            let (mx, my) = mouse_position();
            let (bx, by, bw, _) = self.panel_rect();
            let list_x = bx + 186.0;
            let list_w = bw - 216.0;
            let base_y = by + BOX_H - 18.0 - self.choices.len() as f32 * 26.0;
            for i in 0..self.choices.len() {
                let cy = base_y + i as f32 * 26.0;
                if mx >= list_x && mx <= list_x + list_w && my >= cy && my <= cy + 24.0 {
                    self.choice_sel = i;
                    if is_mouse_button_pressed(MouseButton::Left) {
                        let out = self.choices[i].1.clone();
                        self.open = false;
                        return Some(out);
                    }
                }
            }
            if forced || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Space) {
                let out = self.choices[self.choice_sel].1.clone();
                self.open = false;
                return Some(out);
            }
            return None;
        }

        if advance {
            if !done {
                self.shown = self.text.len();
            } else {
                self.open = false;
                return Some(String::new());
            }
        }
        None
    }

    fn panel_rect(&self) -> (f32, f32, f32, f32) {
        let w = screen_width();
        let h = screen_height();
        let bw = (w - BOX_MARGIN * 2.0).min(960.0);
        let bx = (w - bw) * 0.5;
        let ease = 1.0 - (1.0 - self.open_t).powi(3);
        let by = h - BOX_H - BOX_MARGIN + (1.0 - ease) * 20.0;
        (bx, by, bw, BOX_H)
    }

    pub fn draw(&self, tex: &Textures, skin: &Skin) {
        if !self.open {
            return;
        }
        let (bx, by, bw, bh) = self.panel_rect();
        draw_panel(tex, bx, by, bw, bh, skin);

        // Portrait well on left (48x48 crisp pixel art rendered at 3x = 144x144)
        let px = bx + 20.0;
        let py = by + 24.0;
        let psize = 144.0;
        draw_nine_slice(tex, px - 4.0, py - 4.0, psize + 8.0, psize + 8.0, skin.chip, skin.border);
        if let Some(p) = &self.portrait {
            let talking = self.shown < self.text.len();
            let bob = if talking { (self.timer * 18.0).sin().abs() * -3.0 } else { 0.0 };
            draw_texture_ex(p, px, py + bob, WHITE, DrawTextureParams {
                dest_size: Some(vec2(psize, psize)),
                ..Default::default()
            });
        }

        // Speaker name badge perched cleanly on top edge
        if !self.speaker.is_empty() {
            let nw = tag_width(&self.speaker, 12) + 28.0;
            draw_nine_slice(tex, bx + 24.0, by - 16.0, nw.max(120.0), 32.0, skin.chip, skin.accent);
            draw_tag_shadow(&self.speaker, bx + 38.0, by + 5.0, 12, skin.accent);
        }

        // Spoken text revealed char-by-char (respecting UTF-8 boundaries safely)
        let mut end = self.shown.min(self.text.len());
        while end > 0 && !self.text.is_char_boundary(end) {
            end -= 1;
        }
        let sub = &self.text[..end];
        let tx = bx + 186.0;
        let max_w = bw - 216.0;
        draw_wrapped_rich(sub, tx, by + 48.0, max_w, 28, 30.0, skin.ink, skin.accent);

        if self.shown >= self.text.len() {
            if !self.choices.is_empty() {
                let row_h = 28.0;
                let base_y = by + bh - 14.0 - self.choices.len() as f32 * row_h;
                for (i, (label, _)) in self.choices.iter().enumerate() {
                    let cy = base_y + i as f32 * row_h;
                    let sel = i == self.choice_sel;
                    let fill = if sel { skin.sel } else { skin.chip };
                    let bord = if sel { skin.accent } else { skin.border };
                    draw_nine_slice(tex, tx - 6.0, cy - 2.0, max_w, row_h - 3.0, fill, bord);
                    if sel {
                        draw_rectangle(tx, cy + 2.0, 3.0, row_h - 11.0, skin.accent);
                    }
                    draw_dlg_shadow(label, tx + 10.0, cy + 16.0, 21, if sel { skin.accent } else { skin.ink });
                }
            } else {
                let bob = (get_time() as f32 * 4.5).sin() * 2.5;
                let prompt = "Enter / Click";
                let pw = tag_width(prompt, 10);
                let qx = bx + bw - pw - 48.0;
                draw_texture_ex(&tex.ui.quill, qx, by + bh - 31.0 + bob, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(18.0, 18.0)),
                    ..Default::default()
                });
                draw_tag_shadow(prompt, qx + 24.0, by + bh - 18.0 + bob, 10, skin.accent);
            }
        }
    }
}

// --- Rich Text Wrapping & Option Cards -------------------------------------

pub fn draw_wrapped_rich(text: &str, x: f32, y: f32, max_w: f32, size: u16, line_h: f32, base: Color, accent: Color) {
    draw_wrapped_inner(text, x, y, max_w, size, line_h, base, accent);
}

pub fn draw_wrapped_col(text: &str, x: f32, y: f32, max_w: f32, size: u16, line_h: f32, color: Color) {
    draw_wrapped_inner(text, x, y, max_w, size, line_h, color, color);
}

fn strip_emph(w: &str) -> (std::borrow::Cow<'_, str>, bool) {
    if w.contains('*') {
        (std::borrow::Cow::Owned(w.replace('*', "")), true)
    } else {
        (std::borrow::Cow::Borrowed(w), false)
    }
}

fn draw_wrapped_inner(text: &str, x0: f32, mut y: f32, max_w: f32, size: u16, line_h: f32, base: Color, accent: Color) {
    let f = dialog_font();
    let space = ttext_w(" ", f, size);
    for para in text.split('\n') {
        if para.is_empty() {
            y += line_h * 0.65;
            continue;
        }
        let mut x = x0;
        for raw_word in para.split_whitespace() {
            let (clean, emph) = strip_emph(raw_word);
            let ww = ttext_w(&clean, f, size);
            if x > x0 && x + ww > x0 + max_w {
                x = x0;
                y += line_h;
            }
            let col = if emph { accent } else { base };
            ttext(&clean, x + 2.0, y + 2.0, f, size, Color::new(0.02, 0.02, 0.05, col.a * 0.85));
            ttext(&clean, x, y, f, size, col);
            x += ww + space;
        }
        y += line_h;
    }
}

pub fn wrapped_line_count(text: &str, max_w: f32, size: u16) -> usize {
    let f = dialog_font();
    let space = ttext_w(" ", f, size);
    let mut lines = 0usize;
    for para in text.split('\n') {
        lines += 1;
        let mut x = 0.0f32;
        for raw in para.split_whitespace() {
            let (clean, _) = strip_emph(raw);
            let ww = ttext_w(&clean, f, size);
            if x > 0.0 && x + ww > max_w {
                lines += 1;
                x = ww + space;
            } else {
                x += ww + space;
            }
        }
    }
    lines.max(1)
}

pub fn quiz_header_skinned(tex: &Textures, title: &str, x: f32, y: f32, w: f32, skin: &Skin) {
    let chip_w = (tag_width(title, 11) + 32.0).min(w);
    draw_nine_slice(tex, x, y - 4.0, chip_w.max(200.0), 36.0, skin.chip, skin.border);
    draw_tag_shadow(title, x + 16.0, y + 20.0, 11, skin.chip_ink);
    let stem_w = (w - 28.0).max(16.0);
    draw_texture_ex(&tex.ui.divider, x, y + 35.0, skin.border, DrawTextureParams {
        source: Some(Rect::new(0.0, 0.0, 72.0, 22.0)),
        dest_size: Some(vec2(stem_w, 16.0)),
        ..Default::default()
    });
    draw_texture_ex(&tex.ui.divider, x + stem_w, y + 35.0, skin.accent, DrawTextureParams {
        source: Some(Rect::new(72.0, 0.0, 24.0, 22.0)),
        dest_size: Some(vec2(26.0, 16.0)),
        ..Default::default()
    });
}

fn draw_option_label(x: f32, y: f32, w: f32, h: f32, label: &str, col: Color, accent: Color) {
    let mut size = 24u16;
    let mut lh = 24.0f32;
    let mut lines = wrapped_line_count(label, w - 42.0, size) as f32;
    if lines * lh > h - 12.0 {
        size = 20;
        lh = 20.0;
        lines = wrapped_line_count(label, w - 42.0, size) as f32;
    }
    let start_y = y + (h - lines * lh) * 0.5 + (size as f32 * 0.72);
    draw_wrapped_rich(label, x + 20.0, start_y, w - 40.0, size, lh, col, accent);
}

pub fn draw_option_row_skinned(
    tex: &Textures,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    selected: bool,
    skin: &Skin,
) -> bool {
    let (mx, my) = mouse_position();
    let hov = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let fill = if selected || hov { skin.sel } else { skin.chip };
    let border = if selected { skin.accent } else if hov { skin.chip_ink } else { skin.border };
    draw_nine_slice(tex, x, y, w, h, fill, border);
    if selected {
        draw_rectangle(x + 7.0, y + 9.0, 4.0, (h - 18.0).max(8.0), skin.accent);
    }
    let txt_col = if selected { skin.accent } else { skin.ink };
    draw_option_label(x, y, w, h, label, txt_col, skin.accent);
    hov && is_mouse_button_pressed(MouseButton::Left)
}

/// Non-interactive answer-reveal option row used once a one-try riddle is locked.
pub fn draw_option_row_locked(
    tex: &Textures,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    is_correct: bool,
    is_wrong_pick: bool,
    skin: &Skin,
) {
    let (fill, border, bar_col, txt_col) = if is_correct {
        (skin.sel, Color::new(0.45, 0.94, 0.65, 1.0), Some(Color::new(0.45, 0.94, 0.65, 1.0)), skin.accent)
    } else if is_wrong_pick {
        (Color::new(0.28, 0.11, 0.16, 0.95), Color::new(0.98, 0.45, 0.52, 1.0), Some(Color::new(0.98, 0.45, 0.52, 1.0)), Color::new(1.0, 0.68, 0.72, 1.0))
    } else {
        (skin.chip, skin.border, None, skin.muted)
    };
    draw_nine_slice(tex, x, y, w, h, fill, border);
    if let Some(bc) = bar_col {
        draw_rectangle(x + 7.0, y + 9.0, 4.0, (h - 18.0).max(8.0), bc);
    }
    draw_option_label(x, y, w, h, label, txt_col, skin.accent);
}

// --- Field Notes & Pause Modals ---------------------------------------------

pub fn draw_field_notes(tex: &Textures, prog: &Progress, biome: Biome, skin: &Skin) -> bool {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.01, 0.01, 0.04, 0.72));

    let pw = 780.0f32.min(w - 48.0);
    let ph = 536.0f32.min(h - 48.0);
    let px = (w - pw) * 0.5;
    let py = (h - ph) * 0.5;
    draw_panel(tex, px, py, pw, ph, skin);

    draw_texture_ex(&tex.ui.book, px + 26.0, py + 24.0, WHITE, DrawTextureParams {
        dest_size: Some(vec2(26.0, 26.0)),
        ..Default::default()
    });
    draw_tag_shadow("FIELD NOTES & JOURNAL", px + 60.0, py + 43.0, 16, skin.accent);

    // Objective summary
    let goal = match biome {
        Biome::Kelp => "Solve at least 1 Kelp Riddle, mend The Lone Stone Throne Story Gate, and confront Queen Crab.",
        Biome::Coral => "Gather Blue Seashroom, Tentacle Anemone & True Seaberry (use NPC N/E/W clues!), solve a Coral Riddle, ride Tow-Tou through the Cave Maze.",
        Biome::Slums => "Answer Tenebris's Somber Riddles, conquer the Temple of Figurate Algae, catch stars & light the Rainbow Road.",
        Biome::Home => "Deliver the Elixir of Literacy to heal Nadia!",
    };
    draw_tag_shadow("CURRENT OBJECTIVE:", px + 28.0, py + 82.0, 11, skin.chip_ink);
    draw_wrapped_col(goal, px + 28.0, py + 108.0, pw - 56.0, 22, 24.0, skin.ink);

    // Elixir vial status row
    draw_tag_shadow("ELIXIR VIALS:", px + 28.0, py + 174.0, 11, skin.chip_ink);
    let vials = [
        (&tex.ui.vial_red, "Pearl of Clarity", prog.elixir >= 1),
        (&tex.ui.vial_green, "Algae Synthesis", prog.elixir >= 2),
        (&tex.ui.vial_blue, "Elixir of Literacy", prog.elixir >= 3),
    ];
    let card_w = (pw - 56.0 - 20.0) / 3.0;
    for (i, (ico, lbl, got)) in vials.iter().enumerate() {
        let vx = px + 28.0 + i as f32 * (card_w + 10.0);
        let vy = py + 186.0;
        draw_nine_slice(tex, vx, vy, card_w, 54.0, skin.chip, if *got { skin.accent } else { skin.border });
        let img = if *got { *ico } else { &tex.ui.vial_empty };
        draw_texture_ex(img, vx + 10.0, vy + 13.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(28.0, 28.0)),
            ..Default::default()
        });
        draw_dlg_shadow(lbl, vx + 44.0, vy + 34.0, 20, if *got { skin.accent } else { skin.muted });
    }

    // Aquarius clues & ingredients
    draw_tag_shadow("AQUARIUS CLUES & REAGENTS:", px + 28.0, py + 272.0, 11, skin.chip_ink);
    let items = [
        (&tex.props.item_seashroom, "Blue Seashroom", prog.items[0]),
        (&tex.props.item_anemone, "Tentacle Anemone", prog.items[1]),
        (&tex.props.item_seaberry, "True Seaberry", prog.items[2]),
    ];
    for (i, (ico, name, got)) in items.iter().enumerate() {
        let ix = px + 28.0 + i as f32 * (card_w + 10.0);
        let iy = py + 284.0;
        draw_nine_slice(tex, ix, iy, card_w, 42.0, skin.chip, if *got { skin.accent } else { skin.border });
        let tint = if *got { WHITE } else { Color::new(0.45, 0.48, 0.58, 0.7) };
        draw_texture_ex(*ico, ix + 10.0, iy + 9.0, tint, DrawTextureParams {
            dest_size: Some(vec2(24.0, 24.0)),
            ..Default::default()
        });
        draw_dlg_shadow(
            name,
            ix + 40.0,
            iy + 27.0,
            20,
            if *got { skin.accent } else { skin.muted },
        );
    }

    let mut cy = py + 354.0;
    let footer_y = py + ph - 76.0;
    if prog.clues.is_empty() {
        draw_wrapped_col("Talk to Gillbert, Scalleon, Market Lady, and examine the Neptune Statue.", px + 28.0, cy, pw - 56.0, 21, 23.0, skin.muted);
    } else {
        for c in &prog.clues {
            let line = format!("* {}", c);
            let lines = wrapped_line_count(&line, pw - 56.0, 21);
            if cy + lines as f32 * 22.0 > footer_y {
                break;
            }
            draw_wrapped_col(&line, px + 28.0, cy, pw - 56.0, 21, 22.0, skin.ink);
            cy += lines as f32 * 22.0 + 4.0;
        }
    }

    // Riddle summary footer
    draw_tag_shadow(
        &format!("Riddles: {} right ({}/8 seen)   Puzzles: {}/4",
            prog.total_riddles_right(),
            prog.total_riddles_seen(),
            prog.gate_solved as u8 + prog.maze_solved as u8 + prog.algae_solved as u8 + prog.rainbow_solved as u8,
        ),
        px + 28.0,
        py + ph - 36.0,
        11,
        skin.accent,
    );

    draw_btn(tex, px + pw - 166.0, py + ph - 58.0, 140.0, 40.0, "CLOSE [Tab]", skin, false)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    None,
    Resume,
    ToggleMute,
    VolDown,
    VolUp,
    RestartBiome,
    QuitTitle,
}

pub fn draw_pause_overlay(tex: &Textures, muted: bool, vol: f32, skin: &Skin) -> PauseAction {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.01, 0.01, 0.04, 0.72));

    let pw = 440.0;
    let ph = 398.0;
    let px = (w - pw) * 0.5;
    let py = (h - ph) * 0.5;
    draw_panel(tex, px, py, pw, ph, skin);

    let title = "PAUSED";
    draw_tag_shadow(title, (w - tag_width(title, 18)) * 0.5, py + 48.0, 18, skin.accent);

    let mut action = PauseAction::None;
    let bx = px + 50.0;
    let bw = pw - 100.0;
    if draw_btn(tex, bx, py + 76.0, bw, 46.0, "RESUME GAME [Esc]", skin, true) {
        action = PauseAction::Resume;
    }
    let m_lbl = if muted { "AUDIO: MUTED [M]" } else { "AUDIO: ON [M]" };
    if draw_btn(tex, bx, py + 134.0, bw, 46.0, m_lbl, skin, false) {
        action = PauseAction::ToggleMute;
    }
    if draw_btn(tex, bx, py + 192.0, 78.0, 42.0, "VOL -", skin, false) {
        action = PauseAction::VolDown;
    }
    let v_lbl = format!("Music {}%", (vol * 100.0).round() as i32);
    let vw = dlg_width(&v_lbl, 24);
    draw_dlg_shadow(&v_lbl, px + (pw - vw) * 0.5, py + 220.0, 24, skin.ink);
    if draw_btn(tex, bx + bw - 78.0, py + 192.0, 78.0, 42.0, "VOL +", skin, false) {
        action = PauseAction::VolUp;
    }
    if draw_btn(tex, bx, py + 248.0, bw, 46.0, "RESTART BIOME", skin, false) {
        action = PauseAction::RestartBiome;
    }
    if draw_btn(tex, bx, py + 306.0, bw, 46.0, "RETURN TO TITLE", skin, false) {
        action = PauseAction::QuitTitle;
    }
    action
}
