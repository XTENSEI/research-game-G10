//! Art loading and backdrop rendering.
//!
//! Characters come from the Kenney Platformer Characters pack (CC0) recolored
//! to the candy tokens by assets/tools/build.py: every cast member is emitted
//! as a uniform 96x144 (2:3) canvas in three frames - stand, walk1, walk2 -
//! so the whole game keeps drawing the same 2:3 boxes while River and friends
//! now get real walk cycles. Tiles stay 16px Kenney roguelike remapped to the
//! lemon/licorice palette and are sampled nearest-neighbour so the pixel art
//! stays crisp at 3x. Biome skies are layered, camera-parallaxed and alive.

pub mod fx;

use macroquad::prelude::*;
use crate::game::script::Biome;
use std::collections::HashMap;

pub struct TilePack {
    pub ground: Texture2D,
    pub ground_alt: Texture2D,
    pub ground_speck: Texture2D,
    pub dirt: Texture2D,
    pub flower_a: Texture2D,
    pub flower_b: Texture2D,
    pub flower_c: Texture2D,
    pub bush_top: Texture2D,
    pub bush_mid: Texture2D,
    pub bush_base: Texture2D,
    pub stone: Texture2D,
    pub stone_alt: Texture2D,
    // Home interior furniture (verified visually in the roguelike band probe)
    pub bed_head: Texture2D,
    pub bed_foot: Texture2D,
    pub bed_wide: Texture2D,
    pub table_a: Texture2D,
    pub table_b: Texture2D,
    pub door: Texture2D,
    pub window: Texture2D,
    pub rug: Texture2D,
    pub fire_lit: Texture2D,
    pub fire_dark: Texture2D,
    pub shelf: Texture2D,
    pub wood_wall: Texture2D,
    pub wood_wall_alt: Texture2D,
    pub planks: Texture2D,
}

/// Aquarius fauna and flora from the Kenney Fish Pack (tight-cropped).
pub struct Aqua {
    pub fish: Vec<Texture2D>,
    pub weed: Vec<Texture2D>,
    pub rock: Vec<Texture2D>,
    /// Background kelp / rock silhouettes drawn far behind the fish.
    pub bg: Vec<Texture2D>,
}

/// Cosma void set pieces + floor (Kenney Space Kit & Planets, CC0, recolored
/// to the neon tokens by assets/tools/build.py).
pub struct Cosma {
    /// Distant drifting asteroids & void-stone boulders (far parallax).
    pub boulders: Vec<Texture2D>,
    /// Near floating neon crystals (teal / magenta / violet).
    pub crystals: Vec<Texture2D>,
    /// Tiny rim sparkles that skim the walkway edge.
    pub pebbles: Vec<Texture2D>,
    /// Deep indigo castle-stone floor tiles (3 variants + lit rim + veins).
    pub floor: Vec<Texture2D>,
    pub rim: Texture2D,
    /// Neon line-art circuit panels (OGA Neon Node, CC0) inlaid in the
    /// walkway and behind the quiz slab styling.
    pub neon_nodes: Vec<Texture2D>,
    pub planets: Vec<Texture2D>,
}

pub struct Textures {
    pub river: Texture2D,
    pub jin: Texture2D,
    pub kandy: Texture2D,
    pub matt: Texture2D,
    pub genie: Texture2D,
    pub guard_a: Texture2D,
    pub guard_b: Texture2D,
    pub nadia: Texture2D,
    pub grandparent: Texture2D,
    pub gill: Texture2D,
    pub scal: Texture2D,
    pub lady: Texture2D,
    pub star: Texture2D,
    pub dying_star: Texture2D,
    pub tiles: TilePack,
    pub aqua: Aqua,
    pub cosma: Cosma,
    /// Walk cycles keyed by cast id (used by the overworld).
    pub walks: HashMap<&'static str, [Texture2D; 2]>,
    /// Baked contact shadows cut from each sprite's own alpha feet.
    pub shadows: HashMap<&'static str, Texture2D>,
}

const COSMA_BOULDERS: [&str; 4] = ["boulder_a", "boulder_b", "meteor_a", "meteor_b"];
const COSMA_CRYSTALS: [&str; 3] = ["crystal_teal", "crystal_pink", "crystal_violet"];
const COSMA_PEBBLES: [&str; 2] = ["pebble_cyan", "pebble_pink"];
const COSMA_FLOOR: [&str; 3] = ["floor_a", "floor_b", "floor_c"];
const COSMA_VEINS: [&str; 2] = ["vein_pink", "vein_cyan"];
const COSMA_NEON_NODES: [&str; 5] =
    ["neon_node_0", "neon_node_1", "neon_node_2", "neon_node_3", "neon_node_4"];
