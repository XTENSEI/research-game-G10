//! Pixel-art asset loader and layered parallax biome backdrops.
//!
//! All pixel-art cast sprites, portraits, intro plates, UI borders, props,
//! tiles, and Ansimuz underwater crops use `FilterMode::Nearest` for crisp
//! integer-grid pixel edges, while soft nebula/cloud effects keep linear filtering.

pub mod fx;

use std::collections::HashMap;
use macroquad::prelude::*;
use crate::game::script::Biome;

pub struct TilePack {
    pub ground: Texture2D,
    pub ground_alt: Texture2D,
    pub ground_speck: Texture2D,
    pub dirt: Texture2D,
    pub coral_sand: Texture2D,
    pub stone: Texture2D,
    pub stone_alt: Texture2D,
    pub planks: Texture2D,
    pub wood_wall: Texture2D,
    pub wood_wall_alt: Texture2D,
    pub window: Texture2D,
    pub rug: Texture2D,
    pub door: Texture2D,
}

pub struct UiPack {
    pub panel_fill: Texture2D,
    pub panel_border: Texture2D,
    pub divider: Texture2D,
    pub vial_empty: Texture2D,
    pub vial_green: Texture2D,
    pub vial_red: Texture2D,
    pub vial_blue: Texture2D,
    pub heart_full: Texture2D,
    pub heart_empty: Texture2D,
    pub gem: Texture2D,
    pub star: Texture2D,
    pub dying_star: Texture2D,
}

pub struct PropPack {
    pub algae_temple: Texture2D,
    pub coral_house_0: Texture2D,
    pub coral_house_1: Texture2D,
    pub crab_castle: Texture2D,
    pub home_bed: Texture2D,
    pub home_hearth: Texture2D,
    pub home_shelf: Texture2D,
    pub home_table: Texture2D,
    pub kelp: [Texture2D; 3],
    pub kenny_house: Texture2D,
    pub market_stall: Texture2D,
    pub portal_ring: Texture2D,
    pub riddle_stone: Texture2D,
    pub shipwreck: Texture2D,
    pub slum_shack: Texture2D,
    pub throne_gate: Texture2D,
    pub torch: [Texture2D; 3],
    pub towtou_garage: Texture2D,
    pub whale_skeleton: Texture2D,
}

pub struct Aqua {
    pub ansimuz_bg: Texture2D,
    pub ansimuz_mid: Texture2D,
    pub seabed_ledge: Texture2D,
    pub neptune_statue: Texture2D,
    pub neptune_coral: Texture2D,
    pub coral_arch: Texture2D,
    pub ruin_hall: Texture2D,
    pub sea_bush: Texture2D,
    pub coral_red: Texture2D,
    pub sea_kelp: Texture2D,
    pub anim_fish: Texture2D,
    pub anim_fish_big: Texture2D,
    pub anim_fish_dart: Texture2D,
}

pub struct Cosma {
    pub crystals: Vec<Texture2D>,
    pub floor: Vec<Texture2D>,
    pub planets: Vec<Texture2D>,
}

pub struct Textures {
    pub river: Texture2D,
    pub river_swim: [Texture2D; 2],
    pub nadia: Texture2D,
    pub grandparent: Texture2D,
    pub mark: Texture2D,
    pub kenny: Texture2D,
    pub guard_a: Texture2D,
    pub guard_b: Texture2D,
    pub queen_crab: Texture2D,
    pub sophie: Texture2D,
    pub jack: Texture2D,
    pub jill: Texture2D,
    pub angelfish: Texture2D,
    pub stonefish: Texture2D,
    pub gill: Texture2D,
    pub scal: Texture2D,
    pub lady: Texture2D,
    pub towtou: Texture2D,
    pub angler_a: Texture2D,
    pub angler_b: Texture2D,
    pub angler_c: Texture2D,
    pub thaumiel: Texture2D,
    pub tenebris: Texture2D,
    pub genie: Texture2D,
    pub intro: [Texture2D; 4],
    pub portraits: HashMap<String, Texture2D>,
    pub walks: HashMap<&'static str, [Texture2D; 2]>,
    pub shadows: HashMap<&'static str, Texture2D>,
    pub tiles: TilePack,
    pub ui: UiPack,
    pub props: PropPack,
    pub aqua: Aqua,
    pub cosma: Cosma,
}

