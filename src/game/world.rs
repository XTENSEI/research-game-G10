//! Explorable overworld: player movement, camera lerp, wandering NPCs,
//! particles, world-space decor, interaction prompts.

use macroquad::prelude::*;
use crate::art::{Textures, draw_tile, draw_tile_tint, c};
use crate::art::fx::Fx;
use crate::game::script::Biome;
use crate::game::ui::{draw_tag, tag_width, draw_dlg_text, dlg_width};

pub const WALK: f32 = 240.0;
pub const RIDE: f32 = 400.0;
pub const REACH: f32 = 90.0;

/// Draw a particle sprite centred on (x, y), rotating about its own middle.
fn draw_spr(tex: &Texture2D, x: f32, y: f32, size: f32, rot: f32, col: Color) {
    draw_texture_ex(tex, x - size / 2.0, y - size / 2.0, col, DrawTextureParams {
        dest_size: Some(Vec2::new(size, size)),
        rotation: rot,
        pivot: Some(Vec2::new(size / 2.0, size / 2.0)),
        ..Default::default()
    });
}

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
    /// Cast id for the walk cycle in Textures::walks ("" = single pose).
    pub anim: &'static str,
    pub scale: f32,
}

impl Actor {
    pub fn new(x: f32, y: f32, tex: Texture2D, label: &'static str, action: &'static str) -> Self {
        Self { x, y, tex, label, action, patrol: None, dir: 1.0, speed: 0.0, face: -1.0, anim: "", scale: 1.0 }
    }
    pub fn patrolling(mut self, min: f32, max: f32, speed: f32) -> Self {
        self.patrol = Some((min, max));
        self.speed = speed;
        self
    }
    pub fn anim(mut self, id: &'static str) -> Self {
        self.anim = id;
        self
    }
    pub fn scal(mut self, s: f32) -> Self {
        self.scale = s;
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

/// One particle. Rendered from the Kenney particle sprite kit rather than from
/// primitives, and given per-particle rotation, size and physics so drifts of
/// the same emitter do not read as a repeated stamp.
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    /// 0 dust, 1 bubble, 2 sparkle, 3 ember, 4 sugar swirl, 5 void dust,
    /// 6 rune magic, 7 light fleck.
    pub kind: u8,
    pub size: f32,
    pub rot: f32,
    pub spin: f32,
    /// Downward acceleration in px/s^2 (negative floats).
    pub grav: f32,
    /// Velocity retention per second (1.0 = no drag).
    pub drag: f32,
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
    /// Where River is walking to after a click (world x). None = no click target.
    pub click: Option<f32>,
    /// Spawn accumulator per particle family (indexed by kind).
    amb_acc: [f32; 8],
}

impl World {
    pub fn new(biome: Biome, tex: &Textures) -> Self {
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
            click: None,
            amb_acc: [0.0; 8],
        };
        match biome {
            Biome::Candy => { w.length = 3600.0; }
            Biome::Aquarius => { w.length = 3200.0; }
            Biome::Cosma => { w.length = 3000.0; }
            Biome::Home => { w.length = 1400.0; }
        }
        w.populate(tex);
        w.cam = w.target_cam();
        w
    }