const COSMA_PLANETS: [&str; 7] = [
    "planet00", "planet01", "planet02", "planet03", "planet07", "planet08", "planet09",
];

async fn cosma_tex(name: &str) -> Texture2D {
    let path = format!("assets/game/cosma/{}.png", name);
    macroquad::texture::load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e))
}

async fn load_cosma() -> Cosma {
    // Floor is 16px pixel art drawn at 3x -> nearest; the rest are larger
    // soft-shaded pieces kept linear so upscaling stays smooth.
    let mut floor = Vec::with_capacity(COSMA_FLOOR.len() + COSMA_VEINS.len());
    for n in COSMA_FLOOR.iter().chain(COSMA_VEINS.iter()) {
        let mut t = cosma_tex(n).await;
        crisp(&mut t);
        floor.push(t);
    }
    let mut rim = cosma_tex("floor_rim").await;
    crisp(&mut rim);
    let mut neon_nodes = Vec::with_capacity(COSMA_NEON_NODES.len());
    for n in COSMA_NEON_NODES {
        let mut t = cosma_tex(n).await;
        crisp(&mut t);
        neon_nodes.push(t);
    }
    let mut boulders = Vec::with_capacity(COSMA_BOULDERS.len());
    for n in COSMA_BOULDERS {
        boulders.push(cosma_tex(n).await);
    }
    let mut crystals = Vec::with_capacity(COSMA_CRYSTALS.len());
    for n in COSMA_CRYSTALS {
        crystals.push(cosma_tex(n).await);
    }
    let mut pebbles = Vec::with_capacity(COSMA_PEBBLES.len());
    for n in COSMA_PEBBLES {
        pebbles.push(cosma_tex(n).await);
    }
    let mut planets = Vec::with_capacity(COSMA_PLANETS.len());
    for n in COSMA_PLANETS {
        planets.push(cosma_tex(n).await);
    }
    Cosma {
        boulders,
        crystals,
        pebbles,
        floor,
        rim,
        neon_nodes,
        planets,
    }
}

const AQUA_FISH: [&str; 9] = [
    "fish_blue", "fish_brown", "fish_green", "fish_grey_long_a", "fish_grey_long_b",
    "fish_grey", "fish_orange", "fish_pink", "fish_red",
];
const AQUA_WEED: [&str; 6] = [
    "seaweed_grass_a", "seaweed_grass_b", "seaweed_green_b", "seaweed_green_d",
    "seaweed_pink_a", "seaweed_pink_c",
];
const AQUA_ROCK: [&str; 2] = ["rock_a", "rock_b"];
const AQUA_BG: [&str; 3] = ["background_rock_a", "background_rock_b", "background_seaweed_b"];

async fn aqua_tex(name: &str) -> Texture2D {
    let path = format!("assets/game/aqua/{}.png", name);
    macroquad::texture::load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e))
}

async fn load_aqua() -> Aqua {
    let mut fish = Vec::with_capacity(AQUA_FISH.len());
    for n in AQUA_FISH { fish.push(aqua_tex(n).await); }
    let mut weed = Vec::with_capacity(AQUA_WEED.len());
    for n in AQUA_WEED { weed.push(aqua_tex(n).await); }
    let mut rock = Vec::with_capacity(AQUA_ROCK.len());
    for n in AQUA_ROCK { rock.push(aqua_tex(n).await); }
    let mut bg = Vec::with_capacity(AQUA_BG.len());
    for n in AQUA_BG { bg.push(aqua_tex(n).await); }
    Aqua { fish, weed, rock, bg }
}

const CAST_IDS: [&'static str; 11] = [
    "river", "jin", "guard_a", "guard_b", "kandy", "matt", "nadia", "grandparent",
    "gill", "scal", "lady",
];

async fn char_tex(name: &str) -> Texture2D {
    let path = format!("assets/game/char/{}.png", name);
    macroquad::texture::load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e))
}