const CAST_IDS: &[&str] = &[
    "river", "nadia", "grandparent", "mark",
    "kenny", "guard_a", "guard_b", "queen_crab", "sophie",
    "jack", "jill", "angelfish", "stonefish", "gill", "scal", "lady", "towtou",
    "angler_a", "angler_b", "angler_c", "thaumiel", "tenebris", "genie",
];

const PORTRAIT_KEYS: &[&str] = &[
    "river_normal", "river_shocked",
    "nadia_sick", "nadia_cured",
    "grandparent_normal", "mark_normal",
    "kenny_normal", "kenny_happy", "kenny_worried",
    "guard_a_normal", "guard_b_normal",
    "queen_crab_normal", "queen_crab_fuming",
    "sophie_normal",
    "jack_normal", "jill_normal", "jill_shocked",
    "angelfish_normal", "stonefish_normal",
    "gill_normal", "scal_normal", "lady_normal", "towtou_normal",
    "angler_normal", "thaumiel_normal", "tenebris_normal", "genie_normal",
];

const COSMA_CRYSTALS: [&str; 3] = ["crystal_teal", "crystal_pink", "crystal_violet"];
const COSMA_FLOOR: [&str; 3] = ["floor_a", "floor_b", "floor_c"];
const COSMA_VEINS: [&str; 2] = ["vein_pink", "vein_cyan"];
const COSMA_PLANETS: [&str; 7] = [
    "planet00", "planet01", "planet02", "planet03", "planet07", "planet08", "planet09",
];

async fn load_crisp(path: &str) -> Texture2D {
    let t = macroquad::texture::load_texture(path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e));
    t.set_filter(FilterMode::Nearest);
    t
}

async fn load_linear(path: &str) -> Texture2D {
    let t = macroquad::texture::load_texture(path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e));
    t.set_filter(FilterMode::Linear);
    t
}

async fn char_tex(name: &str) -> Texture2D {
    load_crisp(&format!("assets/game/char/{}.png", name)).await
}

async fn tile_tex(name: &str) -> Texture2D {
    load_crisp(&format!("assets/game/tiles/{}.png", name)).await
}

async fn ui_tex(name: &str) -> Texture2D {
    load_crisp(&format!("assets/game/ui/{}.png", name)).await
}

async fn prop_tex(name: &str) -> Texture2D {
    load_crisp(&format!("assets/game/props/{}.png", name)).await
}

async fn aqua_crisp(name: &str) -> Texture2D {
    load_crisp(&format!("assets/game/aqua/{}.png", name)).await
}

async fn cosma_tex(name: &str) -> Texture2D {
    load_linear(&format!("assets/game/cosma/{}.png", name)).await
}

async fn load_tiles() -> TilePack {
    let names = [
        "ground", "ground_alt", "ground_speck", "dirt", "coral_sand",
        "stone", "stone_alt", "planks", "wood_wall", "wood_wall_alt",
        "window", "rug", "door",
    ];
    let mut it = crate::game::join_all(names.iter().map(|n| tile_tex(n)).collect()).await.into_iter();
    TilePack {
        ground: it.next().unwrap(),
        ground_alt: it.next().unwrap(),
        ground_speck: it.next().unwrap(),
        dirt: it.next().unwrap(),
        coral_sand: it.next().unwrap(),
        stone: it.next().unwrap(),
        stone_alt: it.next().unwrap(),
        planks: it.next().unwrap(),
        wood_wall: it.next().unwrap(),
        wood_wall_alt: it.next().unwrap(),
        window: it.next().unwrap(),
        rug: it.next().unwrap(),
        door: it.next().unwrap(),
    }
}

