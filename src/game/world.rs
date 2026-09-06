//! Explorable overworld: player movement, camera lerp, wandering NPCs,
//! particles, world-space decor, interaction prompts.

use macroquad::prelude::*;
use crate::art::Textures;
use crate::game::script::Biome;

pub const WALK: f32 = 240.0;
pub const RIDE: f32 = 400.0;
pub const REACH: f32 = 90.0;

pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub tex: Texture2D,
    pub label: &'static str,
    pub action: &'static str,
    /// Patrol range (min world x, max world x). None = stands still.
    pub patrol: Option<(f32, f32)>,
    pub dir: f32,
    pub speed: f32,
    pub face: f32,
}

impl Actor {
    pub fn new(x: f32, y: f32, tex: Texture2D, label: &'static str, action: &'static str) -> Self {
        Self { x, y, tex, label, action, patrol: None, dir: 1.0, speed: 0.0, face: -1.0 }
    }
    pub fn patrolling(mut self, min: f32, max: f32, speed: f32) -> Self {
        self.patrol = Some((min, max));
        self.speed = speed;
        self
    }
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

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub kind: u8, // 0 dust, 1 bubble, 2 sparkle, 3 ember
}

pub struct World {
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
    pub torches_lit: u32,
    pub cam: f32,
    pub particles: Vec<Particle>,
    pub step_t: f32,
    pub step_hit: bool,
    pub interact_cd: f32,
    amb_acc: f32,
}

impl World {
    pub fn new(biome: Biome) -> Self {
        let mut w = World {
            biome,
            length: 3000.0,
            player: (140.0, 470.0),
            facing: 1.0,
            bob_t: 0.0,
            moving: false,
            actors: Vec::new(),
            props: Vec::new(),
            hint_t: 0.0,
            hint: String::new(),
            torches_lit: 0,
            cam: 0.0,
            particles: Vec::new(),
            step_t: 0.0,
            step_hit: false,
            interact_cd: 0.0,
            amb_acc: 0.0,
        };
        match biome {
            Biome::Candy => { w.length = 3600.0; }
            Biome::Aquarius => { w.length = 3200.0; }
            Biome::Cosma => { w.length = 3000.0; }
            Biome::Home => { w.length = 1400.0; }
        }
        w.populate();
        w.cam = w.target_cam();
        w
    }