/// Optional variant: missing files (e.g. baked shadows for procedural-only
/// sprites) resolve to None instead of panicking.
async fn char_tex_opt(name: &str) -> Option<Texture2D> {
    let path = format!("assets/game/char/{}.png", name);
    macroquad::texture::load_texture(&path).await.ok()
}

/// Walk frames are soft-rendered sprites downsampled at load; the default
/// linear filter keeps them clean (nearest would alias at 0.5x).
async fn frame_tex(name: &str, suffix: &str) -> Texture2D {
    let path = format!("assets/game/char/{}_{}.png", name, suffix);
    macroquad::texture::load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e))
}

/// 16px tiles are drawn at exactly 3x, so nearest sampling keeps them crisp.
fn crisp(tex: &mut Texture2D) {
    tex.set_filter(FilterMode::Nearest);
}

async fn tile_tex(name: &str) -> Texture2D {
    let path = format!("assets/game/tiles/{}.png", name);
    let mut t = macroquad::texture::load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("missing art asset {}: {:?}", path, e));
    crisp(&mut t);
    t
}

async fn load_tiles() -> TilePack {
    TilePack {
        ground: tile_tex("ground").await,
        ground_alt: tile_tex("ground_alt").await,
        ground_speck: tile_tex("ground_speck").await,
        dirt: tile_tex("dirt").await,
        flower_a: tile_tex("flower_a").await,
        flower_b: tile_tex("flower_b").await,
        flower_c: tile_tex("flower_c").await,
        bush_top: tile_tex("bush_top").await,
        bush_mid: tile_tex("bush_mid").await,
        bush_base: tile_tex("bush_base").await,
        stone: tile_tex("stone").await,
        stone_alt: tile_tex("stone_alt").await,
        bed_head: tile_tex("bed_head").await,
        bed_foot: tile_tex("bed_foot").await,
        bed_wide: tile_tex("bed_wide").await,
        table_a: tile_tex("table_a").await,
        table_b: tile_tex("table_b").await,
        door: tile_tex("door").await,
        window: tile_tex("window").await,
        rug: tile_tex("rug").await,
        fire_lit: tile_tex("fire_lit").await,
        fire_dark: tile_tex("fire_dark").await,
        shelf: tile_tex("shelf").await,
        wood_wall: tile_tex("wood_wall").await,
        wood_wall_alt: tile_tex("wood_wall_alt").await,
        planks: tile_tex("planks").await,
    }
}

// --- procedural remnants (no sprite sheet exists for these) ----------------

const T: Color = Color::new(0.0, 0.0, 0.0, 0.0);

fn px(img: &mut Image, x: i32, y: i32, w: i32, h: i32, c: Color) {
    for yy in y..y + h {
        for xx in x..x + w {
            if xx >= 0 && yy >= 0 {
                img.set_pixel(xx as u32, yy as u32, c);
            }
        }
    }
}

fn make_genie() -> Texture2D {
    let mut img = Image::gen_image_color(16, 24, T);
    let flame = Color::from_rgba(255, 120, 40, 255);
    px(&mut img, 5, 4, 6, 12, flame);
    px(&mut img, 4, 6, 8, 8, flame);
    px(&mut img, 6, 2, 4, 2, Color::from_rgba(255, 220, 80, 255));
    px(&mut img, 6, 4, 4, 3, Color::from_rgba(255, 220, 80, 255));
    px(&mut img, 7, 5, 1, 1, Color::from_rgba(180, 40, 0, 255));
    px(&mut img, 9, 5, 1, 1, Color::from_rgba(180, 40, 0, 255));
    px(&mut img, 5, 8, 2, 4, Color::from_rgba(255, 180, 60, 255));
    px(&mut img, 10, 8, 2, 4, Color::from_rgba(255, 180, 60, 255));
    Texture2D::from_image(&img)
}

fn make_star(dying: bool) -> Texture2D {
    let mut img = Image::gen_image_color(8, 8, T);
    let c = if dying {
        Color::from_rgba(120, 60, 60, 255)
    } else {
        Color::from_rgba(255, 240, 150, 255)
    };
    px(&mut img, 3, 0, 2, 2, c);
    px(&mut img, 3, 6, 2, 2, c);
    px(&mut img, 0, 3, 2, 2, c);
    px(&mut img, 6, 3, 2, 2, c);
    px(&mut img, 2, 2, 4, 4, c);
    let core = if dying {
        Color::from_rgba(200, 90, 90, 255)
    } else {
        Color::from_rgba(255, 255, 220, 255)
    };
    px(&mut img, 3, 3, 2, 2, core);
    Texture2D::from_image(&img)
}