async fn load_ui() -> UiPack {
    let names = [
        "panel_fill", "panel_border", "divider", "vial_empty",
        "vial_green", "vial_red", "vial_blue", "heart_full",
        "heart_empty", "gem", "star", "dying_star",
    ];
    let mut it = crate::game::join_all(names.iter().map(|n| ui_tex(n)).collect()).await.into_iter();
    UiPack {
        panel_fill: it.next().unwrap(),
        panel_border: it.next().unwrap(),
        divider: it.next().unwrap(),
        vial_empty: it.next().unwrap(),
        vial_green: it.next().unwrap(),
        vial_red: it.next().unwrap(),
        vial_blue: it.next().unwrap(),
        heart_full: it.next().unwrap(),
        heart_empty: it.next().unwrap(),
        gem: it.next().unwrap(),
        star: it.next().unwrap(),
        dying_star: it.next().unwrap(),
    }
}

async fn load_props() -> PropPack {
    let names = [
        "algae_temple", "coral_house_0", "coral_house_1", "crab_castle",
        "home_bed", "home_hearth", "home_shelf", "home_table",
        "kelp_0", "kelp_1", "kelp_2", "kenny_house",
        "market_stall", "portal_ring", "riddle_stone", "shipwreck",
        "slum_shack", "throne_gate", "torch_0", "torch_1",
        "torch_2", "towtou_garage", "whale_skeleton",
    ];
    let mut it = crate::game::join_all(names.iter().map(|n| prop_tex(n)).collect()).await.into_iter();
    PropPack {
        algae_temple: it.next().unwrap(),
        coral_house_0: it.next().unwrap(),
        coral_house_1: it.next().unwrap(),
        crab_castle: it.next().unwrap(),
        home_bed: it.next().unwrap(),
        home_hearth: it.next().unwrap(),
        home_shelf: it.next().unwrap(),
        home_table: it.next().unwrap(),
        kelp: [it.next().unwrap(), it.next().unwrap(), it.next().unwrap()],
        kenny_house: it.next().unwrap(),
        market_stall: it.next().unwrap(),
        portal_ring: it.next().unwrap(),
        riddle_stone: it.next().unwrap(),
        shipwreck: it.next().unwrap(),
        slum_shack: it.next().unwrap(),
        throne_gate: it.next().unwrap(),
        torch: [it.next().unwrap(), it.next().unwrap(), it.next().unwrap()],
        towtou_garage: it.next().unwrap(),
        whale_skeleton: it.next().unwrap(),
    }
}

async fn load_aqua() -> Aqua {
    let names = [
        "ansimuz_bg", "ansimuz_mid", "seabed_ledge", "neptune_statue",
        "neptune_coral", "coral_arch", "ruin_hall", "sea_bush",
        "coral_red", "sea_kelp", "anim_fish", "anim_fish_big",
        "anim_fish_dart",
    ];
    let mut it = crate::game::join_all(names.iter().map(|n| aqua_crisp(n)).collect()).await.into_iter();
    Aqua {
        ansimuz_bg: it.next().unwrap(),
        ansimuz_mid: it.next().unwrap(),
        seabed_ledge: it.next().unwrap(),
        neptune_statue: it.next().unwrap(),
        neptune_coral: it.next().unwrap(),
        coral_arch: it.next().unwrap(),
        ruin_hall: it.next().unwrap(),
        sea_bush: it.next().unwrap(),
        coral_red: it.next().unwrap(),
        sea_kelp: it.next().unwrap(),
        anim_fish: it.next().unwrap(),
        anim_fish_big: it.next().unwrap(),
        anim_fish_dart: it.next().unwrap(),
    }
}

