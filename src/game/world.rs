//! Explorable 2D pixel-art overworld: grounded actors, depth-sorted props,
//! Ansimuz reef fauna, particles, camera smoothing, and crisp prompts.

use macroquad::prelude::*;
use crate::art::{c, draw_tile, fx::Fx, Textures};
use crate::game::script::Biome;
use crate::game::ui::{draw_dlg_shadow, draw_nine_slice, draw_tag_shadow, dlg_width, tag_width, Skin};
use crate::game::Progress;

pub const WALK: f32 = 240.0;
pub const SWIFT: f32 = 380.0;
pub const REACH: f32 = 96.0;

fn draw_spr(tex: &Texture2D, x: f32, y: f32, size: f32, rot: f32, col: Color) {
    draw_texture_ex(tex, x - size * 0.5, y - size * 0.5, col, DrawTextureParams {
        dest_size: Some(Vec2::new(size, size)),
        rotation: rot,
        pivot: Some(Vec2::new(size * 0.5, size * 0.5)),
        ..Default::default()
    });
}

pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub tex: Texture2D,
    pub label: &'static str,
    pub action: &'static str,
    pub patrol: Option<(f32, f32)>,
    pub dir: f32,
    pub speed: f32,
    pub face: f32,
    pub anim: &'static str,
    pub scale: f32,
}

impl Actor {
    pub fn new(x: f32, y: f32, tex: Texture2D, label: &'static str, action: &'static str) -> Self {
        Self {
            x,
            y,
            tex,
            label,
            action,
            patrol: None,
            dir: 1.0,
            speed: 0.0,
            face: -1.0,
            anim: "",
            scale: 3.0,
        }
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
}

pub struct Prop {
    pub id: &'static str,
    pub x: f32,
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
    pub kind: u8,
    pub size: f32,
    pub rot: f32,
    pub spin: f32,
    pub grav: f32,
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
    pub click: Option<f32>,
    amb_acc: [f32; 8],
    pub bot_dir: f32,
    pub bot_y: f32,
    pub drawn_bottom: std::cell::Cell<f32>,
    scale_x: f32,
    scale_y: f32,
    lean: f32,
    was_moving: bool,
    prev_facing: f32,
    dash_t: f32,
    dash_cd: f32,
    afterimages: Vec<(f32, f32, f32, f32, bool)>,
    cam_look: f32,
    prompt_pop: f32,
    prompt_target: Option<String>,
}

impl World {
    pub fn new(biome: Biome, tex: &Textures) -> Self {
        let mut w = World {
            biome,
            length: 3800.0,
            player: (160.0, 562.0),
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
            bot_dir: 0.0,
            bot_y: 0.0,
            drawn_bottom: std::cell::Cell::new(0.0),
            scale_x: 1.0,
            scale_y: 1.0,
            lean: 0.0,
            was_moving: false,
            prev_facing: 1.0,
            dash_t: 0.0,
            dash_cd: 0.0,
            afterimages: Vec::new(),
            cam_look: 0.0,
            prompt_pop: 0.0,
            prompt_target: None,
        };
        w.populate(biome, tex);
        w
    }