impl Textures {
    pub async fn load() -> Self {
        let mut walks = HashMap::new();
        let mut shadows = HashMap::new();
        for id in CAST_IDS {
            let w1 = frame_tex(id, "w1").await;
            let w2 = frame_tex(id, "w2").await;
            walks.insert(id, [w1, w2]);
            if let Some(sh) = char_tex_opt(&format!("{}_shadow", id)).await {
                shadows.insert(id, sh);
            }
        }
        Self {
            river: char_tex("river").await,
            jin: char_tex("jin").await,
            kandy: char_tex("kandy").await,
            matt: char_tex("matt").await,
            genie: make_genie(),
            guard_a: char_tex("guard_a").await,
            guard_b: char_tex("guard_b").await,
            nadia: char_tex("nadia").await,
            grandparent: char_tex("grandparent").await,
            gill: char_tex("gill").await,
            scal: char_tex("scal").await,
            lady: char_tex("lady").await,
            star: make_star(false),
            dying_star: make_star(true),
            tiles: load_tiles().await,
            aqua: load_aqua().await,
            cosma: load_cosma().await,
            walks,
            shadows,
        }
    }

    /// Baked contact shadow for a cast id (None for procedural-only sprites).
    pub fn shadow_of(&self, id: &str) -> Option<&Texture2D> {
        self.shadows.get(id)
    }

    /// Big dialogue portrait for a speaker name.
    #[allow(dead_code)]
    pub fn portrait(&self, who: &str) -> Option<&Texture2D> {
        Some(match who {
            "River" => &self.river,
            "Jin" => &self.jin,
            "Kandy Kween" => &self.kandy,
            "Matt" => &self.matt,
            "Magic Fire Genie" => &self.genie,
            "Guard 1" => &self.guard_a,
            "Guard 2" => &self.guard_b,
            "Nadia" => &self.nadia,
            "Grandparent" => &self.grandparent,
            "Gillbert" => &self.gill,
            "Scalleon" => &self.scal,
            "Market Lady" => &self.lady,
            _ => return None,
        })
    }

    /// Walk cycle for a cast id, if one exists.
    pub fn walk_of(&self, id: &str) -> Option<&[Texture2D; 2]> {
        self.walks.get(id)
    }
}

/// Draw a 16px tile texture scaled to `size` screen px at (x, y).
pub fn draw_tile(tex: &Texture2D, x: f32, y: f32, size: f32) {
    draw_texture_ex(tex, x, y, WHITE, DrawTextureParams {
        dest_size: Some(Vec2::new(size, size)),
        ..Default::default()
    });
}

/// Tile draw with a tint (e.g. sea-stone / void-indigo riddle monoliths).
pub fn draw_tile_tint(tex: &Texture2D, x: f32, y: f32, size: f32, tint: Color) {
    draw_texture_ex(tex, x, y, tint, DrawTextureParams {
        dest_size: Some(Vec2::new(size, size)),
        ..Default::default()
    });
}

// --- palette + small helpers ------------------------------------------------

pub fn c(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

/// Layered parallax loop: objects spaced `screen_span` apart on screen, shifted
/// by `cam * factor` plus drift.
fn layer_x(k: i32, cam: f32, factor: f32, drift: f32, span: f32) -> f32 {
    let base = (cam * factor + drift).rem_euclid(span);
    k as f32 * span - base
}

/// Sprite-based twinkle field. Same layout every frame (seeded, not random) so
/// sparkles stay anchored to the world instead of crawling.
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
        let s = 9.0 + ((seed >> 11) % 3) as f32 * 7.0;
        let col = Color::new(hue.r, hue.g, hue.b, tw);
        let spr = fx.sprites.star_at(idx as usize);
        draw_texture_ex(spr, x - s / 2.0, y - s / 2.0, col, DrawTextureParams {
            dest_size: Some(Vec2::new(s, s)),
            rotation: (seed % 40) as f32 * 0.13,
            ..Default::default()
        });
    }
}