    fn populate(&mut self) {
        match self.biome {
            Biome::Candy => {
                self.actors.push(Actor::new(700.0, 470.0, self_jin(), "Jin", "jin"));
                self.actors.push(Actor::new(2450.0, 470.0, self_guard(true), "Guard 1", "guards")
                    .patrolling(2380.0, 2520.0, 30.0));
                self.actors.push(Actor::new(2530.0, 470.0, self_guard(false), "Guard 2", "guards")
                    .patrolling(2440.0, 2600.0, 24.0));
                self.actors.push(Actor::new(3250.0, 440.0, self_kandy(), "Kandy Kween", "kandy"));
                self.props.push(Prop { id: "riddle1", x: 1250.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "riddle2", x: 1700.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "gate", x: 2050.0, y: 380.0, w: 160.0, h: 180.0, label: "Throne Gate", action: "gate" });
            }
            Biome::Aquarius => {
                self.actors.push(Actor::new(600.0, 480.0, self_gill(), "Gillbert", "clue")
                    .patrolling(520.0, 760.0, 26.0));
                self.actors.push(Actor::new(1050.0, 480.0, self_scal(), "Scalleon", "clue")
                    .patrolling(980.0, 1220.0, 32.0));
                self.actors.push(Actor::new(1550.0, 480.0, self_lady(), "Market Lady", "clue"));
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
            Biome::Home => {
                self.actors.push(Actor::new(1020.0, 470.0, self_nadia(), "Nadia", "nadia"));
                self.props.push(Prop { id: "table", x: 600.0, y: 460.0, w: 120.0, h: 60.0, label: "Kitchen Table", action: "none" });
                self.props.push(Prop { id: "bed", x: 1150.0, y: 460.0, w: 150.0, h: 70.0, label: "Nadia's Bed", action: "none" });
            }
        }
    }

    pub fn update(&mut self, dt: f32, locked: bool) {
        self.hint_t = (self.hint_t - dt).max(0.0);
        self.interact_cd = (self.interact_cd - dt).max(0.0);
        self.update_particles(dt);
        self.spawn_ambient(dt);

        if locked {
            self.moving = false;
            return;
        }
        let speed = match self.biome {
            Biome::Candy => if crate::game::ride_unlocked() { RIDE } else { WALK },
            _ => WALK,
        };
        let mut dx = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dx -= speed * dt; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dx += speed * dt; }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { self.player.1 -= 60.0 * dt; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { self.player.1 += 60.0 * dt; }
        self.player.1 = self.player.1.clamp(400.0, 560.0);
        if dx != 0.0 {
            self.facing = dx.signum();
            self.player.0 = (self.player.0 + dx).clamp(60.0, self.length - 60.0);
            self.bob_t += dt * if speed > 300.0 { 14.0 } else { 9.0 };
            self.moving = true;
            // footsteps
            self.step_t += dt;
            let interval = if speed > 300.0 { 0.16 } else { 0.26 };
            if self.step_t >= interval {
                self.step_t = 0.0;
                self.step_hit = true;
                for _ in 0..3 {
                    self.particles.push(Particle {
                        x: self.player.0 + rand::gen_range(-8.0, 8.0),
                        y: self.player.1 + 66.0,
                        vx: rand::gen_range(-30.0, 30.0),
                        vy: rand::gen_range(-40.0, -10.0),
                        life: 0.5, max_life: 0.5, kind: 0,
                    });
                }
            }
        } else {
            self.moving = false;
        }

        // NPC wander + face player
        let px = self.player.0;
        for a in self.actors.iter_mut() {
            if let Some((mn, mx)) = a.patrol {
                let near = (a.x - px).abs() < REACH * 1.4;
                if near {
                    a.face = (px - a.x).signum();
                } else {
                    a.x += a.dir * a.speed * dt;
                    if a.x < mn { a.x = mn; a.dir = 1.0; }
                    if a.x > mx { a.x = mx; a.dir = -1.0; }
                    a.face = a.dir;
                }
            } else {
                if (a.x - px).abs() < REACH * 1.6 {
                    a.face = (px - a.x).signum();
                }
            }
        }

        // camera lerp
        let target = self.target_cam();
        self.cam += (target - self.cam) * (dt * 5.0).min(1.0);
    }

    pub fn target_cam(&self) -> f32 {
        let sw = screen_width();
        (self.player.0 - sw / 2.0).clamp(0.0, (self.length - sw).max(0.0))
    }

    fn update_particles(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0 && p.y > -40.0 && p.y < 780.0);
        if self.particles.len() > 220 {
            self.particles.drain(0..self.particles.len() - 220);
        }
    }

    fn spawn_ambient(&mut self, dt: f32) {
        let (rate, kind) = match self.biome {
            Biome::Candy => (3.0, 2u8),
            Biome::Aquarius => (4.0, 1u8),
            Biome::Cosma => (3.0, 3u8),
            Biome::Home => (2.0, 0u8),
        };
        self.amb_acc += dt * rate;
        while self.amb_acc >= 1.0 {
            self.amb_acc -= 1.0;
            if self.particles.len() >= 200 { break; }
            let x = self.cam + rand::gen_range(0.0, screen_width());
            let p = match kind {
                2 => Particle { x, y: -10.0, vx: rand::gen_range(-15.0, 15.0), vy: rand::gen_range(25.0, 45.0), life: rand::gen_range(3.0, 6.0), max_life: 5.0, kind: 2 },
                1 => Particle { x, y: 740.0, vx: rand::gen_range(-8.0, 8.0), vy: rand::gen_range(-70.0, -35.0), life: rand::gen_range(4.0, 8.0), max_life: 8.0, kind: 1 },
                3 => Particle { x, y: 740.0, vx: rand::gen_range(-20.0, 20.0), vy: rand::gen_range(-30.0, -12.0), life: rand::gen_range(4.0, 9.0), max_life: 9.0, kind: 3 },
                _ => Particle { x, y: rand::gen_range(200.0, 600.0), vx: rand::gen_range(-6.0, 6.0), vy: rand::gen_range(8.0, 20.0), life: rand::gen_range(4.0, 8.0), max_life: 8.0, kind: 0 },
            };
            self.particles.push(p);
        }
    }

