//! Explorable overworld: player movement, camera, NPCs, props, world-space decor.

use macroquad::prelude::*;
use crate::art::Textures;
use crate::game::script::Biome;

pub const WALK: f32 = 240.0;
pub const RIDE: f32 = 400.0;
#[allow(dead_code)]
pub const SWIM: f32 = 170.0;
pub const REACH: f32 = 90.0;

pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub tex: Texture2D,
    pub label: &'static str,
    pub action: &'static str,
}

pub struct Prop {
    pub id: &'static str,
    pub x: f32,
    #[allow(dead_code)]
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: &'static str,
    pub action: &'static str,
}

pub struct World {
    /// World length per biome. Path runs left to right.
    pub biome: Biome,
    pub length: f32,
    pub player: (f32, f32),
    pub facing: f32,
    pub bob_t: f32,
    pub moving: bool,
    pub actors: Vec<Actor>,
    pub props: Vec<Prop>,
    pub hint_t: f32,
    pub hint: String,
    /// How many Cosma torches are lit (0..3).
    pub torches_lit: u32,
}

impl World {
    pub fn new(biome: Biome) -> Self {
        let mut w = World {
            biome,
            length: match biome {
                Biome::Candy => 3600.0,
                Biome::Aquarius => 3200.0,
                _ => 3000.0,
            },
            player: (140.0, 470.0),
            facing: 1.0,
            bob_t: 0.0,
            moving: false,
            actors: Vec::new(),
            props: Vec::new(),
            hint_t: 0.0,
            hint: String::new(),
            torches_lit: 0,
        };
        w.populate();
        w
    }

    fn populate(&mut self) {
        let t = |tex: &Texture2D| tex.clone();
        match self.biome {
            Biome::Candy => {
                self.actors.push(Actor { x: 700.0, y: 470.0, tex: t(&self_jin()), label: "Jin", action: "jin" });
                self.actors.push(Actor { x: 2450.0, y: 470.0, tex: t(&self_guard(true)), label: "Guard 1", action: "guards" });
                self.actors.push(Actor { x: 2530.0, y: 470.0, tex: t(&self_guard(false)), label: "Guard 2", action: "guards" });
                self.actors.push(Actor { x: 3250.0, y: 440.0, tex: t(&self_kandy()), label: "Kandy Kween", action: "kandy" });
                // riddle signposts and gate as props
                self.props.push(Prop { id: "riddle1", x: 1250.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "riddle2", x: 1700.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "gate", x: 2050.0, y: 380.0, w: 160.0, h: 180.0, label: "Throne Gate", action: "gate" });
            }
            Biome::Aquarius => {
                self.actors.push(Actor { x: 600.0, y: 480.0, tex: t(&self_gill()), label: "Gillbert", action: "clue" });
                self.actors.push(Actor { x: 1050.0, y: 480.0, tex: t(&self_scal()), label: "Scalleon", action: "clue" });
                self.actors.push(Actor { x: 1550.0, y: 480.0, tex: t(&self_lady()), label: "Market Lady", action: "clue" });
                self.props.push(Prop { id: "riddle1", x: 1850.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "riddle2", x: 2250.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "cave", x: 2700.0, y: 400.0, w: 170.0, h: 160.0, label: "Cave Entrance", action: "maze" });
            }
            Biome::Cosma => {
                self.props.push(Prop { id: "riddle1", x: 700.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "riddle2", x: 1100.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "torch1", x: 1600.0, y: 430.0, w: 50.0, h: 120.0, label: "Torch Podium", action: "stars" });
                self.props.push(Prop { id: "torch2", x: 2000.0, y: 430.0, w: 50.0, h: 120.0, label: "Torch Podium", action: "stars" });
                self.props.push(Prop { id: "torch3", x: 2400.0, y: 430.0, w: 50.0, h: 120.0, label: "Torch Podium", action: "stars" });
                self.props.push(Prop { id: "portal", x: 2900.0, y: 380.0, w: 180.0, h: 180.0, label: "Stone Circle", action: "portal" });
            }
        }
    }