/// Soft vertical haze: translucent bands fading from `a0` to `a1` alpha. Welds
/// the sky into the ground line and gives distance a colour it can breathe in.
fn haze_band(y0: f32, y1: f32, col: Color, a0: f32, a1: f32, steps: u8) {
    let w = screen_width();
    let n = steps.max(1) as f32;
    let band = (y1 - y0) / n + 1.0;
    for i in 0..steps {
        let f = i as f32 / n;
        let a = a0 + (a1 - a0) * f;
        if a <= 0.002 {
            continue;
        }
        draw_rectangle(0.0, y0 + (y1 - y0) * f, w, band, Color::new(col.r, col.g, col.b, a));
    }
}

// --- biome skies -------------------------------------------------------------

/// Vertical gradient backdrop per biome, drawn each frame. `cam` is the world
/// camera x so every sky element parallaxes at its own depth, and `fx` supplies
/// the baked ramps, cloud and light sprites that keep the sky from reading as
/// stacked primitives.
pub fn draw_biome_bg(biome: Biome, t: f32, cam: f32, fx: &crate::art::fx::Fx) {
    match biome {
        Biome::Candy => candy_sky(fx, t, cam),
        Biome::Aquarius => aqua_sky(fx, t, cam),
        Biome::Home => home_sky(fx, t, cam),
        Biome::Cosma => cosma_sky(fx, t, cam),
    }
}

fn candy_sky(fx: &crate::art::fx::Fx, t: f32, cam: f32) {
    let w = screen_width();

    // 1. sky: one baked 512-row ramp stretched to the ground line. Linear
    //    filtering across those rows is what finally kills the banding the old
    //    stack of ~130 rectangles showed at this size.
    fx.sky_draw(Biome::Candy, 0.0, 0.0, w, 560.0);

    // 2. sun: an authored stack of soft light pools plus an anamorphic streak,
    //    so the halo has a falloff instead of visible concentric steps.
    let sx = 212.0 + (t * 0.13).sin() * 9.0 - (cam * 0.02).rem_euclid(680.0);
    let sy = 108.0 + (t * 0.26).sin() * 6.0;
    let pulse = 0.5 + 0.5 * (t * 0.85).sin();
    fx.glow_rect(sx, sy, 470.0, 470.0, c(255, 188, 100, (7.0 + pulse * 3.0) as u8));
    fx.glow_rect(sx, sy, 250.0, 250.0, c(255, 204, 126, (13.0 + pulse * 5.0) as u8));
    fx.glow_rect(sx, sy, 132.0, 132.0, c(255, 224, 164, (26.0 + pulse * 9.0) as u8));
    fx.glow_rect(sx, sy, 520.0, 18.0, c(255, 228, 176, (10.0 + pulse * 4.0) as u8));
    fx.glow_rect(sx, sy, 18.0, 220.0, c(255, 228, 176, 8));
    for i in 0..12 {
        let a = t * 0.09 + i as f32 * std::f32::consts::TAU / 12.0;
        let l0 = 45.0;
        let l1 = 64.0 + (t * 0.6 + i as f32 * 1.9).sin().abs() * 24.0;
        let (dx, dy) = (a.cos(), a.sin());
        draw_triangle(
            Vec2::new(sx + dx * l0 - dy * 4.0, sy + dy * l0 + dx * 4.0),
            Vec2::new(sx + dx * l0 + dy * 4.0, sy + dy * l0 - dx * 4.0),
            Vec2::new(sx + dx * l1, sy + dy * l1),
            c(255, 240, 198, 38),
        );
    }
    draw_circle(sx, sy, 33.0, c(255, 196, 106, 255));
    draw_circle(sx, sy, 25.0, c(255, 230, 164, 255));
    draw_circle(sx, sy, 18.0, c(255, 250, 226, 255));

    // 3. clouds: metaball-baked sprites at three parallax depths
    let depths: [(f32, f32, f32, f32, f32, f32, f32); 3] = [
        (0.028, 980.0, 7.0, 48.0, 168.0, 390.0, 150.0),
        (0.060, 640.0, 13.0, 82.0, 240.0, 250.0, 182.0),
        (0.108, 470.0, 21.0, 126.0, 300.0, 152.0, 198.0),
    ];
    for (fct, span, spd, ymin, ymax, cw, al) in depths {
        let n = (w / span).ceil() as i32 + 3;
        for k in -1..n + 1 {
            let x = layer_x(k, cam, fct, t * spd, span);
            if x < -cw || x > w + cw {
                continue;
            }
            let mut seed = (k as i64).wrapping_mul(2654435761) as u64;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(7);
            let y = ymin + (((seed >> 8) % 1000) as f32 / 1000.0) * (ymax - ymin);
            let wob = 5.0 * (t * 0.35 + (seed >> 4) as f32 * 0.013).sin();
            let scale = 0.72 + ((seed >> 22) % 100) as f32 / 100.0 * 0.66;
            // lit cotton-candy top over a pink belly
            let body = c(255, 234, 246, al as u8);
            let shade = c(246, 166, 208, (al * 0.85) as u8);
            fx.cloud(x + ((seed >> 40) % 60) as f32 - 30.0, y + wob, cw * scale, body, shade);
        }
    }

    // 4. distant candy hills: two scalloped silhouette layers that give the
    //    horizon something to sit against instead of an empty field of pink
    let hills: [(f32, Color, f32, f32); 2] = [
        (0.042, c(255, 186, 224, 170), 168.0, 74.0),
        (0.082, c(246, 158, 208, 208), 118.0, 48.0),
    ];
    for (fct, col, span, amp) in hills {
        let n = (w / span).ceil() as i32 + 2;
        for k in -1..n + 1 {
            let x = layer_x(k, cam, fct, 0.0, span);
            draw_ellipse(x + span * 0.5, 508.0 + amp * 0.46, span * 0.64, amp, 0.0, col);
        }
    }

    // 5. horizon haze welding sky to meadow, kept light so the lower sky keeps
    //    its colour rather than bleaching to white
    haze_band(452.0, 508.0, c(255, 228, 244, 255), 0.0, 0.26, 10);

    // 6. sugar sparkles
    sparkle_stars(fx, t, 18.0, 320.0, 26, c(255, 250, 255, 255), 0.55, 77);

    // 7. candy balloons on a far parallax band
    let span = 900.0;
    for k in -1..3 {
        let x = layer_x(k, cam, 0.15, t * 9.0, span);
        if x < -74.0 || x > w + 74.0 {
            continue;
        }
        let seed = k.wrapping_mul(97) as u64;
        let bob = (t * 1.0 + k as f32).sin();
        let y = 172.0 + ((seed * 7919) % 130) as f32 + bob * 13.0;
        let col = [c(255, 118, 168, 226), c(132, 220, 208, 226), c(255, 212, 104, 226)]
            [(k.rem_euclid(3)) as usize];
        let bx = x + bob * 8.0;
        for i in 0..5 {
            let wob = (t * 3.0 + i as f32 * 0.7 + k as f32).sin() * 4.0;
            draw_line(bx + wob, y + 22.0 + i as f32 * 9.0, bx + wob * 0.7, y + 31.0 + i as f32 * 9.0,
                1.4, c(96, 64, 92, 140));
        }
        fx.glow_rect(bx, y, 104.0, 118.0, Color::new(col.r, col.g, col.b, 0.10));
        draw_ellipse(bx, y, 17.0, 20.0, 0.0, col);
        draw_triangle(Vec2::new(bx - 6.0, y + 17.0), Vec2::new(bx + 6.0, y + 17.0),
            Vec2::new(bx, y + 25.0), col);
        // specular gloss
        draw_ellipse(bx - 5.5, y - 6.5, 4.2, 5.6, -0.4, Color::new(1.0, 1.0, 1.0, 0.42));
    }
}