async fn load_cosma() -> Cosma {
    let mut floor = Vec::with_capacity(COSMA_FLOOR.len() + COSMA_VEINS.len());
    let fnames: Vec<_> = COSMA_FLOOR.iter().chain(COSMA_VEINS.iter()).map(|n| format!("assets/game/cosma/{}.png", n)).collect();
    for t in crate::game::join_all(fnames.iter().map(|p| load_crisp(p)).collect()).await {
        floor.push(t);
    }
    let crystals = crate::game::join_all(COSMA_CRYSTALS.iter().map(|n| cosma_tex(n)).collect()).await;
    let planets = crate::game::join_all(COSMA_PLANETS.iter().map(|n| cosma_tex(n)).collect()).await;
    Cosma { crystals, floor, planets }
}

impl Textures {
    pub async fn load() -> Self {
        let mut walks = HashMap::new();
        let mut shadows = HashMap::new();
        let mut base_sprites = HashMap::new();

        let mut char_names = Vec::with_capacity(CAST_IDS.len() * 4);
        for &id in CAST_IDS {
            char_names.push(id.to_string());
            char_names.push(format!("{}_w1", id));
            char_names.push(format!("{}_w2", id));
            char_names.push(format!("{}_shadow", id));
        }
        let mut cit = crate::game::join_all(char_names.iter().map(|n| char_tex(n)).collect()).await.into_iter();
        for &id in CAST_IDS {
            base_sprites.insert(id, cit.next().unwrap());
            walks.insert(id, [cit.next().unwrap(), cit.next().unwrap()]);
            shadows.insert(id, cit.next().unwrap());
        }

        let mut portraits = HashMap::new();
        let ppaths: Vec<_> = PORTRAIT_KEYS
            .iter()
            .map(|pk| format!("assets/game/portraits/{}.png", pk))
            .collect();
        let ptex = crate::game::join_all(ppaths.iter().map(|p| load_crisp(p)).collect()).await;
        for (&pk, t) in PORTRAIT_KEYS.iter().zip(ptex.into_iter()) {
            portraits.insert(pk.to_string(), t);
        }

        let swim_tex = crate::game::join_all(vec![char_tex("river_swim1"), char_tex("river_swim2")]).await;
        let mut sw = swim_tex.into_iter();
        let intro_tex = crate::game::join_all(vec![
            load_crisp("assets/game/intro/plate_0.png"),
            load_crisp("assets/game/intro/plate_1.png"),
            load_crisp("assets/game/intro/plate_2.png"),
            load_crisp("assets/game/intro/plate_3.png"),
        ]).await;
        let mut ii = intro_tex.into_iter();
        let intro = [ii.next().unwrap(), ii.next().unwrap(), ii.next().unwrap(), ii.next().unwrap()];

        Self {
            river: base_sprites.remove("river").unwrap(),
            river_swim: [sw.next().unwrap(), sw.next().unwrap()],
            nadia: base_sprites.remove("nadia").unwrap(),
            grandparent: base_sprites.remove("grandparent").unwrap(),
            mark: base_sprites.remove("mark").unwrap(),
            kenny: base_sprites.remove("kenny").unwrap(),
            guard_a: base_sprites.remove("guard_a").unwrap(),
            guard_b: base_sprites.remove("guard_b").unwrap(),
            queen_crab: base_sprites.remove("queen_crab").unwrap(),
            sophie: base_sprites.remove("sophie").unwrap(),
            jack: base_sprites.remove("jack").unwrap(),
            jill: base_sprites.remove("jill").unwrap(),
            angelfish: base_sprites.remove("angelfish").unwrap(),
            stonefish: base_sprites.remove("stonefish").unwrap(),
            gill: base_sprites.remove("gill").unwrap(),
            scal: base_sprites.remove("scal").unwrap(),
            lady: base_sprites.remove("lady").unwrap(),
            towtou: base_sprites.remove("towtou").unwrap(),
            angler_a: base_sprites.remove("angler_a").unwrap(),
            angler_b: base_sprites.remove("angler_b").unwrap(),
            angler_c: base_sprites.remove("angler_c").unwrap(),
            thaumiel: base_sprites.remove("thaumiel").unwrap(),
            tenebris: base_sprites.remove("tenebris").unwrap(),
            genie: base_sprites.remove("genie").unwrap(),
            intro,
            portraits,
            walks,
            shadows,
            tiles: load_tiles().await,
            ui: load_ui().await,
            props: load_props().await,
            aqua: load_aqua().await,
            cosma: load_cosma().await,
        }
    }