    pub fn populate(&mut self, biome: Biome, tex: &Textures) {
        self.biome = biome;
        self.actors.clear();
        self.props.clear();
        self.particles.clear();
        self.afterimages.clear();
        self.click = None;
        self.cam = 0.0;
        self.cam_look = 0.0;
        self.scale_x = 1.0;
        self.scale_y = 1.0;
        self.lean = 0.0;
        self.dash_t = 0.0;
        self.dash_cd = 0.0;
        self.prompt_pop = 0.0;
        self.prompt_target = None;

        match biome {
            Biome::Kelp => {
                self.length = 3800.0;
                self.player = (160.0, 568.0);
                // Actors grounded at feet_y ~ 562..568
                self.actors.push(
                    Actor::new(560.0, 565.0, tex.kenny.clone(), "Kenny", "kenny")
                        .anim("kenny"),
                );
                self.actors.push(
                    Actor::new(2370.0, 563.0, tex.guard_a.clone(), "Guard 1", "guards")
                        .anim("guard_a"),
                );
                self.actors.push(
                    Actor::new(2470.0, 567.0, tex.guard_b.clone(), "Guard 2", "guards")
                        .anim("guard_b"),
                );
                self.actors.push(
                    Actor::new(3300.0, 566.0, tex.sophie.clone(), "Sophie", "queen_crab")
                        .anim("sophie"),
                );
                self.actors.push(
                    Actor::new(3440.0, 563.0, tex.queen_crab.clone(), "Queen Crab", "queen_crab")
                        .anim("queen_crab"),
                );

                // Buildings planted solidly onto the turf surface (y=559 behind actors >=562)
                self.props.push(Prop { id: "kenny_house", x: 780.0, y: 559.0, w: 192.0, h: 168.0, label: "", action: "" });
                self.props.push(Prop { id: "riddle", x: 1060.0, y: 562.0, w: 84.0, h: 114.0, label: "Kelp Monolith I", action: "riddle_0" });
                self.props.push(Prop { id: "riddle", x: 1460.0, y: 562.0, w: 84.0, h: 114.0, label: "Kelp Monolith II", action: "riddle_1" });
                self.props.push(Prop { id: "gate", x: 1890.0, y: 559.0, w: 192.0, h: 210.0, label: "Stone Throne Gate", action: "gate" });
                self.props.push(Prop { id: "riddle", x: 2820.0, y: 562.0, w: 84.0, h: 114.0, label: "Kelp Monolith III", action: "riddle_2" });
                self.props.push(Prop { id: "crab_castle", x: 3560.0, y: 559.0, w: 320.0, h: 280.0, label: "", action: "" });
            }
            Biome::Coral => {
                self.length = 4200.0;
                self.player = (180.0, 568.0);
                self.actors.push(Actor::new(390.0, 563.0, tex.jack.clone(), "Jack", "jack_jill").anim("jack"));
                self.actors.push(Actor::new(480.0, 567.0, tex.jill.clone(), "Jill", "jack_jill").anim("jill"));
                self.actors.push(Actor::new(880.0, 563.0, tex.angelfish.clone(), "AngelFish", "alley").anim("angelfish"));
                self.actors.push(Actor::new(980.0, 567.0, tex.stonefish.clone(), "StoneFish", "alley").anim("stonefish"));

                self.actors.push(
                    Actor::new(1480.0, 563.0, tex.gill.clone(), "Gillbert", "clue_gill_scal")
                        .patrolling(1410.0, 1540.0, 32.0)
                        .anim("gill"),
                );
                self.actors.push(
                    Actor::new(1610.0, 566.0, tex.scal.clone(), "Scalleon", "clue_gill_scal")
                        .anim("scal"),
                );
                self.actors.push(
                    Actor::new(1940.0, 565.0, tex.lady.clone(), "Market Lady", "clue_lady")
                        .patrolling(1870.0, 2020.0, 28.0)
                        .anim("lady"),
                );
                self.actors.push(Actor::new(3350.0, 565.0, tex.towtou.clone(), "Tow-Tou", "towtou").anim("towtou"));
                self.actors.push(Actor::new(3610.0, 562.0, tex.angler_a.clone(), "Angler Trio", "anglers").anim("angler_a"));
                self.actors.push(Actor::new(3700.0, 568.0, tex.angler_b.clone(), "Angler B", "anglers").anim("angler_b"));
                self.actors.push(Actor::new(3790.0, 562.0, tex.angler_c.clone(), "Angler C", "anglers").anim("angler_c"));

                // Houses & City Landmarks planted on the sand plane behind actors (y=559)
                self.props.push(Prop { id: "coral_house_0", x: 300.0, y: 559.0, w: 186.0, h: 168.0, label: "", action: "" });
                self.props.push(Prop { id: "riddle", x: 1180.0, y: 562.0, w: 84.0, h: 114.0, label: "Coral Tablet I", action: "riddle_3" });
                self.props.push(Prop { id: "item_0", x: 1360.0, y: 563.0, w: 72.0, h: 72.0, label: "Blue Seashroom", action: "item_0" });
                self.props.push(Prop { id: "coral_house_1", x: 1660.0, y: 559.0, w: 186.0, h: 168.0, label: "", action: "" });
                self.props.push(Prop { id: "decoy", x: 1750.0, y: 563.0, w: 72.0, h: 72.0, label: "Bright Reef Berry", action: "decoy_berry" });
                self.props.push(Prop { id: "market_stall", x: 1860.0, y: 559.0, w: 150.0, h: 132.0, label: "", action: "" });
                self.props.push(Prop { id: "item_1", x: 2120.0, y: 563.0, w: 72.0, h: 72.0, label: "Tentacle Anemone", action: "item_1" });
                self.props.push(Prop { id: "riddle", x: 2290.0, y: 562.0, w: 84.0, h: 114.0, label: "Coral Tablet II", action: "riddle_4" });
                self.props.push(Prop { id: "neptune_statue", x: 2230.0, y: 559.0, w: 88.0, h: 214.0, label: "", action: "" });
                self.props.push(Prop { id: "ruin_hall", x: 2450.0, y: 559.0, w: 240.0, h: 190.0, label: "", action: "" });
                self.props.push(Prop { id: "neptune", x: 2630.0, y: 559.0, w: 96.0, h: 220.0, label: "Statue of Neptune", action: "clue_neptune" });
                self.props.push(Prop { id: "item_2", x: 2790.0, y: 563.0, w: 76.0, h: 76.0, label: "Glowing Kelp Bush", action: "item_2" });
                self.props.push(Prop { id: "riddle", x: 3020.0, y: 562.0, w: 84.0, h: 114.0, label: "Coral Tablet III", action: "riddle_5" });
                self.props.push(Prop { id: "towtou_garage", x: 3240.0, y: 559.0, w: 204.0, h: 176.0, label: "", action: "" });
                self.props.push(Prop { id: "cave", x: 4000.0, y: 559.0, w: 210.0, h: 205.0, label: "Grave Cave Maze", action: "maze" });
            }
            Biome::Slums => {
                self.length = 3800.0;
                self.player = (180.0, 568.0);
                self.actors.push(Actor::new(1320.0, 566.0, tex.thaumiel.clone(), "Thaumiel", "thaumiel").anim("thaumiel"));
                self.actors.push(Actor::new(1460.0, 563.0, tex.tenebris.clone(), "Tenebris", "tenebris").anim("tenebris"));
                self.actors.push(Actor::new(3520.0, 563.0, tex.genie.clone(), "Magic Fire Genie", "rainbow").anim("genie"));

                self.props.push(Prop { id: "shipwreck", x: 380.0, y: 559.0, w: 220.0, h: 150.0, label: "", action: "" });
                self.props.push(Prop { id: "slum_shack", x: 740.0, y: 559.0, w: 180.0, h: 156.0, label: "", action: "" });
                self.props.push(Prop { id: "whale", x: 1140.0, y: 559.0, w: 260.0, h: 110.0, label: "", action: "" });
                self.props.push(Prop { id: "riddle", x: 1690.0, y: 562.0, w: 84.0, h: 114.0, label: "Abyss Stone I", action: "riddle_6" });
                self.props.push(Prop { id: "riddle", x: 1960.0, y: 562.0, w: 84.0, h: 114.0, label: "Abyss Stone II", action: "riddle_7" });
                self.props.push(Prop { id: "algae_temple", x: 2440.0, y: 559.0, w: 240.0, h: 210.0, label: "Figurate Algae Temple", action: "algae_temple" });
                self.props.push(Prop { id: "torch", x: 2940.0, y: 562.0, w: 66.0, h: 120.0, label: "", action: "" });
                self.props.push(Prop { id: "torch", x: 3140.0, y: 562.0, w: 66.0, h: 120.0, label: "", action: "" });
                self.props.push(Prop { id: "torch", x: 3340.0, y: 562.0, w: 66.0, h: 120.0, label: "", action: "" });
            }
            Biome::Home => {
                self.length = 1140.0;
                self.player = (200.0, 564.0);
                self.actors.push(Actor::new(560.0, 558.0, tex.grandparent.clone(), "Grandparent", "grandparent").anim("grandparent"));
                self.actors.push(Actor::new(720.0, 562.0, tex.mark.clone(), "Mark", "mark").anim("mark"));
                self.actors.push(Actor::new(910.0, 560.0, tex.nadia.clone(), "Nadia", "nadia").anim("nadia"));

                self.props.push(Prop { id: "home_hearth", x: 280.0, y: 540.0, w: 144.0, h: 192.0, label: "", action: "" });
                self.props.push(Prop { id: "home_shelf", x: 460.0, y: 540.0, w: 132.0, h: 168.0, label: "", action: "" });
                self.props.push(Prop { id: "home_table", x: 720.0, y: 546.0, w: 156.0, h: 108.0, label: "", action: "" });
                self.props.push(Prop { id: "home_bed", x: 960.0, y: 554.0, w: 216.0, h: 132.0, label: "", action: "" });
            }
        }
    }