fn aqua_sky(fx: &crate::art::fx::Fx, t: f32, cam: f32) {
    let w = screen_width();

    // Gradient runs to the sand line (560) so no clear-colour band can show
    // between the water column and the sea floor.
    fx.sky_draw(Biome::Aquarius, 0.0, 0.0, w, 560.0);

    // 1. the surface far overhead: the biome's actual light source. Without a
    //    visible ceiling the column reads as a blue sky, not as water.
    for i in 0..4 {
        let yy = -6.0 + i as f32 * 10.0;
        let n = (w / 10.0) as i32 + 1;
        for k in 0..n {
            let px = k as f32 * 10.0;
            let wv = ((px + cam * 0.35) * 0.014 + t * 0.85 + i as f32 * 1.7).sin();
            let a = (22.0 + 26.0 * wv).max(0.0) as u8;
            if a == 0 {
                continue;
            }
            draw_rectangle(px, yy + wv * 6.0, 10.0, 3.0, c(206, 244, 255, a));
        }
    }
    fx.glow_rect(w * 0.5, -80.0, 1400.0, 560.0, c(168, 226, 255, 22));

    // 2. god rays leaning through the column
    for i in 0..6 {
        let x = layer_x(i, cam, 0.035, 0.0, 780.0) + 40.0;
        let lean = (t * 0.22 + i as f32 * 1.7).sin() * 0.11;
        let flick = 0.5 + 0.5 * (t * 0.5 + i as f32 * 2.3).sin();
        let a = 0.048 + flick * 0.070;
        fx.beam_strip(x, -40.0, 140.0 + i as f32 * 24.0, 600.0,
            c(196, 238, 255, (a * 255.0) as u8), lean);
    }

    // 3. drifting shimmer blobs
    for i in 0..7 {
        let x = layer_x(i, cam, 0.075, t * 26.0, 700.0);
        let y = 70.0 + ((i * 211) % 290) as f32 + (t * 0.7 + i as f32).sin() * 20.0;
        let r = 150.0 + ((i * 47) % 90) as f32;
        let a = 0.028 + 0.020 * (t * 0.8 + i as f32).sin().abs();
        fx.light_sprite(x, y, r, c(190, 232, 255, (a * 255.0) as u8), i as usize);
    }

    // 4. distant reef silhouettes at two depths with lit crests
    let depths: [(f32, Color, f32, Color); 2] = [
        (0.055, c(20, 70, 110, 255), 56.0, c(30, 92, 138, 255)),
        (0.125, c(11, 44, 76, 255), 40.0, c(17, 60, 98, 255)),
    ];
    for (fct, col, amp, crest) in depths {
        let step = 22.0;
        let n = (w / step) as i32 + 2;
        let off = (cam * fct).rem_euclid(step * 6.0);
        for i in -1..n + 1 {
            let sxx = i as f32 * step - off;
            let hh = 42.0 + amp * ((sxx * 0.05 + i as f32 * 1.7).sin() * 0.5 + 0.5)
                + 13.0 * (sxx * 0.021 - i as f32).sin();
            let hh = hh.max(14.0);
            draw_rectangle(sxx, 560.0 - hh, step + 1.0, hh, col);
            // rounded crown: without it the reef skyline was a row of hard
            // rectangles that read as a city block, not a reef.
            draw_ellipse(sxx + step * 0.5, 560.0 - hh, step * 0.86, 11.0, 0.0, col);
            draw_ellipse(sxx + step * 0.5, 560.0 - hh, step * 0.72, 5.0, 0.0, crest);
        }
    }

    // 5. rising bubbles: soft ring sprite plus a specular dot
    for i in 0..14 {
        let seed = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(3);
        let bx = ((seed >> 33) % 1024) as f32;
        let cyc = (t * 0.28 + i as f32 * 0.11) % 1.0;
        let y = 560.0 - cyc * 700.0;
        let sway = (t * 0.8 + i as f32 * 1.3).sin() * 26.0;
        let r = 9.0 + ((seed >> 20) % 26) as f32 + (1.0 - cyc) * 8.0;
        let a = 0.18 + 0.32 * (1.0 - cyc);
        let spr = &fx.sprites.circle[i % 3];
        draw_texture_ex(spr, bx + sway - r, y - r, c(214, 242, 255, (a * 255.0) as u8),
            DrawTextureParams { dest_size: Some(Vec2::new(r * 2.0, r * 2.0)), ..Default::default() });
        draw_circle(bx + sway - r * 0.32, y - r * 0.34, 1.8,
            c(240, 255, 255, (a * 330.0).min(255.0) as u8));
    }

    // 6. warm seam light where the water meets the sand
    haze_band(498.0, 560.0, c(255, 236, 190, 255), 0.0, 0.08, 9);
}