    pub fn update(&mut self, dt: f32, locked: bool) {
        self.hint_t = (self.hint_t - dt).max(0.0);
        if locked {
            self.moving = false;
            return;
        }
        let speed = match self.biome {
            Biome::Candy => if self.has_ride() { RIDE } else { WALK },
            _ => WALK,
        };
        let mut dx = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dx -= speed * dt; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dx += speed * dt; }
        // W/S small vertical lane movement
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { self.player.1 -= 60.0 * dt; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { self.player.1 += 60.0 * dt; }
        self.player.1 = self.player.1.clamp(400.0, 560.0);
        if dx != 0.0 {
            self.facing = dx.signum();
            self.player.0 = (self.player.0 + dx).clamp(60.0, self.length - 60.0);
            self.bob_t += dt * if speed > 300.0 { 14.0 } else { 9.0 };
            self.moving = true;
        } else {
            self.moving = false;
        }
    }

    fn has_ride(&self) -> bool {
        crate::game::ride_unlocked()
    }

    pub fn nearest(&self) -> Option<(f32, &'static str)> {
        let px = self.player.0;
        let mut best: Option<(f32, &'static str)> = None;
        for a in &self.actors {
            let d = (a.x - px).abs();
            if d < REACH && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, a.action));
            }
        }
        for p in &self.props {
            let d = (p.x - px).abs();
            if d < REACH && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, p.action));
            }
        }
        best
    }

    pub fn nearest_label(&self) -> Option<&'static str> {
        let px = self.player.0;
        let mut best: Option<(f32, &'static str)> = None;
        for a in &self.actors {
            let d = (a.x - px).abs();
            if d < REACH && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, a.label));
            }
        }
        for p in &self.props {
            let d = (p.x - px).abs();
            if d < REACH && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, p.label));
            }
        }
        best.map(|(_, l)| l)
    }

    pub fn set_hint(&mut self, msg: &str) {
        self.hint = msg.to_string();
        self.hint_t = 2.5;
    }

    /// Which action target lies ahead of the player? For the guide arrow.
    pub fn next_target_x(&self) -> f32 {
        let px = self.player.0;
        let mut next = self.length - 120.0;
        for a in &self.actors {
            if a.x > px + 20.0 { next = next.min(a.x); }
        }
        for p in &self.props {
            if p.x > px + 20.0 { next = next.min(p.x); }
        }
        next
    }

    pub fn draw(&self, tex: &Textures, t: f32) {
        let cam_x = self.camera_x();
        let (sw, sh) = (screen_width(), screen_height());
        let ground_y = 560.0;

        // world-space decor unique per biome
        match self.biome {
            Biome::Candy => {
                // distant castle at the far end
                let cx = self.length - 250.0 - cam_x;
                if cx > -300.0 && cx < sw + 300.0 {
                    draw_rectangle(cx - 70.0, ground_y - 300.0, 140.0, 300.0, Color::from_rgba(255, 170, 200, 255));
                    draw_triangle(Vec2::new(cx - 90.0, ground_y - 300.0), Vec2::new(cx + 90.0, ground_y - 300.0), Vec2::new(cx, ground_y - 400.0), Color::from_rgba(230, 120, 160, 255));
                    draw_rectangle(cx - 20.0, ground_y - 90.0, 40.0, 90.0, Color::from_rgba(120, 60, 90, 255));
                }
                // lollipop bushes with parallax factor 0.9
                for i in 0..((self.length / 150.0) as i32) {
                    let x = i as f32 * 150.0 + 60.0;
                    let sx = x * 0.9 - cam_x * 0.9;
                    if sx < -60.0 || sx > sw + 60.0 { continue; }
                    draw_circle(sx, ground_y - 10.0, 24.0, Color::from_rgba(255, 105, 180, 255));
                    draw_rectangle(sx - 2.0, ground_y - 10.0, 4.0, 24.0, Color::from_rgba(120, 200, 90, 255));
                }
            }
            Biome::Aquarius => {
                // shipwreck ribs
                for i in 0..6 {
                    let x = 400.0 + i as f32 * 500.0;
                    let sx = x - cam_x;
                    if sx < -120.0 || sx > sw + 120.0 { continue; }
                    draw_rectangle(sx, ground_y - 120.0, 8.0, 120.0, Color::from_rgba(90, 60, 40, 255));
                    draw_rectangle(sx + 30.0, ground_y - 90.0, 8.0, 90.0, Color::from_rgba(90, 60, 40, 255));
                }
                // fish: 8, world anchored, sine bob
                for i in 0..8 {
                    let x = ((t * 40.0 + i as f32 * 400.0) % self.length) - cam_x;
                    let y = 200.0 + (i as f32 * 53.0) % 200.0 + (t * 2.0 + i as f32).sin() * 12.0;
                    if x < -30.0 || x > sw + 30.0 { continue; }
                    let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                    draw_triangle(Vec2::new(x, y), Vec2::new(x + 26.0 * dir, y + 8.0), Vec2::new(x, y + 16.0), Color::from_rgba(255, 170, 80, 230));
                    draw_triangle(Vec2::new(x, y + 8.0), Vec2::new(x - 12.0 * dir, y - 2.0), Vec2::new(x - 12.0 * dir, y + 18.0), Color::from_rgba(255, 140, 60, 230));
                }
            }
            Biome::Cosma => {
                // floating stones with slow bob
                for i in 0..12 {
                    let x = i as f32 * 260.0 + 80.0;
                    let sx = x - cam_x;
                    if sx < -40.0 || sx > sw + 40.0 { continue; }
                    let y = 320.0 + ((i * 97) % 160) as f32 + (t * 1.2 + i as f32).sin() * 8.0;
                    draw_poly(sx, y, 6, 14.0, t * 0.3 + i as f32, Color::from_rgba(110, 100, 140, 255));
                }
                // rainbow road arc near the end
                let rx = self.length - 320.0 - cam_x;
                if rx > -400.0 && rx < sw + 400.0 {
                    for band in 0..6 {
                        let c = Color::from_rgba(
                            255, (120 + band * 24) as u8, (60 + band * 30) as u8, 90);
                        draw_rectangle(rx - 10.0, ground_y - 360.0 + band as f32 * 12.0, 20.0, 12.0, c);
                    }
                }
            }
        }

        // ground strip
        let ground = match self.biome {
            Biome::Candy => Color::from_rgba(250, 230, 120, 255),
            Biome::Aquarius => Color::from_rgba(194, 178, 128, 255),
            Biome::Cosma => Color::from_rgba(70, 66, 90, 255),
        };
        draw_rectangle(0.0, ground_y, sw, sh - ground_y, ground);
        // path center line
        draw_rectangle(0.0, 556.0, sw, 4.0, Color::from_rgba(0, 0, 0, 40));

        // props (world space)
        for p in &self.props {
            let sx = p.x - cam_x;
            if sx < -200.0 || sx > sw + 200.0 { continue; }
            match p.id {
                "gate" | "cave" | "portal" => {
                    let c = match p.id {
                        "gate" => Color::from_rgba(120, 110, 130, 255),
                        "cave" => Color::from_rgba(50, 45, 70, 255),
                        _ => Color::from_rgba(90, 80, 120, 255),
                    };
                    draw_rectangle(sx - p.w / 2.0, ground_y - p.h, p.w, p.h, c);
                    draw_rectangle_lines(sx - p.w / 2.0, ground_y - p.h, p.w, p.h, 4.0, Color::from_rgba(30, 26, 40, 255));
                    // arch opening
                    draw_circle(sx, ground_y - p.h + p.h / 2.0, p.w / 3.0, Color::from_rgba(15, 12, 22, 255));
                }
                "torch1" | "torch2" | "torch3" => {
                    draw_rectangle(sx - 8.0, ground_y - p.h, 16.0, p.h, Color::from_rgba(120, 110, 130, 255));
                    let idx: u32 = p.id.get(5..6).and_then(|c| c.to_string().parse().ok()).unwrap_or(1);
                    let lit = idx <= self.torches_lit;
                    let flick = (t * 9.0 + idx as f32 * 2.0).sin() * 3.0;
                    if lit {
                        draw_circle(sx, ground_y - p.h - 12.0 + flick * 0.4, 13.0 + flick * 0.3, Color::from_rgba(255, 160, 60, 240));
                        draw_circle(sx, ground_y - p.h - 10.0, 7.0, Color::from_rgba(255, 230, 120, 255));
                    } else {
                        draw_circle(sx, ground_y - p.h - 10.0, 11.0, Color::from_rgba(70, 60, 80, 255));
                    }
                }
                _ => {
                    // riddle stone
                    draw_rectangle(sx - 20.0, ground_y - p.h, 40.0, p.h, Color::from_rgba(140, 130, 150, 255));
                    draw_text("?", sx - 8.0, ground_y - p.h + 40.0, 30.0, WHITE);
                }
            }
        }

        // actors (world space, gentle idle bob)
        for a in &self.actors {
            let sx = a.x - cam_x;
            if sx < -80.0 || sx > sw + 80.0 { continue; }
            let idle = (t * 3.0 + a.x * 0.01).sin() * 2.0;
            draw_texture_ex(&a.tex, sx - 24.0, a.y + idle, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(48.0, 72.0)), ..Default::default() });
            // nameplate
            let tw = measure_text(a.label, None, 14, 1.0).width;
            draw_rectangle(sx - tw / 2.0 - 6.0, a.y - 14.0, tw + 12.0, 18.0, Color::from_rgba(0, 0, 0, 120));
            draw_text(a.label, sx - tw / 2.0, a.y, 14.0, WHITE);
        }

        // player (facing flip + run bob + squash)
        let px = self.player.0 - cam_x;
        let py = self.player.1 + if self.moving { (self.bob_t * 6.0).sin() * 3.0 } else { (t * 2.0).sin() * 1.5 };
        let squash = if self.moving { 1.0 + (self.bob_t * 6.0).sin() * 0.05 } else { 1.0 };
        let pw = 48.0 * self.facing * squash;
        draw_texture_ex(&tex.river, px - pw / 2.0, py, WHITE,
            DrawTextureParams { dest_size: Some(Vec2::new(pw, 72.0 / squash)), ..Default::default() });

        // guide arrow to next target
        let nx = self.next_target_x() - cam_x;
        let bounce = (t * 4.0).sin() * 6.0;
        if nx > 60.0 && nx < sw + 60.0 {
            draw_triangle(
                Vec2::new(nx, 150.0 + bounce),
                Vec2::new(nx - 14.0, 130.0 + bounce),
                Vec2::new(nx + 14.0, 130.0 + bounce),
                Color::from_rgba(255, 105, 180, 220));
        } else if nx >= sw + 60.0 {
            draw_triangle(
                Vec2::new(sw - 30.0, 150.0 + bounce),
                Vec2::new(sw - 50.0, 130.0 + bounce),
                Vec2::new(sw - 14.0, 140.0 + bounce),
                Color::from_rgba(255, 105, 180, 220));
        }

        // proximity prompt
        if let Some(label) = self.nearest_label() {
            let msg = format!("z: talk to {}", label);
            let tw = measure_text(&msg, None, 18, 1.0).width;
            let bx = px - tw / 2.0 - 12.0;
            draw_rectangle(bx, 380.0, tw + 24.0, 26.0, Color::from_rgba(0, 0, 0, 170));
            draw_text(&msg, px - tw / 2.0, 398.0, 18.0, Color::from_rgba(255, 230, 120, 255));
        }

        // hint toast
        if self.hint_t > 0.0 && !self.hint.is_empty() {
            let tw = measure_text(&self.hint, None, 20, 1.0).width;
            draw_rectangle(sw * 0.5 - tw / 2.0 - 16.0, 90.0, tw + 32.0, 34.0, Color::from_rgba(0, 0, 0, 190));
            draw_text(&self.hint, sw * 0.5 - tw / 2.0, 113.0, 20.0, WHITE);
        }
    }

    pub fn camera_x(&self) -> f32 {
        let sw = screen_width();
        let target = self.player.0 - sw / 2.0;
        target.clamp(0.0, (self.length - sw).max(0.0))
    }
}