    pub fn say_hint(&mut self, msg: impl Into<String>) {
        self.hint = msg.into();
        self.hint_t = 3.2;
    }

    pub fn burst(&mut self, x: f32, y: f32, n: u32, kind: u8) {
        let now = get_time();
        for i in 0..n {
            let s = (now * 1000.0) as u64 + i as u64 * 17;
            let ang = ((s % 360) as f32).to_radians();
            let spd = 40.0 + ((s / 7) % 90) as f32;
            let (size, spin, grav) = match kind {
                1 => (9.0 + (i % 4) as f32 * 4.0, 0.4, -36.0),
                2 => (16.0 + (i % 3) as f32 * 6.0, 2.6, 0.0),
                3 => (12.0 + (i % 4) as f32 * 4.0, 1.5, -55.0),
                _ => (12.0 + (i % 3) as f32 * 5.0, 1.0, 30.0),
            };
            let max_life = 0.7 + (i % 4) as f32 * 0.25;
            self.particles.push(Particle {
                x,
                y,
                vx: ang.cos() * spd,
                vy: ang.sin() * spd - 35.0,
                life: max_life,
                max_life,
                kind,
                size,
                rot: (i as f32 * 1.1) % std::f32::consts::TAU,
                spin,
                grav,
                drag: 0.96,
            });
        }
    }

    pub fn update(&mut self, dt: f32) -> Option<String> {
        let swift = self.biome == Biome::Kelp && crate::game::swift_swim_unlocked();
        let spd = if swift { SWIFT } else { WALK };
        let mut dx = self.bot_dir;
        let mut dy = self.bot_y;

        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            dx -= 1.0;
            self.click = None;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            dx += 1.0;
            self.click = None;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            dy -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            dy += 1.0;
        }

        if is_mouse_button_pressed(MouseButton::Left) && self.interact_cd <= 0.0 {
            let (mx, my) = mouse_position();
            // Avoid intercepting HUD pills in top strip
            if my > 68.0 {
                let wx = self.cam + mx;
                if let Some(act) = self.click_hit(wx, my) {
                    self.click = None;
                    return Some(act);
                }
                self.click = Some(wx.clamp(40.0, self.length - 40.0));
            }
        }

        if let Some(tx) = self.click {
            let d = tx - self.player.0;
            if d.abs() < 12.0 {
                self.click = None;
            } else {
                dx = d.signum();
            }
        }

        self.moving = dx.abs() > 0.05 || dy.abs() > 0.05;
        if dx.abs() > 0.05 {
            self.facing = dx.signum();
        }
        let swimming = self.biome == Biome::Kelp || self.biome == Biome::Coral;
        if self.dash_cd > 0.0 {
            self.dash_cd = (self.dash_cd - dt).max(0.0);
        }
        if (is_key_pressed(KeyCode::LeftShift) || is_key_pressed(KeyCode::RightShift) || is_key_pressed(KeyCode::X))
            && self.dash_cd <= 0.0
        {
            self.dash_t = 0.16;
            self.dash_cd = 0.55;
            self.scale_x = 1.32;
            self.scale_y = 0.76;
            for k in 0..4 {
                if self.particles.len() < 100 {
                    self.particles.push(Particle {
                        x: self.player.0 - self.facing * (12.0 + k as f32 * 7.0),
                        y: self.player.1 - 16.0 - k as f32 * 5.0,
                        vx: -self.facing * (95.0 + k as f32 * 22.0),
                        vy: -18.0 + (k as f32 - 1.5) * 14.0,
                        life: 0.38,
                        max_life: 0.38,
                        kind: if swimming { 1 } else { 5 },
                        size: 11.0,
                        rot: 0.0,
                        spin: 2.2,
                        grav: if swimming { -12.0 } else { 14.0 },
                        drag: 0.90,
                    });
                }
            }
        }
        let mut eff_dx = dx.clamp(-1.0, 1.0);
        let mut move_spd = spd;
        if self.dash_t > 0.0 {
            self.dash_t = (self.dash_t - dt).max(0.0);
            if eff_dx.abs() < 0.1 {
                eff_dx = self.facing;
            }
            move_spd = spd * 1.95;
            if self.afterimages.len() < 8 {
                self.afterimages.push((self.player.0, self.player.1, self.facing, 0.22, swimming));
            }
        }
        for (_, _, _, life, _) in &mut self.afterimages {
            *life -= dt;
        }
        self.afterimages.retain(|a| a.3 > 0.0);

        self.player.0 = (self.player.0 + eff_dx * move_spd * dt).clamp(40.0, self.length - 40.0);
        let y_min = if self.biome == Biome::Home { 544.0 } else { 561.0 };
        self.player.1 = (self.player.1 + dy.clamp(-1.0, 1.0) * (spd * 0.55) * dt).clamp(y_min, 586.0);

        // Soul Knight style squash/stretch & turn puffs
        if self.moving && !self.was_moving {
            self.scale_x = 1.15;
            self.scale_y = 0.86;
        } else if !self.moving && self.was_moving {
            self.scale_x = 0.88;
            self.scale_y = 1.14;
        } else if dx.abs() > 0.05 && (self.facing - self.prev_facing).abs() > 0.5 {
            self.scale_x = 1.18;
            self.scale_y = 0.84;
            if self.particles.len() < 95 {
                self.particles.push(Particle {
                    x: self.player.0 - self.facing * 8.0,
                    y: self.player.1 - 6.0,
                    vx: self.prev_facing * 48.0,
                    vy: -16.0,
                    life: 0.32,
                    max_life: 0.32,
                    kind: if swimming { 1 } else { 5 },
                    size: 10.0,
                    rot: 0.0,
                    spin: 1.8,
                    grav: 10.0,
                    drag: 0.90,
                });
            }
        }
        self.was_moving = self.moving;
        if dx.abs() > 0.05 {
            self.prev_facing = self.facing;
        }
        let spring = (dt * 14.0).min(1.0);
        self.scale_x += (1.0 - self.scale_x) * spring;
        self.scale_y += (1.0 - self.scale_y) * spring;
        let want_lean = if self.dash_t > 0.0 {
            self.facing * 0.14
        } else if self.moving {
            self.facing * 0.055
        } else {
            0.0
        };
        self.lean += (want_lean - self.lean) * (dt * 15.0).min(1.0);