fn home_sky(fx: &crate::art::fx::Fx, t: f32, _cam: f32) {
    let w = screen_width();
    // warm room: cream wall glow from an unseen window, wood trim below
    fx.sky_draw(Biome::Home, 0.0, 0.0, w, 470.0);

    // lamplight shafts from above
    for i in 0..3 {
        let x = 150.0 + i as f32 * 300.0;
        let lean = (t * 0.3 + i as f32).sin() * 0.05;
        fx.beam_strip(x, 0.0, 330.0, 520.0, c(255, 232, 168, 20), lean);
    }
    // dust motes drifting in the light
    sparkle_stars(fx, t, 90.0, 430.0, 30, c(255, 240, 205, 255), 0.34, 200);

    // wall baseboard
    draw_rectangle(0.0, 448.0, w, 11.0, c(158, 116, 80, 255));
    draw_rectangle(0.0, 448.0, w, 3.0, c(120, 84, 56, 255));
}

fn cosma_sky(fx: &crate::art::fx::Fx, t: f32, cam: f32) {
    let w = screen_width();
    let h = screen_height();
    fx.sky_draw(Biome::Cosma, 0.0, 0.0, w, h);

    // 1. nebula: huge soft metaball clouds in three colour families. Replaces
    //    the concentric translucent circles, which read as onion rings.
    let fams: [(Color, Color); 3] = [
        (c(96, 62, 178, 30), c(150, 90, 220, 20)),
        (c(168, 52, 128, 24), c(214, 92, 176, 16)),
        (c(40, 116, 168, 20), c(80, 176, 210, 13)),
    ];
    for (fi, (body, shade)) in fams.iter().enumerate() {
        let span = 1180.0 + fi as f32 * 300.0;
        let n = (w / span).ceil() as i32 + 2;
        for k in -1..n + 1 {
            let x = layer_x(k, cam, 0.02, t * 2.2 + fi as f32 * 300.0, span);
            if x < -720.0 || x > w + 720.0 {
                continue;
            }
            let seed = ((k as i64 + fi as i64 * 31).wrapping_mul(2654435761)) as u64;
            let y = 20.0 + ((seed >> 12) % 420) as f32;
            let size = 540.0 + ((seed >> 4) % 280) as f32;
            fx.cloud(x, y, size, *body, *shade);
            fx.cloud(x + size * 0.30, y + 66.0, size * 0.72, *shade, *body);
        }
    }

    // 2. aurora ribbons: soft beams laid nearly flat
    for i in 0..4 {
        let x = layer_x(i, cam, 0.012, t * 5.0, 1100.0);
        let y = 118.0 + i as f32 * 48.0 + (t * 0.4 + i as f32).sin() * 26.0;
        let lean = -0.16 + (t * 0.15 + i as f32).sin() * 0.10;
        let a = 0.045 + 0.035 * (t * 0.5 + i as f32 * 1.4).sin().abs();
        fx.beam_strip(x, y, 800.0, 86.0, c(90, 240, 230, (a * 255.0) as u8), lean);
    }

    draw_cosma_stars(fx, t);

    // 3. shooting star with a streaking trace sprite
    let cyc = t % 6.0;
    if cyc < 1.5 {
        let seed = ((t / 6.0) as u64).wrapping_mul(6364136223846793005).wrapping_add(7);
        let sx = (seed >> 33) % 640;
        let sy = 26 + (seed >> 20) % 150;
        let x = sx as f32 + cyc * 780.0;
        let y = sy as f32 + cyc * 230.0;
        let a = (1.0 - cyc / 1.5).powf(0.8);
        let spr = &fx.sprites.trace[0];
        draw_texture_ex(spr, x - 46.0, y - 46.0, c(255, 250, 220, (a * 190.0) as u8),
            DrawTextureParams {
                dest_size: Some(Vec2::new(92.0, 92.0)),
                rotation: 0.29,
                ..Default::default()
            });
        draw_circle(x, y, 2.4, c(255, 255, 245, (a * 255.0) as u8));
        fx.light_pool(x, y, 120.0, Color::new(1.0, 0.98, 0.82, a * 0.16));
    }
}

fn draw_cosma_stars(fx: &crate::art::fx::Fx, t: f32) {
    let mut seed = 7u64;
    for idx in 0..110usize {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let x = ((seed >> 33) % 1024) as f32 + 1.0;
        let y = ((seed >> 20) % 720) as f32 + 1.0;
        let col = match (seed >> 8) % 4 {
            0 => c(255, 230, 90, 255),
            1 => c(90, 160, 255, 255),
            2 => c(255, 90, 110, 255),
            _ => c(240, 240, 255, 255),
        };
        let tw = 0.45 + 0.55 * (t * 1.8 + (x % 13.0) * 0.7 + (y % 7.0)).sin().abs();
        if (seed >> 41) % 17 == 0 {
            // hero star: neon 4-point sprite over a soft pool
            let s = 16.0 + ((seed >> 30) % 4) as f32 * 7.0;
            fx.light_pool(x, y, s * 2.4, Color::new(col.r, col.g, col.b, 0.10 * tw));
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