    pub fn shadow_of(&self, id: &str) -> Option<&Texture2D> {
        self.shadows.get(id)
    }

    pub fn walk_of(&self, id: &str) -> Option<&[Texture2D; 2]> {
        self.walks.get(id)
    }

    pub fn portrait_expr(&self, speaker: &str, expr: &str) -> Option<&Texture2D> {
        let base = match speaker {
            "River" => "river",
            "Nadia" => "nadia",
            "Grandparent" => "grandparent",
            "Mark" => "mark",
            "Kenny" => "kenny",
            "Guard 1" => "guard_a",
            "Guard 2" => "guard_b",
            "Queen Crab" => "queen_crab",
            "Sophie" => "sophie",
            "Jack" => "jack",
            "Jill" => "jill",
            "AngelFish" => "angelfish",
            "StoneFish" => "stonefish",
            "Gillbert" => "gill",
            "Scalleon" => "scal",
            "Market Lady" => "lady",
            "Tow-Tou" => "towtou",
            "Angler A" | "Angler B" | "Angler C" => "angler",
            "Thaumiel" => "thaumiel",
            "Tenebris" => "tenebris",
            "Magic Fire Genie" => "genie",
            _ => return None,
        };
        let key = format!("{}_{}", base, expr);
        if let Some(t) = self.portraits.get(&key) {
            return Some(t);
        }
        let fallback = if base == "nadia" {
            "nadia_sick".to_string()
        } else {
            format!("{}_normal", base)
        };
        self.portraits.get(&fallback)
    }
}

pub fn draw_tile(tex: &Texture2D, x: f32, y: f32, size: f32) {
    draw_texture_ex(tex, x, y, WHITE, DrawTextureParams {
        dest_size: Some(Vec2::new(size, size)),
        ..Default::default()
    });
}