// --- standalone NPC sprites (16x24 pixel rigs, distinct silhouettes) ---

fn base_img() -> Image {
    Image::gen_image_color(16, 24, Color::new(0.0, 0.0, 0.0, 0.0))
}
fn px(img: &mut Image, x: i32, y: i32, w: i32, h: i32, c: Color) {
    for yy in y..y + h {
        for xx in x..x + w {
            if xx >= 0 && yy >= 0 && xx < 16 && yy < 24 {
                img.set_pixel(xx as u32, yy as u32, c);
            }
        }
    }
}

use std::sync::OnceLock;
static JIN: OnceLock<Texture2D> = OnceLock::new();
static GUARD_A: OnceLock<Texture2D> = OnceLock::new();
static GUARD_B: OnceLock<Texture2D> = OnceLock::new();
static KANDY: OnceLock<Texture2D> = OnceLock::new();
static GILL: OnceLock<Texture2D> = OnceLock::new();
static SCAL: OnceLock<Texture2D> = OnceLock::new();
static LADY: OnceLock<Texture2D> = OnceLock::new();

fn self_jin() -> Texture2D {
    JIN.get_or_init(|| {
        let mut img = base_img();
        let dough = Color::from_rgba(198, 124, 78, 255);
        let icing = Color::from_rgba(245, 245, 250, 255);
        px(&mut img, 5, 1, 6, 2, dough);   // head
        px(&mut img, 6, 3, 4, 3, dough);
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(40, 20, 10, 255)); // eyes
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(40, 20, 10, 255));
        px(&mut img, 5, 2, 6, 1, icing);   // icing fringe
        px(&mut img, 5, 7, 6, 8, dough);   // body
        px(&mut img, 7, 8, 1, 1, icing);   // gumdrop buttons
        px(&mut img, 7, 10, 1, 1, icing);
        px(&mut img, 3, 7, 2, 6, dough);   // arms
        px(&mut img, 12, 7, 2, 6, dough);
        px(&mut img, 6, 15, 2, 6, dough);  // legs
        px(&mut img, 9, 15, 2, 6, dough);
        px(&mut img, 6, 20, 1, 1, icing);
        px(&mut img, 9, 20, 1, 1, icing);
        Texture2D::from_image(&img)
    }).clone()
}