        if self.moving {
            self.bob_t += dt * 11.0;
            self.step_t += dt;
            let step_iv = if swift { 0.24 } else { 0.34 };
            if self.step_t >= step_iv {
                self.step_t = 0.0;
                self.step_hit = true;
            }
            if swimming && self.particles.len() < 90 {
                self.amb_acc[7] += dt * if swift { 16.0 } else { 9.0 };
                while self.amb_acc[7] >= 1.0 {
                    self.amb_acc[7] -= 1.0;
                    self.particles.push(Particle {
                        x: self.player.0 - self.facing * 26.0,
                        y: self.player.1 - 36.0 + ((get_time() * 43.0).sin() as f32) * 8.0,
                        vx: -self.facing * 24.0,
                        vy: -38.0,
                        life: 1.1,
                        max_life: 1.1,
                        kind: 1,
                        size: 9.0,
                        rot: 0.0,
                        spin: 0.8,
                        grav: -10.0,
                        drag: 0.97,
                    });
                }
            } else if self.step_hit && self.particles.len() < 90 {
                self.particles.push(Particle {
                    x: self.player.0 - self.facing * 10.0,
                    y: self.player.1 - 4.0,
                    vx: -self.facing * 18.0,
                    vy: -12.0,
                    life: 0.45,
                    max_life: 0.45,
                    kind: 5,
                    size: 11.0,
                    rot: 0.0,
                    spin: 1.5,
                    grav: 8.0,
                    drag: 0.92,
                });
            }
        } else {
            self.bob_t = 0.0;
            self.step_t = 0.20;
        }

        for a in &mut self.actors {
            if let Some((min, max)) = a.patrol {
                a.x += a.dir * a.speed * dt;
                a.face = a.dir;
                if a.x <= min {
                    a.x = min;
                    a.dir = 1.0;
                }
                if a.x >= max {
                    a.x = max;
                    a.dir = -1.0;
                }
            }
        }

        self.spawn_ambient(dt);

        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += p.grav * dt;
            let damp = p.drag.powf(dt * 60.0);
            p.vx *= damp;
            p.vy *= damp;
            p.rot += p.spin * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        if self.hint_t > 0.0 {
            self.hint_t = (self.hint_t - dt).max(0.0);
        }
        if self.interact_cd > 0.0 {
            self.interact_cd = (self.interact_cd - dt).max(0.0);
        }

        let want_look = if self.dash_t > 0.0 {
            self.facing * 72.0
        } else if self.moving {
            self.facing * 46.0
        } else {
            self.facing * 14.0
        };
        self.cam_look += (want_look - self.cam_look) * (dt * 4.5).min(1.0);
        let target_cam = (self.player.0 + self.cam_look - screen_width() * 0.44).clamp(0.0, (self.length - screen_width()).max(0.0));
        self.cam += (target_cam - self.cam) * (dt * 8.0).min(1.0);

        let next_lbl = self.nearest_label().map(|x| x.2.to_string());
        if next_lbl != self.prompt_target {
            self.prompt_target = next_lbl;
            self.prompt_pop = 0.0;
        }
        if self.prompt_target.is_some() {
            self.prompt_pop = (self.prompt_pop + dt * 7.5).min(1.0);
        }