    pub fn burst(&mut self, x: f32, y: f32, kind: u8, n: usize) {
        for _ in 0..n {
            let ang = rand::gen_range(0.0, std::f32::consts::TAU);
            let sp = rand::gen_range(50.0, 170.0);
            self.particles.push(Particle {
                x, y,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp - 40.0,
                life: rand::gen_range(0.4, 0.9),
                max_life: 0.9,
                kind,
            });
        }
    }

    pub fn set_hint(&mut self, msg: &str) {
        self.hint = msg.to_string();
        self.hint_t = 2.5;
    }

    /// Action id of the nearest interactable within REACH.
    pub fn nearest_action(&self) -> Option<&'static str> {
        self.nearest_info().map(|(_, _, a)| a)
    }

    /// Nearest interactable within REACH: (label, is_actor, action).
    pub fn nearest_info(&self) -> Option<(&'static str, bool, &'static str)> {
        let px = self.player.0;
        let mut best: Option<(f32, &'static str, bool, &'static str)> = None;
        for a in &self.actors {
            let d = (a.x - px).abs();
            if d < REACH && best.as_ref().map(|(bd, ..)| d < *bd).unwrap_or(true) {
                best = Some((d, a.label, true, a.action));
            }
        }
        for p in &self.props {
            let d = (p.x - px).abs();
            if d < REACH && best.as_ref().map(|(bd, ..)| d < *bd).unwrap_or(true) {
                best = Some((d, p.label, false, p.action));
            }
        }
        best.map(|(_, l, ia, act)| (l, ia, act))
    }

    /// World x of the next thing worth walking to (for the guide arrow).
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

    pub fn draw(&self, tex: &Textures, t: f32, shake: f32) {
        let cam_x = self.cam + shake;
        let (sw, sh) = (screen_width(), screen_height());
        let ground_y = 560.0;

        // biome decor
        match self.biome {
            Biome::Candy => {
                let cx = self.length - 250.0 - cam_x;
                if cx > -300.0 && cx < sw + 300.0 {
                    draw_rectangle(cx - 70.0, ground_y - 300.0, 140.0, 300.0, Color::from_rgba(255, 170, 200, 255));
                    draw_triangle(Vec2::new(cx - 90.0, ground_y - 300.0), Vec2::new(cx + 90.0, ground_y - 300.0), Vec2::new(cx, ground_y - 400.0), Color::from_rgba(230, 120, 160, 255));
                    draw_rectangle(cx - 20.0, ground_y - 90.0, 40.0, 90.0, Color::from_rgba(120, 60, 90, 255));
                }
                for i in 0..((self.length / 150.0) as i32) {
                    let x = i as f32 * 150.0 + 60.0;
                    let sx = x - cam_x;
                    if sx < -60.0 || sx > sw + 60.0 { continue; }
                    draw_circle(sx, ground_y - 10.0, 24.0, Color::from_rgba(255, 105, 180, 255));
                    draw_rectangle(sx - 2.0, ground_y - 10.0, 4.0, 24.0, Color::from_rgba(120, 200, 90, 255));
                }
            }
            Biome::Aquarius => {
                for i in 0..6 {
                    let x = 400.0 + i as f32 * 500.0;
                    let sx = x - cam_x;
                    if sx < -120.0 || sx > sw + 120.0 { continue; }
                    draw_rectangle(sx, ground_y - 120.0, 8.0, 120.0, Color::from_rgba(90, 60, 40, 255));
                    draw_rectangle(sx + 30.0, ground_y - 90.0, 8.0, 90.0, Color::from_rgba(90, 60, 40, 255));
                }
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
                for i in 0..12 {
                    let x = i as f32 * 260.0 + 80.0;
                    let sx = x - cam_x;
                    if sx < -40.0 || sx > sw + 40.0 { continue; }
                    let y = 320.0 + ((i * 97) % 160) as f32 + (t * 1.2 + i as f32).sin() * 8.0;
                    draw_poly(sx, y, 6, 14.0, t * 0.3 + i as f32, Color::from_rgba(110, 100, 140, 255));
                }
                let rx = self.length - 320.0 - cam_x;
                if rx > -400.0 && rx < sw + 400.0 {
                    for band in 0..6 {
                        let c = Color::from_rgba(255, (120 + band * 24) as u8, (60 + band * 30) as u8, 90);
                        draw_rectangle(rx - 10.0, ground_y - 360.0 + band as f32 * 12.0, 20.0, 12.0, c);
                    }
                }
            }
            Biome::Home => {
                // wood plank floor lines
                let start = (cam_x / 90.0).floor() as i32;
                for i in start..start + ((sw / 90.0) as i32) + 2 {
                    let x = i as f32 * 90.0 - cam_x;
                    draw_rectangle(x, ground_y, 2.0, sh - ground_y, Color::from_rgba(110, 74, 48, 255));
                }
                // rug
                let rug_x = 900.0 - cam_x;
                if rug_x > -260.0 && rug_x < sw + 260.0 {
                    draw_rectangle(rug_x - 130.0, ground_y + 40.0, 260.0, 60.0, Color::from_rgba(160, 70, 70, 255));
                    draw_rectangle_lines(rug_x - 130.0, ground_y + 40.0, 260.0, 60.0, 3.0, Color::from_rgba(120, 50, 50, 255));
                }
            }
        }

        // ground strip
        let ground = match self.biome {
            Biome::Candy => Color::from_rgba(250, 230, 120, 255),
            Biome::Aquarius => Color::from_rgba(194, 178, 128, 255),
            Biome::Cosma => Color::from_rgba(70, 66, 90, 255),
            Biome::Home => Color::from_rgba(154, 108, 74, 255),
        };
        draw_rectangle(0.0, ground_y, sw, sh - ground_y, ground);
        if self.biome != Biome::Home {
            draw_rectangle(0.0, 556.0, sw, 4.0, Color::from_rgba(0, 0, 0, 40));
        }

        // props
        for p in &self.props {
            let sx = p.x - cam_x;
            if sx < -220.0 || sx > sw + 220.0 { continue; }
            match p.id {
                "gate" | "cave" | "portal" => {
                    let c = match p.id {
                        "gate" => Color::from_rgba(120, 110, 130, 255),
                        "cave" => Color::from_rgba(50, 45, 70, 255),
                        _ => Color::from_rgba(90, 80, 120, 255),
                    };
                    draw_rectangle(sx - p.w / 2.0, ground_y - p.h, p.w, p.h, c);
                    draw_rectangle_lines(sx - p.w / 2.0, ground_y - p.h, p.w, p.h, 4.0, Color::from_rgba(30, 26, 40, 255));
                    draw_circle(sx, ground_y - p.h + p.h / 2.0, p.w / 3.0, Color::from_rgba(15, 12, 22, 255));
                    if p.id == "portal" && self.torches_lit >= 3 {
                        let glow = 0.5 + 0.5 * (t * 3.0).sin();
                        draw_circle(sx, ground_y - p.h + p.h / 2.0, p.w / 3.0 + glow * 8.0, Color::from_rgba(180, 140, 255, 90));
                    }
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
                "bed" => {
                    draw_rectangle(sx - p.w / 2.0, ground_y - 30.0, p.w, 34.0, Color::from_rgba(110, 70, 50, 255));
                    draw_rectangle(sx - p.w / 2.0, ground_y - 66.0, 14.0, 70.0, Color::from_rgba(90, 55, 40, 255));
                    draw_rectangle(sx - p.w / 2.0 + 12.0, ground_y - 46.0, p.w - 16.0, 22.0, Color::from_rgba(235, 230, 220, 255));
                    draw_rectangle(sx - p.w / 2.0 + 40.0, ground_y - 46.0, p.w - 54.0, 22.0, Color::from_rgba(255, 140, 170, 255));
                    draw_rectangle(sx - p.w / 2.0 + 16.0, ground_y - 52.0, 26.0, 12.0, Color::from_rgba(255, 255, 255, 255));
                }
                "table" => {
                    draw_rectangle(sx - p.w / 2.0, ground_y - 46.0, p.w, 10.0, Color::from_rgba(120, 80, 55, 255));
                    draw_rectangle(sx - p.w / 2.0 + 8.0, ground_y - 36.0, 8.0, 36.0, Color::from_rgba(95, 62, 42, 255));
                    draw_rectangle(sx + p.w / 2.0 - 16.0, ground_y - 36.0, 8.0, 36.0, Color::from_rgba(95, 62, 42, 255));
                    draw_rectangle(sx - 14.0, ground_y - 54.0, 28.0, 8.0, Color::from_rgba(140, 40, 40, 255));
                    draw_rectangle(sx - 2.0, ground_y - 54.0, 4.0, 8.0, Color::from_rgba(90, 20, 20, 255));
                }
                _ => {
                    draw_rectangle(sx - 20.0, ground_y - p.h, 40.0, p.h, Color::from_rgba(140, 130, 150, 255));
                    draw_text("?", sx - 8.0, ground_y - p.h + 40.0, 30.0, WHITE);
                }
            }
        }

        // actors
        for a in &self.actors {
            let sx = a.x - cam_x;
            if sx < -80.0 || sx > sw + 80.0 { continue; }
            let walk = if a.speed > 0.0 && (a.x - self.player.0).abs() > REACH * 1.4 {
                (t * 8.0 + a.x * 0.05).sin() * 2.5
            } else {
                (t * 3.0 + a.x * 0.01).sin() * 2.0
            };
            let aw = 48.0 * a.face;
            draw_texture_ex(&a.tex, sx - 24.0, a.y + walk, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(aw, 72.0)), ..Default::default() });
            let tw = measure_text(a.label, None, 14, 1.0).width;
            draw_rectangle(sx - tw / 2.0 - 6.0, a.y - 14.0, tw + 12.0, 18.0, Color::from_rgba(0, 0, 0, 120));
            draw_text(a.label, sx - tw / 2.0, a.y, 14.0, WHITE);
            // "!" prompt bubble when in range
            if (a.x - self.player.0).abs() < REACH {
                let bounce = (t * 5.0).sin() * 3.0;
                draw_circle(sx, a.y - 34.0 + bounce, 11.0, Color::from_rgba(255, 230, 120, 240));
                draw_text("!", sx - 3.0, a.y - 28.0 + bounce, 18.0, Color::from_rgba(40, 20, 10, 255));
            }
        }

        // player
        let px = self.player.0 - cam_x;
        let py = self.player.1 + if self.moving { (self.bob_t * 6.0).sin() * 3.0 } else { (t * 2.0).sin() * 1.5 };
        let squash = if self.moving { 1.0 + (self.bob_t * 6.0).sin() * 0.05 } else { 1.0 };
        let pw = 48.0 * self.facing * squash;
        draw_texture_ex(&tex.river, px - pw / 2.0, py, WHITE,
            DrawTextureParams { dest_size: Some(Vec2::new(pw, 72.0 / squash)), ..Default::default() });

        // guide arrow
        let nx = self.next_target_x() - cam_x;
        let bounce = (t * 4.0).sin() * 6.0;
        if nx > 60.0 && nx < sw + 60.0 {
            draw_triangle(Vec2::new(nx, 150.0 + bounce), Vec2::new(nx - 14.0, 130.0 + bounce), Vec2::new(nx + 14.0, 130.0 + bounce), Color::from_rgba(255, 105, 180, 220));
        } else if nx >= sw + 60.0 {
            draw_triangle(Vec2::new(sw - 30.0, 150.0 + bounce), Vec2::new(sw - 50.0, 130.0 + bounce), Vec2::new(sw - 14.0, 140.0 + bounce), Color::from_rgba(255, 105, 180, 220));
        }

        // particles
        for p in &self.particles {
            let a = (p.life / p.max_life).clamp(0.0, 1.0);
            let sx = p.x - cam_x;
            if sx < -20.0 || sx > sw + 20.0 { continue; }
            match p.kind {
                0 => draw_circle(sx, p.y, 2.5, Color::from_rgba(220, 210, 200, (150.0 * a) as u8)),
                1 => draw_circle_lines(sx, p.y, 3.0 + (1.0 - a) * 4.0, 1.5, Color::from_rgba(180, 220, 255, (170.0 * a) as u8)),
                2 => {
                    let tw = (p.life * 10.0).sin().abs().max(0.2);
                    draw_rectangle(sx - 2.5, p.y - 1.0, 5.0, 2.0, Color::from_rgba(255, 240, 160, (220.0 * a * tw) as u8));
                    draw_rectangle(sx - 1.0, p.y - 2.5, 2.0, 5.0, Color::from_rgba(255, 240, 160, (220.0 * a * tw) as u8));
                }
                _ => draw_circle(sx, p.y, 2.2, Color::from_rgba(255, 150, 80, (190.0 * a) as u8)),
            }
        }

        // proximity prompt
        if let Some((label, is_actor, _)) = self.nearest_info() {
            let verb = if is_actor { "z: talk to" } else { "z: inspect" };
            let msg = format!("{} {}", verb, label);
            let tw = measure_text(&msg, None, 18, 1.0).width;
            draw_rectangle(px - tw / 2.0 - 12.0, 380.0, tw + 24.0, 26.0, Color::from_rgba(0, 0, 0, 170));
            draw_text(&msg, px - tw / 2.0, 398.0, 18.0, Color::from_rgba(255, 230, 120, 255));
        }

        // hint toast
        if self.hint_t > 0.0 && !self.hint.is_empty() {
            let tw = measure_text(&self.hint, None, 20, 1.0).width;
            draw_rectangle(sw * 0.5 - tw / 2.0 - 16.0, 90.0, tw + 32.0, 34.0, Color::from_rgba(0, 0, 0, 190));
            draw_text(&self.hint, sw * 0.5 - tw / 2.0, 113.0, 20.0, WHITE);
        }
    }
}