fn self_guard(a: bool) -> Texture2D {
    let cell = if a { &GUARD_A } else { &GUARD_B };
    cell.get_or_init(|| {
        let mut img = base_img();
        let uni = if a { Color::from_rgba(230, 60, 60, 255) } else { Color::from_rgba(60, 110, 230, 255) };
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(250, 250, 250, 255)); // helmet plume
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(250, 220, 190, 255)); // head
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(30, 30, 30, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(30, 30, 30, 255));
        px(&mut img, 5, 7, 6, 8, uni);
        px(&mut img, 3, 7, 2, 6, uni);
        px(&mut img, 12, 7, 2, 6, uni);
        px(&mut img, 6, 15, 2, 6, Color::from_rgba(150, 30, 30, 255));
        px(&mut img, 9, 15, 2, 6, Color::from_rgba(150, 30, 30, 255));
        px(&mut img, 14, 3, 1, 10, Color::from_rgba(240, 240, 240, 255)); // lollipop stick
        px(&mut img, 13, 1, 3, 3, if a { Color::from_rgba(255, 100, 160, 255) } else { Color::from_rgba(100, 160, 255, 255) });
        Texture2D::from_image(&img)
    }).clone()
}

fn self_kandy() -> Texture2D {
    KANDY.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 0, 6, 1, Color::from_rgba(255, 215, 0, 255)); // crown
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(255, 105, 180, 255)); // hair
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(255, 220, 200, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(120, 20, 60, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(120, 20, 60, 255));
        px(&mut img, 5, 7, 6, 10, Color::from_rgba(255, 20, 147, 255)); // gown
        px(&mut img, 4, 12, 8, 6, Color::from_rgba(255, 20, 147, 255));
        px(&mut img, 3, 7, 2, 6, Color::from_rgba(255, 220, 200, 255));
        px(&mut img, 12, 7, 2, 6, Color::from_rgba(255, 220, 200, 255));
        px(&mut img, 7, 8, 2, 1, Color::from_rgba(0, 255, 200, 255)); // necklace
        Texture2D::from_image(&img)
    }).clone()
}