        if (is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter)) && self.interact_cd <= 0.0 {
            if let Some(act) = self.nearest_action() {
                return Some(act);
            }
        }
        None
    }

    fn spawn_ambient(&mut self, dt: f32) {
        if self.particles.len() > 80 {
            return;
        }
        let specs: &[(f32, u8)] = match self.biome {
            Biome::Kelp => &[(2.8, 1), (1.4, 2)],
            Biome::Coral => &[(3.8, 1), (1.6, 2)],
            Biome::Slums => &[(3.2, 5), (1.8, 7), (1.0, 6)],
            Biome::Home => &[(1.2, 7)],
        };
        let t = get_time();
        for (i, &(rate, kind)) in specs.iter().enumerate() {
            self.amb_acc[i] += rate * dt;
            while self.amb_acc[i] >= 1.0 {
                self.amb_acc[i] -= 1.0;
                let s = ((t * 973.0) as u64).wrapping_mul(6364136223846793005).wrapping_add(i as u64 * 91);
                let x = self.cam + ((s >> 16) % (screen_width().max(1.0) as u64)) as f32;
                let y = 180.0 + ((s >> 28) % 390) as f32;
                let vx = -8.0 + ((s >> 8) % 16) as f32;
                let vy = match kind {
                    1 => -28.0 - ((s >> 4) % 24) as f32,
                    5 => 6.0 + ((s >> 4) % 12) as f32,
                    _ => -10.0 + ((s >> 4) % 12) as f32,
                };
                self.particles.push(Particle {
                    x,
                    y,
                    vx,
                    vy,
                    life: 2.8,
                    max_life: 2.8,
                    kind,
                    size: 10.0 + ((s >> 12) % 8) as f32,
                    rot: 0.0,
                    spin: 0.5,
                    grav: 0.0,
                    drag: 0.99,
                });
            }
        }
    }

    fn click_hit(&self, wx: f32, wy: f32) -> Option<String> {
        for p in &self.props {
            if !p.action.is_empty()
                && wx >= p.x - p.w * 0.55
                && wx <= p.x + p.w * 0.55
                && wy >= p.y - p.h
                && wy <= p.y + 32.0
                && (self.player.0 - p.x).abs() < REACH * 1.85
            {
                return Some(p.action.to_string());
            }
        }
        for a in &self.actors {
            if !a.action.is_empty()
                && (wx - a.x).abs() < 46.0
                && wy >= a.y - 115.0
                && wy <= a.y + 24.0
                && (self.player.0 - a.x).abs() < REACH * 1.85
            {
                return Some(a.action.to_string());
            }
        }
        None
    }

    pub fn nearest_action(&self) -> Option<String> {
        let mut best: Option<(f32, &str)> = None;
        for a in &self.actors {
            if a.action.is_empty() {
                continue;
            }
            let d = (self.player.0 - a.x).abs();
            if d < REACH && best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, a.action));
            }
        }
        for p in &self.props {
            if p.action.is_empty() {
                continue;
            }
            let d = (self.player.0 - p.x).abs();
            if d < REACH + p.w * 0.32 && best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, p.action));
            }
        }
        best.map(|(_, s)| s.to_string())
    }

    fn nearest_label(&self) -> Option<(f32, f32, &str)> {
        let mut best: Option<(f32, f32, f32, &str)> = None;
        for a in &self.actors {
            if a.label.is_empty() {
                continue;
            }
            let d = (self.player.0 - a.x).abs();
            if d < REACH + 16.0 && best.map_or(true, |(bd, _, _, _)| d < bd) {
                best = Some((d, a.x, a.y - a.tex.height() * a.scale - 14.0, a.label));
            }
        }
        for p in &self.props {
            if p.label.is_empty() {
                continue;
            }
            let d = (self.player.0 - p.x).abs();
            if d < REACH + p.w * 0.35 && best.map_or(true, |(bd, _, _, _)| d < bd) {
                best = Some((d, p.x, p.y - p.h - 14.0, p.label));
            }
        }
        best.map(|(_, x, y, l)| (x, y, l))
    }

    // --- Overworld Render --------------------------------------------------

    pub fn draw(&self, tex: &Textures, fx: &Fx, prog: &Progress) {
        let t = get_time() as f32;
        let ground_y = 556.0;
        let cam = self.cam - fx.shake_x;
        let skin = Skin::for_biome(self.biome);

        crate::art::draw_biome_bg(self.biome, t, cam, fx, tex);
        self.draw_ground(tex, cam, ground_y, t);
        self.draw_midground_decor(tex, cam, ground_y, t);

        // Y-sorted render queue (Props, Actors, River)
        #[derive(Clone, Copy)]
        enum EntityKind { Prop(usize), Actor(usize), Player }
        let mut queue: Vec<(f32, EntityKind)> = Vec::with_capacity(self.props.len() + self.actors.len() + 1);
        for (i, p) in self.props.iter().enumerate() {
            queue.push((p.y, EntityKind::Prop(i)));
        }
        for (i, a) in self.actors.iter().enumerate() {
            queue.push((a.y, EntityKind::Actor(i)));
        }
        queue.push((self.player.1, EntityKind::Player));
        queue.sort_by(|a, b| a.0.total_cmp(&b.0));

        for &(_, kind) in &queue {
            match kind {
                EntityKind::Prop(idx) => {
                    let p = &self.props[idx];
                    let sx = p.x - cam;
                    if sx < -320.0 || sx > screen_width() + 320.0 {
                        continue;
                    }
                    self.draw_prop(tex, fx, prog, p, sx, t);
                }
                EntityKind::Actor(idx) => {
                    let a = &self.actors[idx];
                    let sx = a.x - cam;
                    if sx < -120.0 || sx > screen_width() + 120.0 {
                        continue;
                    }
                    self.draw_actor(tex, a, sx, t);
                }
                EntityKind::Player => {
                    self.draw_player(tex, fx, cam, t);
                }
            }
        }

        self.draw_particles(fx, cam);

        // Interaction badge floating over nearest target with smooth pop-in
        if let Some((lx, ly, lbl)) = self.nearest_label() {
            let sx = lx - cam;
            let e = 1.0 - (1.0 - self.prompt_pop).powi(3);
            let bob = (t * 4.2).sin() * 3.0;
            let txt = format!("[E] {}", lbl);
            let pw = tag_width(&txt, 11) + 28.0;
            let ph = (32.0 * e).max(8.0);
            let py = ly - 32.0 + bob + (32.0 - ph) * 0.5;
            draw_nine_slice(tex, sx - pw * 0.5, py, pw, ph, skin.chip, skin.accent);
            if e > 0.45 {
                draw_tag_shadow(&txt, sx - pw * 0.5 + 14.0, py + 21.0, 11, skin.accent);
            }
        }

        // Bottom toast hint
        if self.hint_t > 0.0 && !self.hint.is_empty() {
            let hw = dlg_width(&self.hint, 24) + 40.0;
            let hx = (screen_width() - hw) * 0.5;
            draw_nine_slice(tex, hx, 620.0, hw, 44.0, skin.fill, skin.accent);
            draw_dlg_shadow(&self.hint, hx + 20.0, 649.0, 24, skin.ink);
        }
    }

    pub fn draw_ground(&self, tex: &Textures, cam: f32, ground_y: f32, t: f32) {
        let sw = screen_width();
        let sh = screen_height();
        let tp = &tex.tiles;

        if self.biome == Biome::Home {
            // Cozy cottage wallpaper walls + wainscot strip + crisp hardwood plank floor
            let step = 48.0;
            let cols = (sw / step) as i32 + 3;
            let rows_wall = ((ground_y - 24.0) / step) as i32 + 1;
            let off_x = cam.rem_euclid(step);
            for c_idx in -1..cols {
                let wx = c_idx as f32 * step - off_x;
                for r in 0..rows_wall {
                    let wy = r as f32 * step;
                    let is_wainscot = wy + step >= ground_y - 28.0;
                    let tile = if is_wainscot { &tp.wood_wall_alt } else { &tp.wood_wall };
                    draw_tile(tile, wx, wy, step);
                }
            }
            // Night sky cottage windows & front door
            let dsx = 120.0 - cam;
            if dsx > -90.0 && dsx < sw + 90.0 {
                draw_texture_ex(&tp.door, dsx - 42.0, ground_y - 118.0, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(84.0, 96.0)),
                    ..Default::default()
                });
            }
            for wx_world in [320.0f32, 680.0] {
                let wsx = wx_world - cam;
                if wsx > -90.0 && wsx < sw + 90.0 {
                    draw_texture_ex(&tp.window, wsx - 42.0, 220.0, WHITE, DrawTextureParams {
                        dest_size: Some(vec2(84.0, 84.0)),
                        ..Default::default()
                    });
                }
            }
            // Hardwood planks below ground_y - 24 with warm baseboard highlight
            let floor_top = ground_y - 24.0;
            draw_rectangle(0.0, floor_top - 4.0, sw, 4.0, c(162, 110, 74, 255));
            draw_rectangle(0.0, floor_top - 1.0, sw, 2.0, c(68, 42, 28, 255));
            let f_rows = ((sh - floor_top) / step) as i32 + 2;
            for c_idx in -1..cols {
                let wx = c_idx as f32 * step - off_x;
                for r in 0..f_rows {
                    draw_tile(&tp.planks, wx, floor_top + r as f32 * step, step);
                }
            }
            // Side-view woven runner strip along the main floor
            for c_idx in 6..18 {
                let wx = c_idx as f32 * step - cam;
                if wx < -step || wx > sw + step {
                    continue;
                }
                draw_texture_ex(&tp.rug, wx, floor_top + 18.0, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(step, 26.0)),
                    ..Default::default()
                });
            }
            return;
        }

        // Outdoor tiled seafloor terrain (`step = 48.0` crisp 3x integer scale for 16px tiles)
        let step = 48.0;
        let first_col = (cam / step).floor() as i32 - 1;
        let cols = (sw / step).ceil() as i32 + 3;
        let rows = ((sh - ground_y) / step).ceil() as i32 + 2;
        for c_idx in first_col..first_col + cols {
            let sx = c_idx as f32 * step - cam;
            for r in 0..rows {
                let ry = ground_y + r as f32 * step;
                let tile = match self.biome {
                    Biome::Kelp => {
                        if c_idx >= 64 {
                            if r == 0 { &tp.stone } else { &tp.stone_alt }
                        } else if r >= 1 {
                            &tp.dirt
                        } else if c_idx.rem_euclid(5) == 0 {
                            &tp.ground_speck
                        } else if c_idx.rem_euclid(2) == 0 {
                            &tp.ground
                        } else {
                            &tp.ground_alt
                        }
                    }
                    Biome::Coral => {
                        if r == 0 { &tp.coral_sand } else { &tp.dirt }
                    }
                    Biome::Slums => {
                        if r == 0 {
                            &tex.cosma.floor[0]
                        } else {
                            let idx = 1 + ((c_idx + r).rem_euclid(2) as usize);
                            &tex.cosma.floor[idx.min(tex.cosma.floor.len() - 1)]
                        }
                    }
                    Biome::Home => &tp.planks,
                };
                draw_tile(tile, sx, ry, step);
            }
            // Neon Rainbow Road shimmer beyond the genie shrine
            if self.biome == Biome::Slums && c_idx > 70 {
                let phase = t * 2.2 + c_idx as f32 * 0.4;
                let col = Color::new(
                    0.5 + 0.5 * phase.sin(),
                    0.5 + 0.5 * (phase + 2.1).sin(),
                    0.5 + 0.5 * (phase + 4.2).sin(),
                    0.35,
                );
                draw_rectangle(sx, ground_y + 4.0, step, 14.0, col);
            }
        }
    }

    fn draw_midground_decor(&self, tex: &Textures, cam: f32, ground_y: f32, t: f32) {
        let sw = screen_width();
        match self.biome {
            Biome::Kelp => {
                // Organic swaying kelp groves (skip landmarks so stalks frame doorways instead of slicing roofs)
                for i in 0..22 {
                    let wx = 110.0 + i as f32 * 165.0 + ((i * 53) % 46) as f32;
                    if (wx - 780.0).abs() < 110.0 || (wx - 1890.0).abs() < 115.0 || (wx - 3560.0).abs() < 180.0 {
                        continue;
                    }
                    let sx = wx - cam;
                    if sx < -90.0 || sx > sw + 90.0 {
                        continue;
                    }
                    let stalk = &tex.props.kelp[i % 3];
                    let kh = 176.0 + ((i * 29) % 52) as f32;
                    let sway = (t * 1.45 + i as f32 * 0.85).sin() * 0.055;
                    draw_texture_ex(stalk, sx - 32.0, ground_y - kh + 5.0, WHITE, DrawTextureParams {
                        dest_size: Some(vec2(64.0, kh)),
                        rotation: sway,
                        pivot: Some(vec2(sx, ground_y + 5.0)),
                        ..Default::default()
                    });
                    if i % 3 == 0 {
                        draw_texture_ex(&tex.aqua.sea_bush, sx + 18.0, ground_y - 68.0 + 4.0, WHITE, DrawTextureParams {
                            dest_size: Some(vec2(64.0, 72.0)),
                            ..Default::default()
                        });
                    }
                }
                // Passing Ansimuz reef fish swimming gently through upper kelp canopy
                let frame = ((t * 5.0) as usize) % 4;
                for i in 0..4 {
                    let bx = (260.0 + i as f32 * 880.0 - t * 34.0).rem_euclid(self.length);
                    let sx = bx - cam;
                    if sx < -80.0 || sx > sw + 80.0 {
                        continue;
                    }
                    let fy = 170.0 + (i % 2) as f32 * 58.0 + (t * 1.4 + i as f32).sin() * 12.0;
                    let spr = if i % 2 == 0 { &tex.aqua.anim_fish } else { &tex.aqua.anim_fish_dart };
                    let fw = spr.width() / 4.0;
                    let fh = spr.height();
                    draw_texture_ex(spr, sx, fy, c(198, 240, 232, 225), DrawTextureParams {
                        source: Some(Rect::new(frame as f32 * fw, 0.0, fw, fh)),
                        dest_size: Some(vec2(fw * 2.0, fh * 2.0)),
                        ..Default::default()
                    });
                }
            }
            Biome::Coral => {
                // Animated Ansimuz spritesheet fish schools swimming gracefully overhead
                let frames = 4;
                let frame = ((t * 5.0) as usize) % frames;
                for i in 0..6 {
                    let bx = (160.0 + i as f32 * 640.0 - t * 48.0).rem_euclid(self.length);
                    let sx = bx - cam;
                    if sx < -80.0 || sx > sw + 80.0 {
                        continue;
                    }
                    let fy = 180.0 + (i % 3) as f32 * 62.0 + (t * 1.5 + i as f32).sin() * 14.0;
                    let spr = match i % 3 {
                        0 => &tex.aqua.anim_fish,
                        1 => &tex.aqua.anim_fish_dart,
                        _ => &tex.aqua.anim_fish_big,
                    };
                    let fw = spr.width() / frames as f32;
                    let fh = spr.height();
                    draw_texture_ex(spr, sx, fy, WHITE, DrawTextureParams {
                        source: Some(Rect::new(frame as f32 * fw, 0.0, fw, fh)),
                        dest_size: Some(vec2(fw * 2.2, fh * 2.2)),
                        ..Default::default()
                    });
                }
                // Red & green reef coral tufts anchored behind the unified side-view sand cap
                for i in 0..16 {
                    let wx = 200.0 + i as f32 * 250.0;
                    let sx = wx - cam;
                    if sx < -80.0 || sx > sw + 80.0 {
                        continue;
                    }
                    let spr = if i % 2 == 0 { &tex.aqua.coral_red } else { &tex.aqua.sea_bush };
                    draw_texture_ex(spr, sx - 28.0, ground_y - 68.0, WHITE, DrawTextureParams {
                        dest_size: Some(vec2(68.0, 72.0)),
                        ..Default::default()
                    });
                }
            }
            Biome::Slums => {
                // Grounded luminous void crystals behind walkway (skip hero buildings/skeleton)
                for i in 0..13 {
                    let wx = 250.0 + i as f32 * 270.0;
                    if (wx - 380.0).abs() < 110.0 || (wx - 740.0).abs() < 100.0 || (wx - 1140.0).abs() < 140.0 || (wx - 2440.0).abs() < 140.0 {
                        continue;
                    }
                    let sx = wx - cam;
                    if sx < -80.0 || sx > sw + 80.0 {
                        continue;
                    }
                    let spr = &tex.cosma.crystals[i % tex.cosma.crystals.len()];
                    draw_texture_ex(spr, sx - 24.0, ground_y - 63.0, WHITE, DrawTextureParams {
                        dest_size: Some(vec2(52.0, 68.0)),
                        ..Default::default()
                    });
                }
            }
            Biome::Home => {}
        }
    }

    fn draw_prop(&self, tex: &Textures, fx: &Fx, prog: &Progress, p: &Prop, sx: f32, t: f32) {
        let top = p.y - p.h;
        match p.id {
            "kenny_house" => draw_texture_ex(&tex.props.kenny_house, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "crab_castle" => draw_texture_ex(&tex.props.crab_castle, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "coral_house_0" => draw_texture_ex(&tex.props.coral_house_0, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "coral_house_1" => draw_texture_ex(&tex.props.coral_house_1, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "market_stall" => draw_texture_ex(&tex.props.market_stall, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "ruin_hall" => draw_texture_ex(&tex.aqua.ruin_hall, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "towtou_garage" => draw_texture_ex(&tex.props.towtou_garage, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "shipwreck" => draw_texture_ex(&tex.props.shipwreck, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "slum_shack" => draw_texture_ex(&tex.props.slum_shack, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "whale" => draw_texture_ex(&tex.props.whale_skeleton, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "algae_temple" => {
                fx.glow_rect(sx, top + p.h * 0.55, 220.0, 220.0, c(104, 244, 220, 40));
                draw_texture_ex(&tex.props.algae_temple, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            "home_bed" => draw_texture_ex(&tex.props.home_bed, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "home_hearth" => {
                let fl = 0.88 + 0.12 * (t * 9.0).sin();
                fx.glow_rect(sx, top + p.h * 0.72, 240.0 * fl, 220.0 * fl, c(255, 162, 74, 48));
                draw_texture_ex(&tex.props.home_hearth, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            "home_shelf" => draw_texture_ex(&tex.props.home_shelf, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "home_table" => draw_texture_ex(&tex.props.home_table, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "riddle" => {
                let done = prog.done.contains(p.action);
                let pulse = if done { 0.35 } else { 0.75 + 0.25 * (t * 3.0).sin() };
                let glow_c = if self.biome == Biome::Kelp {
                    c(110, 246, 196, (pulse * 65.0) as u8)
                } else if self.biome == Biome::Coral {
                    c(106, 220, 255, (pulse * 65.0) as u8)
                } else {
                    c(172, 130, 255, (pulse * 65.0) as u8)
                };
                fx.glow_rect(sx, top + p.h * 0.45, 128.0, 128.0, glow_c);
                let tint = if done { c(170, 185, 200, 255) } else { WHITE };
                draw_texture_ex(&tex.props.riddle_stone, sx - p.w * 0.5, top, tint, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            "gate" => {
                let open = prog.gate_solved;
                fx.glow_rect(sx, top + p.h * 0.5, 220.0, 220.0, if open { c(120, 255, 190, 52) } else { c(120, 170, 255, 42) });
                draw_texture_ex(&tex.props.throne_gate, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            "neptune_statue" => draw_texture_ex(&tex.aqua.neptune_statue, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() }),
            "neptune" => {
                fx.glow_rect(sx, top + p.h * 0.4, 150.0, 190.0, c(110, 225, 255, 36));
                draw_texture_ex(&tex.aqua.neptune_coral, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            "cave" => {
                draw_texture_ex(&tex.aqua.coral_arch, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
                fx.glow_rect(sx, top + p.h * 0.65, 130.0, 130.0, c(24, 16, 48, 120));
            }
            "item_0" => {
                if !prog.items[0] {
                    let hover = (t * 3.0).sin() * 5.0;
                    let cy = top + p.h * 0.5 + hover;
                    fx.glow_rect(sx, cy, 118.0, 118.0, c(90, 210, 255, 76));
                    draw_texture_ex(&tex.props.item_seashroom, sx - p.w * 0.5, top + hover, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
                    for k in 0..2 {
                        let a = t * 2.4 + k as f32 * 3.14;
                        draw_spr(fx.sprites.star_at(k), sx + a.cos() * 24.0, cy + a.sin() * 14.0, 12.0, t * 2.0, c(160, 248, 255, 195));
                    }
                }
            }
            "item_1" => {
                if !prog.items[1] {
                    let hover = (t * 3.0 + 1.2).sin() * 5.0;
                    let cy = top + p.h * 0.5 + hover;
                    fx.glow_rect(sx, cy, 118.0, 118.0, c(255, 130, 220, 76));
                    draw_texture_ex(&tex.props.item_anemone, sx - p.w * 0.5, top + hover, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
                    for k in 0..2 {
                        let a = t * 2.4 + 1.0 + k as f32 * 3.14;
                        draw_spr(fx.sprites.star_at(k + 1), sx + a.cos() * 24.0, cy + a.sin() * 14.0, 12.0, t * 2.0, c(255, 180, 240, 195));
                    }
                }
            }
            "item_2" => {
                if !prog.items[2] {
                    let hover = (t * 3.2 + 2.4).sin() * 5.0;
                    let cy = top + p.h * 0.5 + hover;
                    fx.glow_rect(sx, cy, 122.0, 122.0, c(96, 226, 255, 82));
                    draw_texture_ex(&tex.props.item_seaberry, sx - p.w * 0.5, top + hover, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
                    for k in 0..2 {
                        let a = t * 2.6 + 2.0 + k as f32 * 3.14;
                        draw_spr(fx.sprites.star_at(k + 2), sx + a.cos() * 26.0, cy + a.sin() * 14.0, 13.0, t * 2.2, c(160, 252, 255, 210));
                    }
                }
            }
            "decoy" => {
                draw_texture_ex(&tex.props.item_decoy, sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            "torch" => {
                let idx = if self.torches_lit > 0 { 1 + ((t * 6.0) as usize % 2) } else { 0 };
                if self.torches_lit > 0 {
                    fx.glow_rect(sx, top + 36.0, 190.0, 190.0, c(255, 164, 72, 56));
                }
                draw_texture_ex(&tex.props.torch[idx], sx - p.w * 0.5, top, WHITE, DrawTextureParams { dest_size: Some(vec2(p.w, p.h)), ..Default::default() });
            }
            _ => {}
        }
    }

    fn draw_actor(&self, tex: &Textures, a: &Actor, sx: f32, t: f32) {
        let phase_t = t + a.x * 0.017;
        let walks = tex.walk_of(a.anim);
        let idle = tex.idle_of(a.anim);

        // Smooth 4-frame motion for walkers & lively cast; calm blink+exhale idle for stationary cast
        let spr = if a.patrol.is_some() {
            if let Some(frames) = walks {
                &frames[((phase_t * 7.5) as usize) % 4]
            } else {
                &a.tex
            }
        } else if let Some(frames) = walks {
            match a.anim {
                "genie" => &frames[((phase_t * 8.0) as usize) % 4],
                "kenny" | "queen_crab" => match ((phase_t * 4.8) as usize) % 6 {
                    0 => &a.tex,
                    1 => &frames[0],
                    2 => &frames[1],
                    3 => &frames[2],
                    4 => &frames[3],
                    _ => idle.unwrap_or(&a.tex),
                },
                "angler_a" | "angler_b" | "angler_c" | "thaumiel" | "sophie" => {
                    &frames[((phase_t * 5.2) as usize) % 4]
                }
                "guard_a" | "guard_b" | "towtou" => {
                    &frames[((phase_t * 4.5) as usize) % 4]
                }
                _ => {
                    // Calm breathing + natural eye blink rhythm without lifting feet
                    let cyc = phase_t % 3.6;
                    if cyc > 3.25 || (cyc > 1.5 && cyc < 1.85) {
                        idle.unwrap_or(&a.tex)
                    } else {
                        &a.tex
                    }
                }
            }
        } else {
            &a.tex
        };

        let hover_y = match a.anim {
            "guard_a" | "guard_b" => (phase_t * 2.8).sin() * 4.0,
            "genie" => -6.0 + (phase_t * 3.2).sin() * 5.0,
            _ => 0.0,
        };

        // Crisp integer scaling (avoid 1.015 fractional Y shimmer on static pixel art!)
        let near = a.patrol.is_none() && (self.player.0 - a.x).abs() <= REACH;
        let talk_hop = if near && ((phase_t * 4.2) as usize % 4 == 1) { -2.0 } else { 0.0 };
        let dw = spr.width() * a.scale;
        let dh = spr.height() * a.scale;
        let face = if a.patrol.is_none() && (self.player.0 - a.x).abs() < 230.0 {
            if self.player.0 >= a.x { 1.0 } else { -1.0 }
        } else {
            a.face
        };

        if let Some(sh) = (!a.anim.is_empty()).then(|| tex.shadow_of(a.anim)).flatten() {
            let sw = sh.width() * a.scale;
            let shh = sh.height() * a.scale;
            draw_texture_ex(sh, sx - sw * 0.5, a.y - shh * 0.5, WHITE, DrawTextureParams {
                dest_size: Some(vec2(sw, shh)),
                ..Default::default()
            });
        }
        draw_texture_ex(spr, sx - dw * 0.5, a.y - dh + hover_y + talk_hop, WHITE, DrawTextureParams {
            dest_size: Some(vec2(dw, dh)),
            flip_x: face < 0.0,
            ..Default::default()
        });
    }

    fn draw_player(&self, tex: &Textures, _fx: &Fx, cam: f32, t: f32) {
        let sx = self.player.0 - cam;
        let feet_y = self.player.1;
        let swimming = self.biome == Biome::Kelp || self.biome == Biome::Coral;
        let dashing = self.dash_t > 0.0;
        let spr = if swimming && dashing {
            let idx = (self.bob_t * 0.72) as usize % 4;
            &tex.river_swim[idx]
        } else if self.moving {
            let idx = (self.bob_t * 0.72) as usize % 4;
            &tex.river_walks[idx]
        } else {
            let cyc = t % 3.6;
            if cyc > 3.25 || (cyc > 1.5 && cyc < 1.85) {
                tex.idle_of("river").unwrap_or(&tex.river)
            } else {
                &tex.river
            }
        };

        // Dash cyan afterimages use the matching swim/stride pose
        for &(ax, ay, aface, life, aswim) in &self.afterimages {
            let k = (life / 0.22).clamp(0.0, 1.0);
            let aspr = if aswim { &tex.river_swim[((life * 14.0) as usize) % 4] } else { spr };
            let aw = aspr.width() * 3.0 * self.scale_x;
            let ah = aspr.height() * 3.0 * self.scale_y;
            draw_texture_ex(
                aspr,
                ax - cam - aw * 0.5,
                ay - ah,
                c(110, 232, 255, (k * 135.0) as u8),
                DrawTextureParams {
                    dest_size: Some(vec2(aw, ah)),
                    flip_x: aface < 0.0,
                    ..Default::default()
                },
            );
        }

        let dw = spr.width() * 3.0 * self.scale_x;
        let dh = spr.height() * 3.0 * self.scale_y;
        let float_off = if swimming && dashing {
            -12.0 + (t * 5.5).sin() * 4.5
        } else {
            0.0
        };

        if let Some(sh) = tex.shadow_of("river") {
            let sh_scale = (1.0 - float_off.abs() * 0.012).clamp(0.65, 1.15);
            let sw = sh.width() * 3.0 * sh_scale;
            let shh = sh.height() * 3.0 * sh_scale;
            draw_texture_ex(sh, sx - sw * 0.5, feet_y - shh * 0.5, WHITE, DrawTextureParams {
                dest_size: Some(vec2(sw, shh)),
                ..Default::default()
            });
        }

        let top_y = feet_y - dh + float_off;
        self.drawn_bottom.set(feet_y);
        draw_texture_ex(spr, sx - dw * 0.5, top_y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(dw, dh)),
            flip_x: self.facing < 0.0,
            rotation: self.lean,
            pivot: Some(vec2(sx, top_y + dh * 0.5)),
            ..Default::default()
        });
    }

    fn draw_particles(&self, fx: &Fx, cam: f32) {
        for p in &self.particles {
            let k = (p.life / p.max_life).clamp(0.0, 1.0);
            let sx = p.x - cam;
            let (spr, col) = match p.kind {
                1 => (&fx.sprites.circle[(p.size as usize) % 3], c(200, 245, 255, (k * 170.0) as u8)),
                2 => (fx.sprites.star_at(p.size as usize), c(255, 235, 130, (k * 230.0) as u8)),
                3 => (&fx.sprites.fire[(p.size as usize) % 2], c(255, 160, 80, (k * 220.0) as u8)),
                5 => (&fx.sprites.smoke[(p.size as usize) % 3], c(160, 145, 215, (k * 135.0) as u8)),
                6 => (&fx.sprites.magic[(p.size as usize) % 3], c(120, 240, 230, (k * 195.0) as u8)),
                _ => (fx.sprites.light_at(p.size as usize), c(255, 240, 200, (k * 140.0) as u8)),
            };
            draw_spr(spr, sx, p.y, p.size, p.rot, col);
        }
    }
}