pub fn c(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn layer_x(k: i32, cam: f32, factor: f32, drift: f32, span: f32) -> f32 {
    let base = (cam * factor + drift).rem_euclid(span);
    k as f32 * span - base
}

fn sparkle_stars(fx: &crate::art::fx::Fx, t: f32, y0: f32, y1: f32, n: u32, hue: Color, alpha: f32, base_seed: u64) {
    let mut seed = base_seed;
    for idx in 0..n {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let x = ((seed >> 33) % 1024) as f32 + 1.0;
        let span = (y1 - y0).max(1.0);
        let y = y0 + ((seed >> 20) % 360) as f32 % span;
        let ph = (seed >> 5) % 16;
        let tw = (0.20 + 0.80 * (t * 1.7 + ph as f32).sin().abs()) * alpha;
        if tw < 0.02 {
            continue;
        }
        let s = 8.0 + ((seed >> 11) % 3) as f32 * 6.0;
        let col = Color::new(hue.r, hue.g, hue.b, tw);
        let spr = fx.sprites.star_at(idx as usize);
        draw_texture_ex(spr, x - s / 2.0, y - s / 2.0, col, DrawTextureParams {
            dest_size: Some(Vec2::new(s, s)),
            rotation: (seed % 40) as f32 * 0.13,
            ..Default::default()
        });
    }
}

fn draw_bubbles(fx: &crate::art::fx::Fx, t: f32, max_y: f32, count: usize, tint: Color) {
    for i in 0..count {
        let seed = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(3);
        let bx = ((seed >> 33) % 1024) as f32;
        let cyc = (t * 0.24 + i as f32 * (1.0 / count as f32)) % 1.0;
        let y = max_y - cyc * (max_y + 120.0);
        let sway = (t * 0.85 + i as f32 * 1.3).sin() * 22.0;
        let r = 8.0 + ((seed >> 20) % 20) as f32 + (1.0 - cyc) * 6.0;
        let a = (0.16 + 0.28 * (1.0 - cyc)) * tint.a;
        let spr = &fx.sprites.circle[i % 3];
        draw_texture_ex(spr, bx + sway - r, y - r, Color::new(tint.r, tint.g, tint.b, a),
            DrawTextureParams { dest_size: Some(Vec2::new(r * 2.0, r * 2.0)), ..Default::default() });
        draw_circle(bx + sway - r * 0.30, y - r * 0.30, 1.6,
            Color::new(0.95, 1.0, 1.0, (a * 1.4).min(1.0)));
    }
}

/// Layered biome backdrop with GPU gradient + Ansimuz pixel reefs + god rays.
pub fn draw_biome_bg(biome: Biome, t: f32, cam: f32, fx: &crate::art::fx::Fx, tex: &Textures) {
    match biome {
        Biome::Kelp => kelp_sky(fx, tex, t, cam),
        Biome::Coral => coral_sky(fx, tex, t, cam),
        Biome::Slums => slums_sky(fx, tex, t, cam),
        Biome::Home => home_sky(fx, t, cam),
    }
}

fn kelp_sky(fx: &crate::art::fx::Fx, tex: &Textures, t: f32, cam: f32) {
    let w = screen_width();
    fx.sky_draw(Biome::Kelp, 0.0, 0.0, w, 560.0);

    // Distant Ansimuz ocean silhouettes tiled smoothly across horizon
    let bg = &tex.aqua.ansimuz_bg;
    let bgh = 360.0;
    let bgw = bg.width() * (bgh / bg.height().max(1.0));
    let n = (w / bgw).ceil() as i32 + 2;
    for k in -1..n {
        let x = layer_x(k, cam, 0.08, 0.0, bgw);
        draw_texture_ex(bg, x, 560.0 - bgh, c(120, 232, 200, 115), DrawTextureParams {
            dest_size: Some(vec2(bgw + 1.0, bgh)),
            ..Default::default()
        });
    }

    // Underwater sunlight caustics + emerald god rays
    for i in 0..6 {
        let x = layer_x(i, cam, 0.04, 0.0, 780.0) + 40.0;
        let lean = (t * 0.25 + i as f32 * 1.7).sin() * 0.12;
        let flick = 0.5 + 0.5 * (t * 0.55 + i as f32 * 2.1).sin();
        fx.beam_strip(
            x, -40.0, 130.0 + i as f32 * 22.0, 600.0,
            c(166, 255, 224, ((0.05 + flick * 0.065) * 255.0) as u8), lean,
        );
    }

    // Mid-depth swaying dark teal kelp silhouettes
    for k in 0..14 {
        let x = layer_x(k, cam, 0.16, 0.0, 1180.0);
        let h = 150.0 + ((k * 73) % 130) as f32;
        let sway = (t * 1.1 + k as f32 * 0.9).sin() * 10.0;
        let kx = x + sway * 0.35;
        let prop = &tex.aqua.sea_kelp;
        draw_texture_ex(prop, kx, 560.0 - h, c(36, 138, 120, 145), DrawTextureParams {
            dest_size: Some(vec2(72.0, h)),
            rotation: sway * 0.008,
            pivot: Some(vec2(kx + 36.0, 560.0)),
            ..Default::default()
        });
    }

    draw_bubbles(fx, t, 550.0, 14, c(195, 255, 235, 255));
}

fn coral_sky(fx: &crate::art::fx::Fx, tex: &Textures, t: f32, cam: f32) {
    let w = screen_width();
    fx.sky_draw(Biome::Coral, 0.0, 0.0, w, 560.0);

    // Ansimuz layered background & midground reef formations
    let bg = &tex.aqua.ansimuz_bg;
    let bgh = 360.0;
    let bgw = bg.width() * (bgh / bg.height().max(1.0));
    let n = (w / bgw).ceil() as i32 + 2;
    for k in -1..n {
        let x = layer_x(k, cam, 0.05, 0.0, bgw);
        draw_texture_ex(bg, x, 560.0 - bgh, c(200, 235, 255, 165), DrawTextureParams {
            dest_size: Some(vec2(bgw + 1.0, bgh)),
            ..Default::default()
        });
    }

    // Sunlit shimmering water ceiling
    for i in 0..4 {
        let yy = -4.0 + i as f32 * 9.0;
        let steps = (w / 12.0) as i32 + 1;
        for k in 0..steps {
            let px = k as f32 * 12.0;
            let wv = ((px + cam * 0.35) * 0.015 + t * 0.9 + i as f32 * 1.7).sin();
            let a = (24.0 + 28.0 * wv).max(0.0) as u8;
            if a > 0 {
                draw_rectangle(px, yy + wv * 5.0, 12.0, 3.0, c(210, 248, 255, a));
            }
        }
    }

    // God rays
    for i in 0..6 {
        let x = layer_x(i, cam, 0.035, 0.0, 780.0) + 40.0;
        let lean = (t * 0.22 + i as f32 * 1.7).sin() * 0.11;
        let flick = 0.5 + 0.5 * (t * 0.5 + i as f32 * 2.3).sin();
        fx.beam_strip(
            x, -40.0, 140.0 + i as f32 * 24.0, 600.0,
            c(196, 238, 255, ((0.048 + flick * 0.070) * 255.0) as u8), lean,
        );
    }

    // Ansimuz midground coral silhouettes
    let mid = &tex.aqua.ansimuz_mid;
    let mh = 220.0;
    let mw = mid.width() * (mh / mid.height().max(1.0));
    let mn = (w / mw).ceil() as i32 + 2;
    for k in -1..mn {
        let x = layer_x(k, cam, 0.14, 0.0, mw);
        draw_texture_ex(mid, x, 560.0 - mh, c(175, 224, 248, 210), DrawTextureParams {
            dest_size: Some(vec2(mw + 1.0, mh)),
            ..Default::default()
        });
    }

    draw_bubbles(fx, t, 550.0, 16, c(214, 242, 255, 255));
}

fn slums_sky(fx: &crate::art::fx::Fx, tex: &Textures, t: f32, cam: f32) {
    let w = screen_width();
    let h = screen_height();
    fx.sky_draw(Biome::Slums, 0.0, 0.0, w, h);

    // Abyssal nebula clouds + faint aurora
    let fams: [(Color, Color); 3] = [
        (c(86, 56, 160, 28), c(138, 78, 200, 18)),
        (c(142, 46, 118, 22), c(186, 82, 162, 15)),
        (c(34, 98, 148, 20), c(68, 156, 192, 12)),
    ];
    for (fi, (body, shade)) in fams.iter().enumerate() {
        let span = 1180.0 + fi as f32 * 300.0;
        let n = (w / span).ceil() as i32 + 2;
        for k in -1..n + 1 {
            let x = layer_x(k, cam, 0.02, t * 2.0 + fi as f32 * 300.0, span);
            if x < -720.0 || x > w + 720.0 {
                continue;
            }
            let seed = ((k as i64 + fi as i64 * 31).wrapping_mul(2654435761)) as u64;
            let y = 20.0 + ((seed >> 12) % 360) as f32;
            let size = 500.0 + ((seed >> 4) % 260) as f32;
            fx.cloud(x, y, size, *body, *shade);
        }
    }

    for i in 0..4 {
        let x = layer_x(i, cam, 0.012, t * 5.0, 1100.0);
        let y = 112.0 + i as f32 * 48.0 + (t * 0.4 + i as f32).sin() * 22.0;
        let lean = -0.16 + (t * 0.15 + i as f32).sin() * 0.10;
        let a = 0.042 + 0.032 * (t * 0.5 + i as f32 * 1.4).sin().abs();
        fx.beam_strip(x, y, 780.0, 82.0, c(90, 236, 224, (a * 255.0) as u8), lean);
    }

    draw_cosma_stars(fx, t);

    // Distant fractured planets above the Somber Trench sky
    for i in 0..2 {
        let p = &tex.cosma.planets[i % tex.cosma.planets.len()];
        let px = layer_x(i as i32, cam, 0.03, 180.0, 1100.0);
        draw_texture_ex(p, px, 72.0 + i as f32 * 56.0, c(160, 145, 220, 130), DrawTextureParams {
            dest_size: Some(vec2(108.0 - i as f32 * 24.0, 108.0 - i as f32 * 24.0)),
            ..Default::default()
        });
    }

    // Distant abyssal ruin arches lurking in trench mist
    let hall = &tex.aqua.ruin_hall;
    for k in 0..3 {
        let x = layer_x(k, cam, 0.11, 140.0, 960.0);
        draw_texture_ex(hall, x, 560.0 - 210.0, c(36, 32, 74, 150), DrawTextureParams {
            dest_size: Some(vec2(260.0, 200.0)),
            ..Default::default()
        });
    }

    // Slow drifting bioluminescent marine snow
    for i in 0..28 {
        let seed = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(19);
        let sx = (((seed >> 33) % 1024) as f32 + t * 8.0 + (t * 0.7 + i as f32).sin() * 18.0) % 1024.0;
        let sy = (((seed >> 20) % 520) as f32 + t * 14.0) % 540.0;
        let a = 110 + ((seed >> 8) % 90) as u8;
        draw_circle(sx, sy, 1.5, c(166, 240, 255, a));
    }
}

fn home_sky(fx: &crate::art::fx::Fx, t: f32, _cam: f32) {
    let w = screen_width();
    fx.sky_draw(Biome::Home, 0.0, 0.0, w, 470.0);
    for i in 0..3 {
        let x = 150.0 + i as f32 * 300.0;
        let lean = (t * 0.3 + i as f32).sin() * 0.05;
        fx.beam_strip(x, 0.0, 320.0, 520.0, c(255, 220, 148, 22), lean);
    }
    sparkle_stars(fx, t, 85.0, 425.0, 26, c(255, 236, 198, 255), 0.34, 200);
}

pub fn draw_cosma_stars(fx: &crate::art::fx::Fx, t: f32) {
    let mut seed = 7u64;
    for idx in 0..95usize {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let x = ((seed >> 33) % 1024) as f32 + 1.0;
        let y = ((seed >> 20) % 680) as f32 + 1.0;
        let col = match (seed >> 8) % 4 {
            0 => c(255, 230, 110, 255),
            1 => c(110, 190, 255, 255),
            2 => c(255, 118, 168, 255),
            _ => c(238, 242, 255, 255),
        };
        let tw = 0.40 + 0.60 * (t * 1.6 + (x % 13.0) * 0.7 + (y % 7.0)).sin().abs();
        if (seed >> 41) % 15 == 0 {
            let s = 15.0 + ((seed >> 30) % 4) as f32 * 6.0;
            fx.light_pool(x, y, s * 2.2, Color::new(col.r, col.g, col.b, 0.11 * tw));
            let spr = fx.sprites.star_at(idx);
            draw_texture_ex(spr, x - s / 2.0, y - s / 2.0, Color::new(col.r, col.g, col.b, tw),
                DrawTextureParams {
                    dest_size: Some(Vec2::new(s, s)),
                    rotation: (seed % 30) as f32 * 0.21,
                    ..Default::default()
                });
        } else {
            let s = 1.6 + ((seed >> 35) % 2) as f32 * 0.9;
            draw_rectangle(x, y, s, s, Color::new(col.r, col.g, col.b, tw));
        }
    }
}