    fn populate(&mut self, tex: &Textures) {
        match self.biome {
            Biome::Candy => {
                self.actors.push(Actor::new(700.0, 470.0, tex.jin.clone(), "Jin", "jin").anim("jin"));
                self.actors.push(Actor::new(2450.0, 470.0, tex.guard_a.clone(), "Guard 1", "guards")
                    .anim("guard_a").patrolling(2380.0, 2520.0, 30.0));
                self.actors.push(Actor::new(2530.0, 470.0, tex.guard_b.clone(), "Guard 2", "guards")
                    .anim("guard_b").patrolling(2440.0, 2600.0, 24.0));
                self.actors.push(Actor::new(3250.0, 440.0, tex.kandy.clone(), "Kandy Kween", "kandy").anim("kandy"));
                self.props.push(Prop { id: "riddle1", x: 1250.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "riddle2", x: 1700.0, y: 460.0, w: 60.0, h: 90.0, label: "Riddle Stone", action: "riddle" });
                self.props.push(Prop { id: "gate", x: 2050.0, y: 380.0, w: 160.0, h: 180.0, label: "Throne Gate", action: "gate" });
            }
            Biome::Aquarius => {
                self.actors.push(Actor::new(600.0, 480.0, tex.gill.clone(), "Gillbert", "clue")
                    .anim("gill").patrolling(520.0, 760.0, 26.0));
                self.actors.push(Actor::new(1050.0, 480.0, tex.scal.clone(), "Scalleon", "clue")
                    .anim("scal").patrolling(980.0, 1220.0, 32.0));
                self.actors.push(Actor::new(1550.0, 480.0, tex.lady.clone(), "Market Lady", "clue")
                    .anim("lady"));
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
                self.actors.push(Actor::new(1020.0, 470.0, tex.nadia.clone(), "Nadia", "nadia").anim("nadia").scal(0.8));
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
        // click-to-walk target
        if dx == 0.0 {
            if let Some(tx) = self.click {
                let d = tx - self.player.0;
                if d.abs() > 8.0 {
                    dx = d.signum() * speed * dt;
                } else {
                    self.click = None;
                }
            }
        }
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
                self.step_puff(if speed > 300.0 { 1.7 } else { 1.0 });
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

    /// Kicked-up ground under the player's feet, scaled by speed so a gallop
    /// throws more than a walk.
    fn step_puff(&mut self, power: f32) {
        let n = (3.0 * power) as usize + 1;
        for _ in 0..n {
            self.particles.push(Particle {
                x: self.player.0 - self.facing * 12.0 + rand::gen_range(-10.0, 10.0),
                y: self.player.1 + 66.0 + rand::gen_range(-4.0, 4.0),
                vx: rand::gen_range(-34.0, 34.0) - self.facing * 24.0,
                vy: rand::gen_range(-46.0, -12.0) * power,
                life: rand::gen_range(0.32, 0.58),
                max_life: 0.58,
                kind: 0,
                size: rand::gen_range(16.0, 30.0) * power,
                rot: rand::gen_range(0.0, 6.28),
                spin: rand::gen_range(-2.4, 2.4),
                grav: 34.0,
                drag: 0.10,
            });
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += p.grav * dt;
            let d = (1.0 - p.drag * dt).clamp(0.0, 1.0);
            p.vx *= d;
            p.vy *= d;
            p.rot += p.spin * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0 && p.y > -80.0 && p.y < 900.0);
        if self.particles.len() > 320 {
            let cut = self.particles.len() - 320;
            self.particles.drain(0..cut);
        }
    }

    /// Ambient life, per biome. Each biome mixes several families at their own
    /// rates, which is what stops the atmosphere reading as one uniform drizzle.
    fn spawn_ambient(&mut self, dt: f32) {
        let families: &[(f32, u8)] = match self.biome {
            // sugar swirls drifting down + glinting sparkles
            Biome::Candy => &[(2.6, 4), (1.5, 2)],
            // bubbles, kicked silt, light flecks
            Biome::Aquarius => &[(4.4, 1), (2.0, 0), (1.6, 7)],
            // neon embers rising, void dust, rune magic
            Biome::Cosma => &[(3.2, 3), (2.0, 5), (0.9, 6)],
            // room motes and hearth ash
            Biome::Home => &[(1.4, 7), (0.7, 0)],
        };
        let sw = screen_width();
        for (rate, kind) in families {
            let k = *kind as usize;
            self.amb_acc[k] += dt * rate;
            while self.amb_acc[k] >= 1.0 {
                self.amb_acc[k] -= 1.0;
                if self.particles.len() >= 280 {
                    break;
                }
                let x = self.cam + rand::gen_range(-60.0, sw + 60.0);
                let p = match kind {
                    // falling sugar swirl
                    4 => Particle {
                        x, y: -16.0,
                        vx: rand::gen_range(-16.0, 16.0),
                        vy: rand::gen_range(22.0, 42.0),
                        life: rand::gen_range(5.0, 9.0), max_life: 9.0, kind: 4,
                        size: rand::gen_range(12.0, 26.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-1.6, 1.6),
                        grav: 0.0, drag: 0.0,
                    },
                    // star sparkle
                    2 => Particle {
                        x, y: rand::gen_range(-20.0, 240.0),
                        vx: rand::gen_range(-10.0, 10.0),
                        vy: rand::gen_range(8.0, 22.0),
                        life: rand::gen_range(3.5, 6.5), max_life: 6.5, kind: 2,
                        size: rand::gen_range(10.0, 22.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-0.8, 0.8),
                        grav: 0.0, drag: 0.05,
                    },
                    // bubble rising off the sand
                    1 => Particle {
                        x: self.cam + rand::gen_range(0.0, sw), y: rand::gen_range(560.0, 720.0),
                        vx: rand::gen_range(-10.0, 10.0),
                        vy: rand::gen_range(-64.0, -30.0),
                        life: rand::gen_range(5.0, 10.0), max_life: 10.0, kind: 1,
                        size: rand::gen_range(7.0, 22.0),
                        rot: 0.0, spin: 0.0,
                        grav: 0.0, drag: 0.02,
                    },
                    // silt drifting through the current
                    7 => Particle {
                        x, y: rand::gen_range(60.0, 640.0),
                        vx: rand::gen_range(-14.0, 14.0),
                        vy: rand::gen_range(-6.0, 6.0),
                        life: rand::gen_range(5.0, 10.0), max_life: 10.0, kind: 7,
                        size: rand::gen_range(10.0, 26.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-0.5, 0.5),
                        grav: 0.0, drag: 0.0,
                    },
                    // neon ember lifting off the walkway
                    3 => Particle {
                        x, y: rand::gen_range(540.0, 720.0),
                        vx: rand::gen_range(-22.0, 22.0),
                        vy: rand::gen_range(-42.0, -16.0),
                        life: rand::gen_range(4.0, 9.0), max_life: 9.0, kind: 3,
                        size: rand::gen_range(8.0, 20.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-2.0, 2.0),
                        grav: -6.0, drag: 0.25,
                    },
                    // huge slow void dust
                    5 => Particle {
                        x, y: rand::gen_range(120.0, 700.0),
                        vx: rand::gen_range(-8.0, 8.0),
                        vy: rand::gen_range(-10.0, -2.0),
                        life: rand::gen_range(8.0, 15.0), max_life: 15.0, kind: 5,
                        size: rand::gen_range(60.0, 150.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-0.25, 0.25),
                        grav: 0.0, drag: 0.0,
                    },
                    // rune magic
                    6 => Particle {
                        x, y: rand::gen_range(300.0, 620.0),
                        vx: rand::gen_range(-10.0, 10.0),
                        vy: rand::gen_range(-24.0, -8.0),
                        life: rand::gen_range(2.5, 5.0), max_life: 5.0, kind: 6,
                        size: rand::gen_range(18.0, 40.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-1.2, 1.2),
                        grav: 0.0, drag: 0.0,
                    },
                    // plain dust mote
                    _ => Particle {
                        x, y: rand::gen_range(200.0, 620.0),
                        vx: rand::gen_range(-7.0, 7.0),
                        vy: rand::gen_range(-4.0, 10.0),
                        life: rand::gen_range(5.0, 10.0), max_life: 10.0, kind: 0,
                        size: rand::gen_range(14.0, 34.0),
                        rot: rand::gen_range(0.0, 6.28),
                        spin: rand::gen_range(-0.7, 0.7),
                        grav: 0.0, drag: 0.0,
                    },
                };
                self.particles.push(p);
            }
        }
    }

    /// A radial burst of `kind` particles, used for every puzzle result.
    pub fn burst(&mut self, x: f32, y: f32, kind: u8, n: usize) {
        for i in 0..n {
            let ang = rand::gen_range(0.0, std::f32::consts::TAU);
            let sp = rand::gen_range(60.0, 240.0);
            self.particles.push(Particle {
                x, y,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp - 50.0,
                life: rand::gen_range(0.45, 1.05),
                max_life: 1.05,
                kind,
                size: rand::gen_range(16.0, 40.0)
                    * if i % 4 == 0 { 1.5 } else { 1.0 },
                rot: ang,
                spin: rand::gen_range(-5.0, 5.0),
                grav: 26.0,
                drag: 0.9,
            });
        }
    }

    /// A ring of particles expanding outward - the shape a shockwave makes, and
    /// much more legible than a radial spray for "something happened here".
    pub fn ring(&mut self, x: f32, y: f32, kind: u8, n: usize, speed: f32) {
        for i in 0..n {
            let ang = i as f32 / n as f32 * std::f32::consts::TAU;
            self.particles.push(Particle {
                x, y,
                vx: ang.cos() * speed,
                vy: ang.sin() * speed * 0.45,
                life: 0.7,
                max_life: 0.7,
                kind,
                size: rand::gen_range(16.0, 26.0),
                rot: ang,
                spin: 0.0,
                grav: 0.0,
                drag: 3.2,
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

    /// Did the player click on an actor/prop at screen pos (sx, sy)?
    /// Returns the world x to walk to and the action id.
    pub fn hit_click(&self, sx: f32, sy: f32) -> Option<(f32, &'static str)> {
        if sy < 340.0 || sy > 620.0 {
            return None;
        }
        let mut best: Option<(f32, f32, &'static str)> = None;
        for a in &self.actors {
            let d = (a.x - self.cam - sx).abs();
            if d < 42.0 && best.as_ref().map(|(bd, ..)| d < *bd).unwrap_or(true) {
                best = Some((d, a.x, a.action));
            }
        }
        for p in &self.props {
            let d = (p.x - self.cam - sx).abs();
            if d < 56.0 && best.as_ref().map(|(bd, ..)| d < *bd).unwrap_or(true) {
                best = Some((d, p.x, p.action));
            }
        }
        best.map(|(_, x, act)| (x, act))
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

    pub fn draw(&self, tex: &Textures, fx: &Fx, t: f32, shake: f32) {
        let cam_x = self.cam + shake;
        let (sw, sh) = (screen_width(), screen_height());
        let ground_y = 560.0; // decor baseline (props/plants stand on this line)

        // biome decor
        match self.biome {
            Biome::Candy => {
                let tp = &tex.tiles;
                // Fairy-tale candy castle on the horizon: spaced towers with
                // crenellations and a pink-roofed keep, tinted into the sky
                // haze so they read as distant silhouette. Drawn first so the
                // bush line overlaps its base.
                let seg = 2260.0;
                let tint = c(226, 176, 216, 255); // pulls lilac stone into the pink
                let mut tx = seg;
                let mut ti = 0u32;
                while tx < self.length + 200.0 {
                    let sx = tx - cam_x;
                    if sx > -170.0 && sx < sw + 170.0 {
                        let seed = ti.wrapping_mul(2654435761) as u64;
                        let tw_cols = 2 + (seed % 2) as i32; // 2-3 tiles wide
                        let mut rows = 3 + (seed >> 8) % 3; // 3-5 tiles tall
                        let is_keep = ti % 4 == 2;
                        if is_keep { rows += 2; } // keeps: 5-7, roof stays in frame
                        let base_y = 452.0;
                        for r in 0..rows {
                            let y = base_y - (r as f32 + 1.0) * 48.0;
                            for ccol in 0..tw_cols {
                                let tile = if (r as i32 + ccol) % 2 == 0 { &tp.stone } else { &tp.stone_alt };
                                draw_texture_ex(tile, sx + ccol as f32 * 48.0, y, tint,
                                    DrawTextureParams { dest_size: Some(Vec2::new(48.0, 48.0)), ..Default::default() });
                            }
                        }
                        // crenellations: merlons along the top
                        let ytop = base_y - (rows as f32 + 1.0) * 48.0;
                        for ccol in 0..tw_cols {
                            if ccol % 2 == 0 {
                                draw_texture_ex(&tp.stone_alt, sx + ccol as f32 * 48.0, ytop + 24.0, tint,
                                    DrawTextureParams { dest_size: Some(Vec2::new(48.0, 24.0)), ..Default::default() });
                            }
                        }
                        if is_keep {
                            // pink conical roof marks the keep
                            let apex = Vec2::new(sx + tw_cols as f32 * 24.0, ytop - 52.0);
                            let bl = Vec2::new(sx - 10.0, ytop + 22.0);
                            let br = Vec2::new(sx + tw_cols as f32 * 48.0 + 10.0, ytop + 22.0);
                            draw_triangle(apex, bl, br, c(214, 96, 156, 255));
                        } else {
                            // tiny dark window slit
                            draw_rectangle(sx + tw_cols as f32 * 20.0,
                                base_y - rows as f32 * 48.0 * 0.62, 8.0, 14.0, c(96, 62, 116, 255));
                        }
                    }
                    tx += 250.0 + (ti % 3) as f32 * 90.0;
                    ti += 1;
                }
                // low curtain wall seam at the meadow line
                let mut x = seg;
                while x < self.length {
                    let sx = x - cam_x;
                    if sx > -48.0 && sx < sw + 48.0 {
                        let tile = if ((x / 48.0) as i32) % 2 == 0 { &tp.stone } else { &tp.stone_alt };
                        draw_texture_ex(tile, sx, 452.0, tint,
                            DrawTextureParams { dest_size: Some(Vec2::new(48.0, 48.0)), ..Default::default() });
                    }
                    x += 48.0;
                }
                // Distant candy bushes lining the meadow horizon.
                for i in 0..((self.length / 140.0) as i32) {
                    let x = i as f32 * 140.0 + 90.0;
                    let sx = x - cam_x;
                    if sx < -70.0 || sx > sw + 70.0 { continue; }
                    match i % 4 {
                        0 => draw_tile(&tp.bush_base, sx - 24.0, ground_y - 96.0, 48.0),
                        1 => draw_tile(&tp.bush_top, sx - 24.0, ground_y - 96.0, 48.0),
                        2 => {
                            draw_tile(&tp.bush_top, sx - 24.0, ground_y - 144.0, 48.0);
                            draw_tile(&tp.bush_base, sx - 24.0, ground_y - 96.0, 48.0);
                        }
                        _ => draw_tile(&tp.bush_mid, sx - 24.0, ground_y - 96.0, 48.0),
                    }
                    // contact shadow: bushes sat on the meadow with no grounding
                    draw_ellipse(sx, ground_y + 4.0, 30.0, 7.0, 0.0, c(70, 40, 20, 46));
                }

                // Meadow midground: gumdrops, candy canes and lollipops. A single
                // repeated bush shape read as a pattern rather than as a place,
                // so the meadow gets three real props on a stable spacing.
                let step2 = 236.0;
                let f2 = (cam_x / step2).floor() as i32 - 1;
                let l2 = f2 + (sw / step2) as i32 + 3;
                for i in f2..l2 {
                    let mut seed = (i as i64).wrapping_mul(40503) as u64;
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(5);
                    let x = i as f32 * step2 + ((seed >> 11) % 96) as f32;
                    let sx = x - cam_x;
                    if sx < -170.0 || sx > sw + 170.0 {
                        continue;
                    }
                    match (seed >> 20) % 3 {
                        // gumdrop mound: lit top, sugar-crust base, soft halo
                        0 => {
                            let r = 32.0 + ((seed >> 24) % 24) as f32;
                            let col = [c(255, 128, 176, 255), c(146, 224, 214, 255),
                                c(255, 200, 118, 255)][((seed >> 30) % 3) as usize];
                            let cy = ground_y - r * 0.52;
                            fx.light_sprite(sx, cy, r * 3.6, Color::new(col.r, col.g, col.b, 0.13),
                                ((seed >> 6) % 3) as usize);
                            draw_ellipse(sx, ground_y + 2.0, r * 0.96, r * 0.20, 0.0, c(96, 44, 30, 40));
                            draw_ellipse(sx, cy, r, r * 0.94, 0.0, col);
                            draw_ellipse(sx - r * 0.26, cy + r * 0.16, r * 0.66, r * 0.60,
                                0.0, Color::new(0.0, 0.0, 0.0, 0.10));
                            draw_ellipse(sx - r * 0.30, cy - r * 0.36, r * 0.28, r * 0.20, -0.5,
                                Color::new(1.0, 1.0, 1.0, 0.55));
                            draw_ellipse(sx, cy + r * 0.72, r * 0.98, r * 0.13, 0.0,
                                Color::new(1.0, 0.88, 0.95, 0.40));
                        }
                        // candy cane: banded stripes with a hooked crook
                        1 => {
                            let hh = 86.0 + ((seed >> 26) % 64) as f32;
                            let top = ground_y - hh;
                            draw_ellipse(sx + 2.0, ground_y + 2.0, 17.0, 5.0, 0.0, c(96, 44, 30, 46));
                            for k in 0..(hh / 12.0) as i32 {
                                let y = top + k as f32 * 12.0;
                                let col = if k % 2 == 0 { c(255, 250, 246, 255) } else { c(232, 62, 108, 255) };
                                draw_rectangle(sx - 5.0, y, 10.0, 12.0, col);
                                draw_rectangle(sx - 5.0, y, 3.5, 12.0,
                                    Color::new(0.0, 0.0, 0.0, 0.10));
                                draw_rectangle(sx + 2.5, y, 2.5, 12.0,
                                    Color::new(1.0, 1.0, 1.0, 0.22));
                            }
                            draw_circle_lines(sx + 2.0, top - 3.0, 12.0, 10.0, c(232, 62, 108, 255));
                            draw_circle_lines(sx + 2.0, top - 3.0, 12.0, 4.0, c(255, 250, 246, 255));
                        }
                        // swirl lollipop: real twirl sprite over a soft halo
                        _ => {
                            let hh = 66.0 + ((seed >> 26) % 54) as f32;
                            let top = ground_y - hh;
                            draw_ellipse(sx, ground_y + 2.0, 18.0, 5.0, 0.0, c(96, 44, 30, 42));
                            draw_rectangle(sx - 3.0, top, 6.0, hh, c(248, 242, 230, 255));
                            draw_rectangle(sx - 3.0, top, 2.0, hh, Color::new(0.0, 0.0, 0.0, 0.12));
                            let r = 28.0 + ((seed >> 30) % 16) as f32;
                            let cy = top - r * 0.34;
                            let tsw = t * 0.45 + (seed % 100) as f32 * 0.06;
                            fx.light_sprite(sx, cy, r * 3.8, Color::new(1.0, 0.46, 0.72, 0.13),
                                ((seed >> 9) % 3) as usize);
                            draw_circle(sx, cy, r, c(255, 118, 170, 255));
                            draw_circle(sx, cy, r * 0.86, c(255, 146, 190, 255));
                            let sw2 = &fx.sprites.twirl[((seed >> 5) % 2) as usize];
                            draw_spr(sw2, sx, cy, r * 1.9, tsw, Color::new(1.0, 0.97, 0.99, 0.88));
                            draw_circle(sx - r * 0.34, cy - r * 0.36, r * 0.16,
                                Color::new(1.0, 1.0, 1.0, 0.55));
                        }
                    }
                }
            }
            Biome::Aquarius => {
                // Distant kelp drifting behind everything. Only the seaweed
                // silhouettes are used: the pack's background rocks are solid
                // filled rectangles, which tiled out as a blocky skyline.
                let aq = &tex.aqua;
                let span = 300.0;
                let base = (cam_x * 0.3).rem_euclid(span);
                for i in -1..(sw / span) as i32 + 2 {
                    let sx = i as f32 * span - base;
                    let seed = (i as i64).wrapping_mul(2654435761) as u64;
                    let kind = ((seed >> 6) % aq.bg.len() as u64) as usize;
                    let bw = &aq.bg[kind];
                    if bw.width() > bw.height() * 1.4 {
                        continue; // rock slabs, not kelp
                    }
                    let h = bw.height() * (2.0 + ((seed >> 3) % 34) as f32 * 0.05);
                    let w2 = bw.width() * (h / bw.height());
                    let sway = (t * 1.2 + (seed % 37) as f32 * 0.05).sin() * 9.0;
                    draw_texture_ex(bw, sx - w2 / 2.0 + sway * 0.6, 500.0 - h, Color::new(0.45, 0.85, 0.75, 0.30),
                        DrawTextureParams { dest_size: Some(Vec2::new(w2, h)), ..Default::default() });
                }
                // schools of real fish weaving between depths
                for j in 0..5 {
                    let seed = (j as u64).wrapping_mul(6364136223846793005).wrapping_add(11);
                    let speed = 24.0 + ((seed >> 40) % 46) as f32;
                    let offset = ((seed >> 20) % self.length as u64) as f32;
                    let y = 165.0 + ((seed >> 8) % 210) as f32;
                    let p = (t * speed + offset) % (self.length * 2.0);
                    let right = p < self.length;
                    let x = if right { p } else { self.length * 2.0 - p };
                    let sx = x - cam_x;
                    if sx < -140.0 || sx > sw + 140.0 { continue; }
                    let kind = ((seed >> 32) % aq.fish.len() as u64) as usize;
                    let fw = &aq.fish[kind];
                    let dw = fw.width() * 0.6;
                    let dh = fw.height() * 0.6;
                    let bob = (t * 2.2 + j as f32 * 1.3).sin() * 5.0;
                    let ddir = if right { dw } else { -dw };
                    draw_texture_ex(fw, sx - ddir.abs() / 2.0, y + bob, WHITE,
                        DrawTextureParams { dest_size: Some(Vec2::new(ddir, dh)), ..Default::default() });
                }
            }
            Biome::Cosma => {
                // Real set pieces at three parallax depths (Space Kit + Planets
                // packs, recolored to the neon tokens by the asset pipeline).
                let cs = &tex.cosma;
                // far layer: big drifting planets, sparse and slow
                let span = 1500.0;
                let lim = (sw / span).ceil() as i32 + 2;
                for k in -1..=lim {
                    let x = k as f32 * span - (cam_x * 0.05).rem_euclid(span) + (t * 2.0 + k as f32 * 7.0);
                    if x < -380.0 || x > sw + 380.0 { continue; }
                    let seed = (k as i64).wrapping_mul(6364136223846793005).wrapping_add(31) as u64;
                    let p = &cs.planets[((seed >> 9) % cs.planets.len() as u64) as usize];
                    let size = 130.0 + ((seed >> 4) % 26) as f32 * 7.0;
                    let y = 70.0 + ((seed >> 33) % 120) as f32 + (t * 0.35 + k as f32).sin() * 5.0;
                    draw_texture_ex(p, x - size / 2.0, y, WHITE,
                        DrawTextureParams { dest_size: Some(Vec2::new(size, size)), ..Default::default() });
                }
                // mid layer: void boulders & meteors drifting between the stars
                let span = 640.0;
                let lim = (sw / span).ceil() as i32 + 2;
                for k in -1..=lim {
                    let x = k as f32 * span - (cam_x * 0.24).rem_euclid(span) + 90.0;
                    if x < -260.0 || x > sw + 260.0 { continue; }
                    let seed = (k as i64).wrapping_mul(2654435761) as u64;
                    let b = &cs.boulders[((seed >> 7) % cs.boulders.len() as u64) as usize];
                    let sc = 1.4 + ((seed >> 20) % 14) as f32 * 0.24;
                    let dw = b.width() as f32 * sc;
                    let dh = b.height() as f32 * sc;
                    let y = 250.0 + ((seed >> 4) % 190) as f32 + (t * 0.45 + k as f32 * 0.8).sin() * 9.0;
                    let dim = 0.5 + ((seed >> 33) % 24) as f32 * 0.018;
                    draw_texture_ex(b, x - dw / 2.0, y - dh / 2.0,
                        Color::new(0.82, 0.78, 1.0, dim),
                        DrawTextureParams { dest_size: Some(Vec2::new(dw, dh)), ..Default::default() });
                }
                // near layer: floating neon crystals bobbing above the walkway
                let span = 300.0;
                let lim = (sw / span).ceil() as i32 + 2;
                for k in -1..=lim {
                    let x = k as f32 * span - (cam_x * 0.82).rem_euclid(span) + 60.0;
                    if x < -220.0 || x > sw + 220.0 { continue; }
                    let seed = (k as i64).wrapping_mul(40503) as u64;
                    let c = &cs.crystals[((seed >> 11) % cs.crystals.len() as u64) as usize];
                    let sc = 1.7 + ((seed >> 4) % 10) as f32 * 0.3;
                    let dw = c.width() as f32 * sc;
                    let dh = c.height() as f32 * sc;
                    let base = 350.0 + ((seed >> 40) % 170) as f32;
                    let y = base + (t * 0.9 + k as f32 * 1.4).sin() * 14.0;
                    let glow = Color::new(0.9, 0.9, 1.0, 0.9);
                    draw_texture_ex(c, x - dw / 2.0, y, glow,
                        DrawTextureParams { dest_size: Some(Vec2::new(dw, dh)), ..Default::default() });
                    // cool light pooling on the stones below a lit crystal
                    let a = 20.0 + 14.0 * (t * 1.6 + k as f32).sin().abs();
                    draw_ellipse(x, 560.0, dw * 0.4, 6.0, 0.0, Color::from_rgba(200, 160, 255, a as u8));
                }
                // rim sparkles skimming the ledge
                let span = 190.0;
                let lim = (sw / span).ceil() as i32 + 2;
                for k in -1..=lim {
                    let x = k as f32 * span - (cam_x * 0.9).rem_euclid(span) + 50.0;
                    if x < -120.0 || x > sw + 120.0 { continue; }
                    let seed = (k as i64).wrapping_mul(6364136223846793005).wrapping_add(9) as u64;
                    let p = &cs.pebbles[((seed >> 21) % cs.pebbles.len() as u64) as usize];
                    let sc = 1.1 + ((seed >> 9) % 8) as f32 * 0.2;
                    let dw = p.width() as f32 * sc;
                    let dh = p.height() as f32 * sc;
                    let y = 542.0 + ((seed >> 33) % 12) as f32 + (t * 1.3 + k as f32).sin() * 3.0;
                    draw_texture_ex(p, x - dw / 2.0, y, Color::new(1.0, 1.0, 1.0, 0.95),
                        DrawTextureParams { dest_size: Some(Vec2::new(dw, dh)), ..Default::default() });
                }
                // the far edge of the world: a dimension tear behind the circle
                let rx = self.length - 320.0 - cam_x;
                if rx > -420.0 && rx < sw + 420.0 {
                    let pulse = (t * 2.0).sin();
                    for band in 0..6 {
                        let off = (t * 1.4 + band as f32 * 1.05).sin() * 24.0;
                        let c = Color::from_rgba(255, (110 + band * 22) as u8, (150 + band * 16) as u8, 70);
                        draw_rectangle(rx + off - 4.0, ground_y - 340.0 + band as f32 * 12.0, 8.0, 30.0, c);
                    }
                    let halo = Color::from_rgba(255, 120, 200, (26.0 + pulse.abs() * 40.0) as u8);
                    draw_circle(rx, ground_y - 250.0, 60.0, halo);
                    draw_rectangle_lines(rx - 26.0, ground_y - 240.0, 52.0, 8.0, 1.5, Color::from_rgba(255, 220, 255, 90));
                }
            }
            Biome::Home => {
                let tp = &tex.tiles;
                // plank floor: real planks tile, warm lamplight wash in the
                // middle of the room
                let size = 48.0;
                let first = (cam_x / size).floor() as i32 - 1;
                let last = first + (sw / size) as i32 + 3;
                for i in first..last {
                    let sx = i as f32 * size - cam_x;
                    let h = (i as i64).wrapping_mul(2654435761) as u32;
                    let tile = if h % 7 == 0 { &tp.wood_wall_alt } else { &tp.wood_wall };
                    draw_tile(tile, sx, ground_y, size);
                    draw_tile(&tp.planks, sx, ground_y + size, size);
                    if ground_y + size * 2.0 < sh {
                        draw_tile(&tp.planks, sx, ground_y + size * 2.0, size);
                    }
                }
                let lamp_w = 200.0;
                for (i, lx) in [430.0, 980.0].iter().enumerate() {
                    let x = lx - cam_x;
                    if x > -lamp_w && x < sw + lamp_w {
                        let flick = 0.86 + 0.14 * (t * 7.0 + i as f32 * 2.0).sin();
                        fx.glow_rect(x, ground_y + 60.0, lamp_w * 2.2 * flick, 240.0 * flick,
                            Color::from_rgba(255, 206, 138, 34));
                    }
                }
                // back wall band with a window and the hearth glow
                draw_rectangle(0.0, ground_y - 150.0, sw, 150.0, Color::from_rgba(52, 34, 48, 255));
                for i in first..last {
                    let sx = i as f32 * size - cam_x;
                    draw_tile(&tp.wood_wall, sx, ground_y - 150.0, size);
                    draw_tile(&tp.wood_wall, sx, ground_y - 102.0, size);
                    draw_tile(&tp.wood_wall, sx, ground_y - 54.0, size);
                }
                let wx = 700.0 - cam_x;
                if wx > -80.0 && wx < sw + 80.0 {
                    // night window: two panes of deep blue with a star
                    draw_tile(&tp.window, wx - 24.0, ground_y - 126.0, 48.0);
                    draw_rectangle(wx - 2.0, ground_y - 126.0, 4.0, 48.0, Color::from_rgba(40, 28, 40, 255));
                    let tw = 0.5 + 0.5 * (t * 2.0).sin();
                    draw_rectangle(wx + 14.0, ground_y - 112.0, 3.0, 3.0,
                        Color::from_rgba(255, 240, 200, (120.0 + tw * 120.0) as u8));
                }
                // front door + bookshelf beside it
                let dx = 200.0 - cam_x;
                if dx > -80.0 && dx < sw + 80.0 {
                    draw_tile(&tp.door, dx - 24.0, ground_y - 96.0, 48.0);
                    draw_tile(&tp.shelf, dx - 96.0, ground_y - 96.0, 48.0);
                    draw_tile(&tp.shelf, dx - 96.0, ground_y - 48.0, 48.0);
                }
                // lit hearth on the far wall
                let hx = 1220.0 - cam_x;
                if hx > -80.0 && hx < sw + 80.0 {
                    let flick = 0.75 + 0.25 * (t * 8.0).sin();
                    // the hearth is the room's light source: far pool first, then
                    // a live flame sprite, so it lights the floor it sits on
                    fx.glow_rect(hx, ground_y - 26.0, 520.0 * flick, 470.0 * flick,
                        Color::from_rgba(255, 148, 68, 22));
                    fx.glow_rect(hx, ground_y - 18.0, 240.0 * flick, 220.0 * flick,
                        Color::from_rgba(255, 178, 96, 34));
                    draw_tile(&tp.fire_dark, hx - 24.0, ground_y - 96.0, 48.0);
                    draw_tile(&tp.fire_lit, hx - 24.0, ground_y - 48.0, 48.0);
                    let fl = &fx.sprites.fire[0];
                    draw_spr(fl, hx, ground_y - 34.0, 96.0 + flick * 10.0,
                        (t * 0.4).sin() * 0.05, Color::from_rgba(255, 198, 122, 228));
                    fx.glow_rect(hx, ground_y + 4.0, 320.0 * flick, 90.0,
                        Color::from_rgba(255, 160, 76, 40));
                }
                // rug: real tile, drawn wide
                let rug_x = 900.0 - cam_x;
                if rug_x > -260.0 && rug_x < sw + 260.0 {
                    for (i, j) in [(0i32, 0i32), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1), (0, 2), (1, 2), (2, 2)] {
                        draw_tile(&tp.rug, rug_x - 72.0 + i as f32 * 48.0, ground_y + 46.0 + j as f32 * 20.0 - 10.0, 48.0);
                    }
                }
            }
        }

        // butterflies drifting over the meadow
        if self.biome == Biome::Candy {
            for i in 0..4 {
                let base = 340.0 + i as f32 * 760.0;
                let cx_ = base - cam_x + (t * 0.55 + i as f32 * 1.3).sin() * 90.0;
                if cx_ < -40.0 || cx_ > sw + 40.0 {
                    continue;
                }
                let cy_ = 236.0 + ((i as f32 * 97.0) % 70.0) + (t * 0.8 + i as f32).sin() * 34.0;
                let flap = (t * 13.0 + i as f32 * 2.1).sin().abs().max(0.15);
                let col = if i % 2 == 0 {
                    Color::from_rgba(255, 140, 190, 235)
                } else {
                    Color::from_rgba(255, 222, 120, 235)
                };
                draw_ellipse(cx_ - 4.0, cy_, 6.5 * flap + 0.5, 4.2, -0.4, col);
                draw_ellipse(cx_ + 4.0, cy_, 6.5 * flap + 0.5, 4.2, 0.4, col);
                draw_rectangle(cx_ - 0.5, cy_ - 4.0, 1.2, 8.0, Color::from_rgba(90, 50, 80, 255));
            }
        }

        // ground strip
        if self.biome == Biome::Candy {
            // Lemon-tile meadow, textured ground with scattered flowers.
            let tp = &tex.tiles;
            let size = 48.0;
            let row0 = 500.0;
            let first = (cam_x / size).floor() as i32 - 1;
            let last = first + (sw / size).ceil() as i32 + 2;
            let nrow = ((sh - row0) / size).ceil() as i32 + 1;
            for row in 0..nrow {
                let y = row0 + row as f32 * size;
                for i in first..last {
                    let wx = i as f32 * size;
                    if wx < -size || wx > self.length + size { continue; }
                    let sx = wx - cam_x;
                    // Hash the CELL, not the column: keying on i alone made every
                    // speckled or flowering tile repeat down the whole screen as
                    // a vertical stripe.
                    let h = ((i as i64).wrapping_mul(2654435761)
                        ^ (row as i64).wrapping_mul(40503)) as u32;
                    let base = if h % 31 == 0 { &tp.ground_speck } else if h % 37 == 0 { &tp.ground_alt } else { &tp.ground };
                    // Per-cell brightness keeps the repeating 16px pattern from
                    // aligning into visible boxes across the meadow.
                    let shade = 1.0 + ((h >> 9) % 11) as f32 * 0.006 - 0.03;
                    draw_tile_tint(base, sx, y, size, Color::new(shade, shade * 0.995, shade * 0.975, 1.0));
                    if row > 0 && h % 211 == 0 {
                        draw_tile(&tp.dirt, sx, y, size);
                    }
                    if row == 0 && h % 23 == 0 {
                        let f = match h % 3 { 0 => &tp.flower_a, 1 => &tp.flower_b, _ => &tp.flower_c };
                        draw_tile(f, sx, y, size);
                    }
                }
            }
        } else if self.biome == Biome::Cosma {
            // floating void-castle walkway: a lit neon rim over deep indigo
            // castle stone that fades into the abyss toward the screen bottom
            let cs = &tex.cosma;
            let size = 48.0;
            let first = (cam_x / size).floor() as i32 - 1;
            let last = first + (sw / size).ceil() as i32 + 2;
            let nrow = ((sh - ground_y) / size).ceil() as i32 + 1;
            for row in 0..nrow {
                let y = ground_y + row as f32 * size;
                for i in first..last {
                    let wx = i as f32 * size;
                    if wx < -size || wx > self.length + size {
                        continue;
                    }
                    let sx = wx - cam_x;
                    if row == 0 {
                        draw_tile(&cs.rim, sx, y, size);
                    } else {
                        let h = (i as i64).wrapping_mul(2654435761) as u32;
                        // 0..3 body blocks; 4/5 rare baked neon veins
                        let idx = if h % 97 == 0 {
                            3 + (h % 2) as usize
                        } else if h % 31 == 0 {
                            2
                        } else if h % 5 == 0 {
                            1
                        } else {
                            0
                        };
                        draw_tile(&cs.floor[idx], sx, y, size);
                        // rare OGA Neon Node circuit panels inlaid in the masonry
                        if h % 53 == 0 {
                            let nn = &cs.neon_nodes[(h % 5) as usize];
                            draw_texture_ex(nn, sx + 4.0, y + 4.0, WHITE,
                                DrawTextureParams { dest_size: Some(Vec2::new(size - 8.0, size - 8.0)), ..Default::default() });
                        }
                    }
                }
            }
            // soft distance fog: the ledge dims into the void below
            let steps = 6;
            for s in 0..steps {
                let f = s as f32 / steps as f32;
                let yy = ground_y + 118.0 + f * (sh - ground_y - 118.0);
                let a = (46.0 + f * 130.0) as u8;
                draw_rectangle(0.0, yy, sw, 26.0, Color::from_rgba(5, 4, 14, a));
            }
            // faint violet edge-light bleeding just under the rim
            let flick = 0.5 + 0.5 * (t * 1.8).sin();
            draw_rectangle(0.0, ground_y + 46.0, sw, 2.0,
                Color::from_rgba(165, 130, 255, (26.0 + flick * 26.0) as u8));
        } else if self.biome == Biome::Aquarius {
            // textured sea floor: wet sand rim, dune shading, grain, caustics
            draw_rectangle(0.0, ground_y, sw, sh - ground_y, Color::from_rgba(194, 178, 128, 255));
            // wet sand where the water column meets the floor
            draw_rectangle(0.0, ground_y, sw, 9.0, Color::from_rgba(163, 145, 101, 255));
            draw_rectangle(0.0, ground_y + 9.0, sw, 2.0, Color::from_rgba(140, 122, 82, 255));
            // gentle dune shading bands
            let band_step = 40.0;
            let first_b = (cam_x / band_step).floor() as i32 - 1;
            let n_bands = (sw / band_step) as i32 + 3;
            for b in first_b..first_b + n_bands {
                let h = (b as i64).wrapping_mul(2654435761) as u32;
                if h % 3 == 0 { continue; }
                let bx = b as f32 * band_step - cam_x;
                let bw = 24.0 + (h % 44) as f32;
                let by = ground_y + 18.0 + (h % 100) as f32;
                if by > sh - 8.0 { continue; }
                let col = if h % 2 == 0 { Color::from_rgba(174, 157, 110, 255) } else { Color::from_rgba(209, 193, 147, 255) };
                draw_rectangle(bx, by, bw, 4.0 + (h % 6) as f32, col);
            }
            // sand grain speckles, world-anchored so they scroll with the floor
            let step = 13.0;
            let first = (cam_x / step).floor() as i32 - 1;
            let last = first + (sw / step) as i32 + 3;
            for i in first..last {
                let h = (i as i64).wrapping_mul(2654435761) as u32;
                let sx = i as f32 * step - cam_x + (h % 9) as f32;
                let sy = ground_y + 14.0 + ((h >> 5) % ((sh as u32).saturating_sub(ground_y as u32 + 22)).max(1)) as f32;
                let col = if h % 2 == 0 { Color::from_rgba(215, 199, 153, 255) } else { Color::from_rgba(167, 149, 103, 255) };
                draw_rectangle(sx, sy, 2.0, 2.0, col);
            }
            // caustic light: faint wavy bright lines rippling over the sand
            for k in 0..3 {
                let yy = ground_y + 26.0 + k as f32 * 24.0;
                let n = (sw / 8.0) as i32 + 1;
                for x in 0..n {
                    let px = x as f32 * 8.0;
                    let wave = ((px + cam_x) * 0.045 + t * 1.3 + k as f32 * 2.1).sin();
                    let a = (12.0 + 10.0 * wave).max(0.0) as u8;
                    if a == 0 { continue; }
                    draw_rectangle(px, yy + wave * 3.0, 8.0, 2.0, Color::from_rgba(255, 244, 214, a));
                }
            }
        } else {
            let ground = Color::from_rgba(154, 108, 74, 255);
            draw_rectangle(0.0, ground_y, sw, sh - ground_y, ground);
            if self.biome != Biome::Home {
                draw_rectangle(0.0, 556.0, sw, 4.0, Color::from_rgba(0, 0, 0, 40));
            }
        }

        // Aquarius foreground: swaying seaweed and rocks rooted in the sand
        if self.biome == Biome::Aquarius {
            let aq = &tex.aqua;
            let base = 560.0;
            let span = 320.0;
            let first = (cam_x / span).floor() as i32 - 1;
            let last = first + (sw / span) as i32 + 3;
            for i in first..last {
                let seed = (i as i64).wrapping_mul(2654435761) as u64;
                let x = i as f32 * span + ((seed >> 11) % 170) as f32;
                let sx = x - cam_x;
                if sx < -180.0 || sx > sw + 180.0 {
                    continue;
                }
                let wk = ((seed >> 4) % aq.weed.len() as u64) as usize;
                let w = &aq.weed[wk];
                // Rooted foreground weed: tall enough to frame the sand line,
                // short enough to never cover River's body or the prompt chips.
                let dh = w.height() * 1.35;
                let dw = w.width() * 1.35;
                let sway = (t * 1.8 + i as f32 * 0.7).sin() * 9.0;
                draw_texture_ex(w, sx - dw / 2.0 + sway * 0.55, base - dh, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(dw, dh)), ..Default::default() });
                if seed % 3 == 0 {
                    let rk = ((seed >> 3) % aq.rock.len() as u64) as usize;
                    let r = &aq.rock[rk];
                    let rh = r.height() * 1.7;
                    let rw2 = r.width() * 1.7;
                    let off = ((seed >> 9) % 120) as f32 - 60.0;
                    draw_texture_ex(r, sx + off - rw2 / 2.0, base - rh, WHITE,
                        DrawTextureParams { dest_size: Some(Vec2::new(rw2, rh)), ..Default::default() });
                }
            }
        }

        // props
        for p in &self.props {
            let sx = p.x - cam_x;
            if sx < -220.0 || sx > sw + 220.0 { continue; }
            match p.id {
                "gate" => {
                    // The Lone Stone Story of Throne Gate: a carved slab monolith
                    // with a dark story-arch recess and drifting rune sparks.
                    let tp = &tex.tiles;
                    let x0 = sx - p.w / 2.0;
                    let top = ground_y - p.h;
                    let tsize = 46.0;
                    let cols = (p.w / tsize).ceil() as i32;
                    let rows = (p.h / tsize).ceil() as i32;
                    for r in 0..rows {
                        for c in 0..cols {
                            let tile = if (r + c) % 2 == 0 { &tp.stone } else { &tp.stone_alt };
                            draw_tile(tile, x0 + c as f32 * tsize, top + r as f32 * tsize, tsize);
                        }
                    }
                    let aw = p.w * 0.56;
                    let ah = p.h * 0.66;
                    let dark = Color::from_rgba(24, 18, 36, 255);
                    draw_circle(sx, ground_y - ah, aw / 2.0 + 4.0, dark);
                    draw_rectangle(sx - aw / 2.0, ground_y - ah + aw / 4.0, aw, ah - aw / 4.0, dark);
                    draw_rectangle_lines(sx - aw / 2.0 - 4.0, ground_y - ah + 6.0, aw + 8.0, ah - 6.0, 4.0,
                        Color::from_rgba(205, 176, 220, 200));
                    for i in 0..3 {
                        let ph = t * 2.0 + i as f32 * 2.1;
                        let rx = sx + (ph * 1.6).sin() * (aw * 0.3);
                        let ry = ground_y - ah * 0.55 + (ph * 1.2).cos() * 20.0;
                        let a = 0.45 + 0.5 * (t * 3.2 + i as f32 * 1.9).sin().abs();
                        draw_circle(rx, ry, 1.5 + i as f32 * 0.9, Color::new(1.0, 0.62, 0.85, a * 0.9));
                    }
                }
                "cave" => {
                    // The Grave Cave: a rock arch mouth opening under the sand.
                    let mouth_x = sx;
                    let base = ground_y + 4.0;
                    let rock = Color::from_rgba(30, 24, 52, 255);
                    let rock_hi = Color::from_rgba(84, 60, 120, 255);
                    // entrance hole
                    draw_ellipse(mouth_x, base - 66.0, 92.0, 76.0, 0.0, rock);
                    draw_ellipse(mouth_x, base - 84.0, 58.0, 52.0, 0.0, Color::from_rgba(10, 8, 20, 255));
                    let glow = 0.5 + 0.5 * (t * 1.6).sin();
                    draw_ellipse(mouth_x, base - 84.0, 44.0 + glow * 5.0, 40.0, 0.0, Color::from_rgba(120, 80, 190, 20));
                    // rocky mound silhouettes framing the mouth
                    for (dx, dy, rr) in [(-104.0, -26.0, 58.0), (-70.0, -70.0, 50.0), (104.0, -26.0, 58.0), (70.0, -76.0, 52.0)] {
                        draw_circle(mouth_x + dx, base - 6.0 - dy * 0.4, rr, rock);
                        draw_circle(mouth_x + dx + 8.0, base - 12.0 - dy * 0.4, rr * 0.55, rock_hi);
                    }
                    // sandy ramp shadow under the entrance
                    draw_ellipse(mouth_x, base - 4.0, 100.0, 12.0, 0.0, Color::from_rgba(20, 12, 30, 120));
                }
                "portal" => {
                    // Stone circle that wakes when all three torches burn. In
                    // Cosma the standing stones are the void-castle block.
                    let tp = &tex.tiles;
                    let (stone_a, stone_b) = if self.biome == Biome::Cosma {
                        (&tex.cosma.floor[2], &tex.cosma.rim)
                    } else {
                        (&tp.stone, &tp.stone_alt)
                    };
                    let cy = ground_y - p.h * 0.44;
                    let ring = 96.0;
                    for i in 0..12 {
                        let a = i as f32 / 12.0 * std::f32::consts::TAU;
                        let rx = a.cos() * ring;
                        let ry = a.sin() * ring * 0.42;
                        if ry < -1.0 {
                            let tile = if i % 2 == 0 { stone_a } else { stone_b };
                            draw_tile(tile, sx + rx - 22.0, cy + ry - 38.0, 44.0);
                        }
                    }
                    let alive = self.torches_lit >= 3;
                    let pulse = 0.5 + 0.5 * (t * (if alive { 3.2 } else { 1.0 })).sin();
                    let col = if alive {
                        Color::from_rgba(190, 150, 255, (60.0 + pulse * 110.0) as u8)
                    } else {
                        Color::from_rgba(120, 110, 160, (26.0 + pulse * 30.0) as u8)
                    };
                    draw_circle(sx, cy, 46.0 + pulse * 9.0, col);
                    if alive {
                        for i in 0..5 {
                            let a = t * 0.9 + i as f32 * 1.26;
                            let rr = 24.0 + (a * 3.0).sin().abs() * 10.0;
                            draw_circle(sx + a.cos() * rr, cy + a.sin() * rr * 0.7, 2.0, Color::from_rgba(240, 215, 255, 230));
                        }
                    }
                }
                "torch1" | "torch2" | "torch3" => {
                    let tp = &tex.tiles;
                    let (stone_a, stone_b) = if self.biome == Biome::Cosma {
                        (&tex.cosma.floor[0], &tex.cosma.floor[2])
                    } else {
                        (&tp.stone, &tp.stone_alt)
                    };
                    let idx: u32 = p.id.get(5..6).and_then(|c| c.to_string().parse().ok()).unwrap_or(1);
                    let lit = idx <= self.torches_lit;
                    let flick = (t * 9.0 + idx as f32 * 2.0).sin() * 2.5;
                    // pedestal of castle stone
                    draw_tile(stone_a, sx - 16.0, ground_y - 26.0, 22.0);
                    draw_tile(stone_b, sx - 16.0, ground_y - 48.0, 22.0);
                    draw_rectangle(sx - 5.0, ground_y - p.h + 30.0, 10.0, p.h - 52.0, Color::from_rgba(70, 58, 92, 255));
                    if lit {
                        let fy = ground_y - p.h + 14.0;
                        // far halo -> near halo -> live flame -> floor pool
                        fx.glow_rect(sx, fy, 440.0 + flick * 10.0, 440.0 + flick * 10.0,
                            Color::from_rgba(255, 146, 58, 24));
                        fx.light_sprite(sx, fy, 170.0 + flick * 12.0,
                            Color::from_rgba(255, 176, 84, 120), idx as usize);
                        let fl = &fx.sprites.flame[(idx as usize) % fx.sprites.flame.len()];
                        draw_spr(fl, sx, fy, 50.0 + flick * 2.5,
                            (t * 0.7 + idx as f32).sin() * 0.13,
                            Color::from_rgba(255, 218, 152, 238));
                        draw_circle(sx, fy + 3.0, 6.0, Color::from_rgba(255, 246, 196, 240));
                        fx.glow_rect(sx, ground_y + 6.0, 250.0, 74.0,
                            Color::from_rgba(255, 170, 80, 36));
                    } else {
                        draw_rectangle(sx - 3.0, ground_y - p.h - 6.0, 6.0, 10.0, Color::from_rgba(50, 44, 64, 255));
                        draw_circle(sx, ground_y - p.h - 8.0, 7.0, Color::from_rgba(90, 78, 116, 255));
                    }
                }
                "bed" => {
                    let tp = &tex.tiles;
                    // real furniture tiles: headboard, mattress, footboard
                    let x0 = sx - p.w / 2.0;
                    draw_tile(&tp.bed_head, x0, ground_y - 58.0, 48.0);
                    let mid = p.w - 96.0;
                    if mid > 0.0 {
                        draw_texture_ex(&tp.bed_wide, x0 + 48.0, ground_y - 34.0, WHITE,
                            DrawTextureParams { dest_size: Some(Vec2::new(mid, 48.0)), ..Default::default() });
                    }
                    draw_tile(&tp.bed_foot, x0 + p.w - 48.0, ground_y - 40.0, 48.0);
                    // quilt band across the mattress
                    draw_rectangle(x0 + 6.0, ground_y - 26.0, p.w - 14.0, 10.0, Color::from_rgba(255, 150, 180, 255));
                    draw_rectangle(x0 + 6.0, ground_y - 26.0, p.w - 14.0, 3.0, Color::from_rgba(255, 190, 210, 255));
                    // pillow
                    draw_rectangle(x0 + 10.0, ground_y - 46.0, 30.0, 12.0, Color::from_rgba(250, 246, 238, 255));
                }
                "table" => {
                    let tp = &tex.tiles;
                    let x0 = sx - p.w / 2.0;
                    // tiled top with legs, and the heirloom book on a stand
                    let n = (p.w / 48.0).ceil() as i32;
                    for i in 0..n {
                        let tile = if i % 2 == 0 { &tp.table_a } else { &tp.table_b };
                        draw_tile(tile, x0 + i as f32 * 48.0, ground_y - 52.0, 48.0);
                        draw_tile(&tp.table_b, x0 + i as f32 * 48.0, ground_y - 14.0, 48.0);
                    }
                    draw_tile(&tp.table_a, x0, ground_y - 52.0, 48.0);
                    // the heirloom: glowing red book on the table
                    let glow = 0.5 + 0.5 * (t * 2.2).sin();
                    draw_circle(sx + 20.0, ground_y - 58.0, 14.0 + glow * 5.0, Color::from_rgba(255, 224, 140, 40 + (glow * 40.0) as u8));
                    draw_rectangle(sx + 8.0, ground_y - 68.0, 24.0, 18.0, Color::from_rgba(178, 66, 66, 255));
                    draw_rectangle(sx + 18.0, ground_y - 68.0, 4.0, 18.0, Color::from_rgba(110, 30, 30, 255));
                    draw_rectangle(sx + 11.0, ground_y - 63.0, 8.0, 3.0, Color::from_rgba(255, 224, 170, 180));
                }
                _ => {
                    // riddle stone: engraved monolith with a glowing rune;
                    // tinted to the biome's stone so it never reads as imported
                    let tp = &tex.tiles;
                    let stone_tint = match self.biome {
                        Biome::Aquarius => Color::from_rgba(148, 168, 176, 255), // sea-worn slate
                        Biome::Cosma => Color::from_rgba(120, 110, 168, 255),    // void indigo
                        _ => Color::from_rgba(255, 255, 255, 255),
                    };
                    let s = 46.0;
                    let top = ground_y - p.h; // riddle props are 90 tall
                    let x0 = sx - s;
                    draw_tile_tint(&tp.stone, x0, top + 2.0, s, stone_tint);
                    draw_tile_tint(&tp.stone_alt, x0 + s, top + 2.0, s, stone_tint);
                    draw_tile_tint(&tp.stone_alt, x0, top + 2.0 + s, s, stone_tint);
                    draw_tile_tint(&tp.stone, x0 + s, top + 2.0 + s, s, stone_tint);
                    // carved seam down the middle
                    draw_rectangle(sx - 2.0, top + 6.0, 4.0, 86.0, Color::from_rgba(70, 48, 88, 255));
                    // pulsing rune in the token pink, with a light pool so the
                    // monolith looks lit rather than merely coloured
                    let pulse = 0.55 + 0.45 * (t * 3.0 + sx * 0.1).sin().abs();
                    fx.glow_rect(sx, top + 34.0, 132.0, 132.0,
                        Color::new(1.0, 0.42, 0.66, 0.08 + pulse * 0.14));
                    let qw = dlg_width("?", 46);
                    draw_dlg_text("?", sx - qw / 2.0, top + 48.0, 46, Color::new(1.0, 0.55, 0.72, pulse));
                    // tufts at the base: grass in dry biomes, weed in the reef
                    let tuft = if self.biome == Biome::Aquarius { Color::from_rgba(70, 160, 110, 255) } else { Color::from_rgba(190, 168, 60, 255) };
                    draw_rectangle(sx - 30.0, ground_y - 6.0, 3.0, 6.0, tuft);
                    draw_rectangle(sx + 26.0, ground_y - 8.0, 3.0, 8.0, tuft);
                }
            }
        }

        // actors: stand/walk frames + drop shadow + styled name chip
        let pxx = self.player.0;
        for a in &self.actors {
            let sx = a.x - cam_x;
            if sx < -110.0 || sx > sw + 110.0 { continue; }
            let scale = a.scale;
            let w = 48.0 * scale;
            let h = 72.0 * scale;            let walking = a.speed > 0.0 && (a.x - pxx).abs() > REACH * 1.4;
            let texid = if a.anim.is_empty() { None } else { tex.walk_of(a.anim) };
            let breath = if walking {
                (t * 6.0 + a.x * 0.03).sin() * 1.6
            } else {
                (t * 2.6 + a.x * 0.01).sin() * 2.2
            };
            let sq = if walking { 1.0 + (t * 6.0 + a.x * 0.1).sin() * 0.05 } else { 1.0 };
            // baked contact shadow: cut from the sprite's own alpha feet, so it
            // is aligned per-character (Nadia small, Kandy wide, etc.)
            let feet = a.y + h;
            if let Some(sh) = tex.shadow_of(a.anim) {
                let sw2 = w * 1.02;
                draw_texture_ex(sh, sx - sw2 / 2.0, feet - 10.0 * scale, WHITE,
                    DrawTextureParams { dest_size: Some(Vec2::new(sw2, 22.0 * scale)), ..Default::default() });
            } else {
                draw_ellipse(sx, feet + 3.0, w * 0.44 / sq, 5.0 * scale, 0.0, Color::from_rgba(60, 30, 40, 60));
            }
            // stride: w1/w2 alternate, the sprite contains the full stride
            let phase = ((t * 5.2 + a.x * 0.02) * 2.0) as usize;
            let step = phase % 2;
            let frame: &Texture2D = match texid {
                Some(fr) if walking => &fr[step],
                _ => &a.tex,
            };
            let aw = w * a.face * sq;
            draw_texture_ex(frame, sx - aw.abs() / 2.0, a.y + breath, WHITE,
                DrawTextureParams { dest_size: Some(Vec2::new(aw, h / sq)), ..Default::default() });
            // name chip
            let tw = tag_width(a.label, 11);
            let chip_w = tw + 18.0;
            draw_rectangle(sx - chip_w / 2.0, a.y - 22.0, chip_w, 17.0, Color::from_rgba(74, 42, 99, 205));
            draw_rectangle_lines(sx - chip_w / 2.0, a.y - 22.0, chip_w, 17.0, 1.0, Color::from_rgba(255, 214, 230, 90));
            draw_tag(a.label, sx - tw / 2.0, a.y - 10.0, 11, Color::from_rgba(255, 246, 233, 255));
            // "!" prompt bubble when in range
            if (a.x - pxx).abs() < REACH {
                let bounce = (t * 5.0).sin() * 3.0;
                let by = a.y - 40.0 + bounce;
                draw_circle(sx, by, 11.0, Color::from_rgba(255, 214, 230, 248));
                draw_circle_lines(sx, by, 11.0, 2.0, Color::from_rgba(74, 42, 99, 255));
                draw_text("!", sx - 3.5, by + 4.5, 20.0, Color::from_rgba(74, 42, 99, 255));
            }
        }

        // player: walk cycle with a shadow cut from the sprite's own feet
        let px = pxx - cam_x;
        let ride = self.biome == Biome::Candy && crate::game::ride_unlocked();
        let hopf = if ride { 11.0 } else { 7.0 };
        let hop_phase = self.bob_t * hopf;
        let hop = if self.moving { hop_phase.sin() } else { (t * 2.0).sin() * 0.6 };
        let py = self.player.1 + hop * 2.3;
        // stride squash while moving, a slow breath while standing
        let squash = 1.0 + if self.moving {
            (hop_phase * 0.55).sin() * 0.05
        } else {
            (t * 2.0).sin() * 0.014
        };
        let feet = self.player.1 + 72.0;
        // shadow stays on the ground and breathes opposite the hop: the body
        // rises, the shadow shrinks — read as contact with the ground.
        let shadow_scale = if self.moving { 1.0 - hop.abs() * 0.18 } else { 1.0 };
        if let Some(sh) = tex.shadow_of("river") {
            let sw2 = 50.0 * shadow_scale;
            draw_texture_ex(sh, px - sw2 / 2.0, feet - 10.0, Color::new(1.0, 1.0, 1.0, 0.9),
                DrawTextureParams { dest_size: Some(Vec2::new(sw2, 22.0)), ..Default::default() });
        } else {
            draw_ellipse(px, feet + 4.0, 24.0 * shadow_scale / squash.max(0.9), 5.0, 0.0, Color::from_rgba(60, 30, 40, 85));
        }
        if ride && self.moving {
            for g in 1..4 {
                let gx = px - self.facing * g as f32 * 26.0;
                if gx < -60.0 || gx > sw + 60.0 { continue; }
                let alpha = 0.22 / g as f32;
                draw_texture_ex(&tex.river, gx - 24.0, py, Color::new(1.0, 0.9, 1.0, alpha), DrawTextureParams {
                    dest_size: Some(Vec2::new(48.0 * self.facing, 72.0)),
                    ..Default::default()
                });
            }
        }
        let frame: &Texture2D = if self.moving {
            match tex.walk_of("river") {
                Some(fr) => &fr[((hop_phase / std::f32::consts::PI).max(0.0) as usize + pxx as usize) % 2],
                None => &tex.river,
            }
        } else { &tex.river };
        let pw = 48.0 * self.facing * squash;
        draw_texture_ex(frame, px - pw.abs() / 2.0, py, WHITE,
            DrawTextureParams { dest_size: Some(Vec2::new(pw, 72.0 / squash)), ..Default::default() });

        // near-foreground fringe. It scrolls FASTER than the camera, so the
        // scene gains depth in front of the player as well as behind them -
        // the cheapest trick that stops an overworld reading as a flat diorama.
        if self.biome != Biome::Home {
            let fct = 1.22;
            let step = 150.0;
            let base = (cam_x * fct / step).floor() as i64;
            let n = (sw / step) as i64 + 3;
            for j in -1..n {
                let idx = base + j;
                let sxx = idx as f32 * step - cam_x * fct;
                let mut seed = (idx as u64).wrapping_mul(2654435761).wrapping_add(99);
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(17);
                match self.biome {
                    // deep-magenta candy grass, well below the meadow's own hue
                    // so it reads as being out of focus rather than as a plant
                    Biome::Candy => {
                                                for b in 0..6u64 {
                            let bs = seed.wrapping_mul(b + 1);
                            let bx = sxx + ((bs >> 11) % 150) as f32;
                            let hh = 26.0 + ((bs >> 20) % 34) as f32;
                            let lean = ((bs >> 30) % 20) as f32 * 0.07 - 0.7;
                            let sway = (t * 1.1 + bx * 0.02).sin() * 6.0;
                            let tip = Vec2::new(bx + lean * hh * 0.7 + sway, 800.0 - hh);

                            // out-of-focus blade: translucent body plus a lit inner
                            // edge, so it frames the shot without reading as a spike
                            // or hiding the meadow it is supposed to border
                            draw_triangle(Vec2::new(bx - 6.0, 800.0), Vec2::new(bx + 6.0, 800.0),
                                tip, c(120, 40, 82, 88));
                            draw_triangle(Vec2::new(bx - 3.0, 800.0), Vec2::new(bx + 2.0, 800.0),
                                Vec2::new(tip.x - 1.0, tip.y + 8.0), c(198, 78, 138, 96));
                        }
                    }
                    // out-of-focus coral: the real weed art, scaled up and
                    // pushed to silhouette
                    Biome::Aquarius => {
                        let aq = &tex.aqua;
                        for b in 0..3u64 {
                            let bs = seed.wrapping_mul(b + 1);
                            let wk = ((bs >> 4) % aq.weed.len() as u64) as usize;
                            let w2 = &aq.weed[wk];
                            // Never taller than the shot: the old 4.6x-6.8x
                            // scale put a black kelp wall over the sand line.
                            let dh = w2.height() * (1.5 + ((bs >> 20) % 22) as f32 * 0.04);
                            let dw = w2.width() * (dh / w2.height());
                            let bx = sxx + ((bs >> 11) % 150) as f32;
                            let sway = (t * 1.5 + bx * 0.01).sin() * 12.0;
                            draw_texture_ex(w2, bx - dw / 2.0 + sway, 800.0 - dh, c(26, 66, 58, 190),
                                DrawTextureParams { dest_size: Some(Vec2::new(dw, dh)), ..Default::default() });
                        }
                    }
                    // void-stone lip with a neon rim catching the sky
                    Biome::Cosma => {
                        for b in 0..4u64 {
                            let bs = seed.wrapping_mul(b + 1);
                            let bx = sxx + ((bs >> 11) % 150) as f32;
                            let ww = 62.0 + ((bs >> 20) % 70) as f32;
                            let hh = 30.0 + ((bs >> 25) % 58) as f32;
                            let apex = bx + ((bs >> 30) % 22) as f32 - 11.0;
                            draw_triangle(Vec2::new(bx - ww * 0.5, 800.0),
                                Vec2::new(bx + ww * 0.5, 800.0),
                                Vec2::new(apex, 800.0 - hh), c(7, 4, 18, 255));
                            draw_line(bx - ww * 0.5, 800.0, apex, 800.0 - hh, 2.0, c(124, 92, 226, 130));
                        }
                    }
                    _ => {}
                }
            }
        }

        // guide arrow
        let nx = self.next_target_x() - cam_x;
        let bounce = (t * 4.0).sin() * 6.0;
        if nx > 60.0 && nx < sw + 60.0 {
            draw_triangle(Vec2::new(nx, 150.0 + bounce), Vec2::new(nx - 14.0, 130.0 + bounce), Vec2::new(nx + 14.0, 130.0 + bounce), Color::from_rgba(255, 105, 180, 220));
        } else if nx >= sw + 60.0 {
            draw_triangle(Vec2::new(sw - 30.0, 150.0 + bounce), Vec2::new(sw - 50.0, 130.0 + bounce), Vec2::new(sw - 14.0, 140.0 + bounce), Color::from_rgba(255, 105, 180, 220));
        }

        // particles: every family is a real sprite from the Kenney particle
        // kit, rotated and scaled per particle, instead of a coloured circle
        for p in &self.particles {
            let sx = p.x - cam_x;
            if sx < -140.0 || sx > sw + 140.0 {
                continue;
            }
            let a = (p.life / p.max_life).clamp(0.0, 1.0);
            let e = a * a * (3.0 - 2.0 * a); // ease-out fade
            let rot = p.rot;
            let pick = (rot.abs() * 4.0) as usize;
            match p.kind {
                // kicked dust / silt
                0 => {
                    let s = &fx.sprites.dirt[pick % fx.sprites.dirt.len()];
                    let col = match self.biome {
                        Biome::Aquarius => c(198, 208, 180, (e * 170.0) as u8),
                        Biome::Cosma => c(152, 132, 192, (e * 120.0) as u8),
                        Biome::Home => c(196, 176, 150, (e * 150.0) as u8),
                        _ => c(228, 210, 178, (e * 175.0) as u8),
                    };
                    draw_spr(s, sx, p.y, p.size, rot, col);
                }
                // bubble: soft ring plus a specular catch
                1 => {
                    let s = &fx.sprites.circle[pick % fx.sprites.circle.len()];
                    draw_spr(s, sx, p.y, p.size, 0.0, c(214, 242, 255, (e * 195.0) as u8));
                    draw_circle(sx - p.size * 0.15, p.y - p.size * 0.16, 1.7,
                        c(240, 255, 255, (e * 245.0) as u8));
                }
                // star sparkle
                2 => {
                    let s = fx.sprites.star_at(pick);
                    let tw = (p.life * 9.0).sin().abs() * 0.55 + 0.45;
                    draw_spr(s, sx, p.y, p.size, rot, c(255, 248, 214, (e * tw * 240.0) as u8));
                }
                // ember
                3 => {
                    let s = fx.sprites.spark_at(pick);
                    let tw = (p.life * 13.0).sin().abs() * 0.5 + 0.5;
                    draw_spr(s, sx, p.y, p.size, rot, c(255, 174, 98, (e * tw * 235.0) as u8));
                }
                // sugar swirl: two counter-rotating twirls
                4 => {
                    let s = &fx.sprites.twirl[pick % fx.sprites.twirl.len()];
                    draw_spr(s, sx, p.y, p.size, rot, c(255, 234, 247, (e * 155.0) as u8));
                    draw_spr(s, sx, p.y, p.size * 0.55, rot * 1.7, c(255, 188, 222, (e * 135.0) as u8));
                }
                // void dust
                5 => {
                    let s = fx.sprites.smoke_at(pick);
                    draw_spr(s, sx, p.y, p.size, rot, c(124, 104, 192, (e * 44.0) as u8));
                }
                // rune magic
                6 => {
                    let s = &fx.sprites.magic[pick % fx.sprites.magic.len()];
                    let tw = (p.life * 5.0).sin().abs() * 0.4 + 0.6;
                    draw_spr(s, sx, p.y, p.size, rot, c(255, 170, 228, (e * tw * 200.0) as u8));
                }
                // light fleck
                _ => {
                    let s = fx.sprites.light_at(pick);
                    draw_spr(s, sx, p.y, p.size, rot, c(255, 246, 216, (e * 125.0) as u8));
                }
            }
        }

        // proximity prompt (styled chip)
        if let Some((label, is_actor, _)) = self.nearest_info() {
            let msg = if is_actor {
                format!("click / z: talk to {}", label)
            } else {
                format!("click / z: inspect {}", label)
            };
            let mw = dlg_width(&msg, 24) + 44.0;
            let mx = (px - mw / 2.0).clamp(6.0, sw - mw - 6.0);
            let my = 374.0;
            draw_rectangle(mx, my, mw, 30.0, Color::from_rgba(255, 246, 233, 240));
            draw_rectangle_lines(mx, my, mw, 30.0, 2.0, Color::from_rgba(74, 42, 99, 255));
            draw_dlg_text(&msg, mx + 22.0, my + 22.0, 24, Color::from_rgba(74, 42, 99, 255));
        }

        // hint toast (styled chip)
        if self.hint_t > 0.0 && !self.hint.is_empty() {
            let hw = dlg_width(&self.hint, 26) + 48.0;
            let hx = (sw * 0.5 - hw / 2.0).clamp(6.0, sw - hw - 6.0);
            let hy = 86.0;
            draw_rectangle(hx, hy, hw, 34.0, Color::from_rgba(74, 42, 99, 230));
            draw_rectangle_lines(hx, hy, hw, 34.0, 2.0, Color::from_rgba(255, 214, 230, 190));
            draw_dlg_text(&self.hint, hx + 24.0, hy + 26.0, 26, Color::from_rgba(255, 246, 233, 255));
        }
    }
}