// --- standalone NPC sprites ---

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
static NADIA: OnceLock<Texture2D> = OnceLock::new();

fn self_jin() -> Texture2D {
    JIN.get_or_init(|| {
        let mut img = base_img();
        let dough = Color::from_rgba(198, 124, 78, 255);
        let icing = Color::from_rgba(245, 245, 250, 255);
        px(&mut img, 5, 1, 6, 2, dough);
        px(&mut img, 6, 3, 4, 3, dough);
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(40, 20, 10, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(40, 20, 10, 255));
        px(&mut img, 5, 2, 6, 1, icing);
        px(&mut img, 5, 7, 6, 8, dough);
        px(&mut img, 7, 8, 1, 1, icing);
        px(&mut img, 7, 10, 1, 1, icing);
        px(&mut img, 3, 7, 2, 6, dough);
        px(&mut img, 12, 7, 2, 6, dough);
        px(&mut img, 6, 15, 2, 6, dough);
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
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(250, 250, 250, 255));
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(250, 220, 190, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(30, 30, 30, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(30, 30, 30, 255));
        px(&mut img, 5, 7, 6, 8, uni);
        px(&mut img, 3, 7, 2, 6, uni);
        px(&mut img, 12, 7, 2, 6, uni);
        px(&mut img, 6, 15, 2, 6, Color::from_rgba(150, 30, 30, 255));
        px(&mut img, 9, 15, 2, 6, Color::from_rgba(150, 30, 30, 255));
        px(&mut img, 14, 3, 1, 10, Color::from_rgba(240, 240, 240, 255));
        px(&mut img, 13, 1, 3, 3, if a { Color::from_rgba(255, 100, 160, 255) } else { Color::from_rgba(100, 160, 255, 255) });
        Texture2D::from_image(&img)
    }).clone()
}