fn self_gill() -> Texture2D {
    GILL.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(80, 200, 160, 255)); // fin hair
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(120, 220, 180, 255)); // fish skin
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(10, 40, 30, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(10, 40, 30, 255));
        px(&mut img, 5, 7, 6, 8, Color::from_rgba(90, 210, 170, 255));
        px(&mut img, 2, 8, 3, 2, Color::from_rgba(70, 180, 140, 255)); // side fins
        px(&mut img, 12, 8, 3, 2, Color::from_rgba(70, 180, 140, 255));
        px(&mut img, 6, 15, 2, 6, Color::from_rgba(60, 160, 130, 255));
        px(&mut img, 9, 15, 2, 6, Color::from_rgba(60, 160, 130, 255));
        Texture2D::from_image(&img)
    }).clone()
}

fn self_scal() -> Texture2D {
    SCAL.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(60, 120, 220, 255));
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(100, 160, 240, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(10, 20, 60, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(10, 20, 60, 255));
        px(&mut img, 5, 7, 6, 8, Color::from_rgba(80, 140, 230, 255));
        // scales
        px(&mut img, 6, 9, 1, 1, Color::from_rgba(140, 190, 255, 255));
        px(&mut img, 8, 9, 1, 1, Color::from_rgba(140, 190, 255, 255));
        px(&mut img, 10, 9, 1, 1, Color::from_rgba(140, 190, 255, 255));
        px(&mut img, 7, 11, 1, 1, Color::from_rgba(140, 190, 255, 255));
        px(&mut img, 9, 11, 1, 1, Color::from_rgba(140, 190, 255, 255));
        px(&mut img, 2, 8, 3, 2, Color::from_rgba(60, 110, 200, 255));
        px(&mut img, 12, 8, 3, 2, Color::from_rgba(60, 110, 200, 255));
        px(&mut img, 6, 15, 2, 6, Color::from_rgba(50, 100, 180, 255));
        px(&mut img, 9, 15, 2, 6, Color::from_rgba(50, 100, 180, 255));
        Texture2D::from_image(&img)
    }).clone()
}

fn self_lady() -> Texture2D {
    LADY.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(120, 80, 50, 255));
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(30, 30, 30, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(30, 30, 30, 255));
        px(&mut img, 5, 7, 6, 10, Color::from_rgba(200, 60, 120, 255)); // apron dress
        px(&mut img, 6, 10, 4, 4, Color::from_rgba(240, 230, 210, 255)); // apron
        px(&mut img, 3, 7, 2, 6, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 12, 7, 2, 6, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 6, 17, 1, 4, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 9, 17, 1, 4, Color::from_rgba(240, 200, 170, 255));
        Texture2D::from_image(&img)
    }).clone()
}