fn self_kandy() -> Texture2D {
    KANDY.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 0, 6, 1, Color::from_rgba(255, 215, 0, 255));
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(255, 105, 180, 255));
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(255, 220, 200, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(120, 20, 60, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(120, 20, 60, 255));
        px(&mut img, 5, 7, 6, 10, Color::from_rgba(255, 20, 147, 255));
        px(&mut img, 4, 12, 8, 6, Color::from_rgba(255, 20, 147, 255));
        px(&mut img, 3, 7, 2, 6, Color::from_rgba(255, 220, 200, 255));
        px(&mut img, 12, 7, 2, 6, Color::from_rgba(255, 220, 200, 255));
        px(&mut img, 7, 8, 2, 1, Color::from_rgba(0, 255, 200, 255));
        Texture2D::from_image(&img)
    }).clone()
}

fn self_gill() -> Texture2D {
    GILL.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(80, 200, 160, 255));
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(120, 220, 180, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(10, 40, 30, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(10, 40, 30, 255));
        px(&mut img, 5, 7, 6, 8, Color::from_rgba(90, 210, 170, 255));
        px(&mut img, 2, 8, 3, 2, Color::from_rgba(70, 180, 140, 255));
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
        px(&mut img, 5, 7, 6, 10, Color::from_rgba(200, 60, 120, 255));
        px(&mut img, 6, 10, 4, 4, Color::from_rgba(240, 230, 210, 255));
        px(&mut img, 3, 7, 2, 6, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 12, 7, 2, 6, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 6, 17, 1, 4, Color::from_rgba(240, 200, 170, 255));
        px(&mut img, 9, 17, 1, 4, Color::from_rgba(240, 200, 170, 255));
        Texture2D::from_image(&img)
    }).clone()
}

fn self_nadia() -> Texture2D {
    NADIA.get_or_init(|| {
        let mut img = base_img();
        px(&mut img, 5, 1, 6, 2, Color::from_rgba(60, 40, 35, 255));
        px(&mut img, 6, 3, 4, 3, Color::from_rgba(235, 195, 165, 255));
        px(&mut img, 7, 4, 1, 1, Color::from_rgba(40, 40, 60, 255));
        px(&mut img, 9, 4, 1, 1, Color::from_rgba(40, 40, 60, 255));
        px(&mut img, 5, 7, 6, 10, Color::from_rgba(255, 170, 190, 255));
        px(&mut img, 4, 12, 8, 5, Color::from_rgba(255, 170, 190, 255));
        px(&mut img, 3, 7, 2, 6, Color::from_rgba(235, 195, 165, 255));
        px(&mut img, 12, 7, 2, 6, Color::from_rgba(235, 195, 165, 255));
        px(&mut img, 6, 17, 1, 4, Color::from_rgba(235, 195, 165, 255));
        px(&mut img, 9, 17, 1, 4, Color::from_rgba(235, 195, 165, 255));
        Texture2D::from_image(&img)
    }).clone()
}
