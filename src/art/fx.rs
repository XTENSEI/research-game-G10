//! Screen-space effects: baked atmosphere textures, the Kenney particle sprite
//! kit, a post-processing compositor and camera feel.
//!
//! Two halves:
//!
//! * [`FxSprites`] - the 32 CC0 particle sprites emitted by
//!   `assets/tools/build.py` from the Kenney Particle Pack. Soft-alpha glows,
//!   smoke wisps, sparks, twirls and streaks. Every effect in the game draws
//!   from this kit instead of from `draw_circle`, which is what stops the
//!   particles reading as programmer art.
//!
//! * [`Fx`] - a render-target compositor. The scene is drawn into an offscreen
//!   texture, then composited with bloom, a per-biome colour grade, a baked
//!   vignette, film grain and a star-shaped iris transition. It also owns the
//!   camera feel (shake, punch-zoom, hit-stop, screen flash).
//!
//! Blending note: macroquad 0.4 exposes no blend-mode switch, and a custom
//! additive pipeline would mean maintaining GLSL for the web build and MSL for
//! Metal. The bloom is therefore built from saturating multi-pass draws - the
//! same bright-on-dark look without a shader, and the wasm build stays
//! shader-free.

use macroquad::prelude::*;
use crate::game::script::Biome;

/// Render-target textures are stored bottom-up, so compositing them back onto
/// the screen needs a vertical flip.
const FLIP: bool = true;

// ---------------------------------------------------------------------------
// baked texture helpers
// ---------------------------------------------------------------------------

fn blank(w: u16, h: u16) -> Image {
    Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0))
}

fn finish_tex(img: &Image, linear: bool) -> Texture2D {
    let t = Texture2D::from_image(img);
    if linear {
        t.set_filter(FilterMode::Linear);
    }
    t
}

/// Vertical multi-stop gradient. 512 rows stretched over the screen interpolate
/// smoothly under linear filtering, which removes the hard banding the old
/// per-biome skies got from stacking ~130 discrete rectangles.
fn bake_gradient(stops: &[(f32, Color)]) -> Texture2D {
    const ROWS: u32 = 512;
    let mut img = blank(8, ROWS as u16);
    for y in 0..ROWS {
        let f = y as f32 / (ROWS - 1) as f32;
        let mut col = stops[0].1;
        for w in stops.windows(2) {
            let (a, ca) = w[0];
            let (b, cb) = w[1];
            if f >= a && f <= b {
                let t = if b > a { (f - a) / (b - a) } else { 0.0 };
                // smoothstep the blend so stop joins are not visible as creases
                let s = t * t * (3.0 - 2.0 * t);
                col = Color::new(
                    ca.r + (cb.r - ca.r) * s,
                    ca.g + (cb.g - ca.g) * s,
                    ca.b + (cb.b - ca.b) * s,
                    ca.a + (cb.a - ca.a) * s,
                );
                break;
            }
            col = cb;
        }
        for x in 0..8 {
            img.set_pixel(x, y, col);
        }
    }
    finish_tex(&img, true)
}

/// Radial falloff disc: the workhorse for light pools, flares and contact
/// glows. `power` controls how tight the core is.
fn bake_glow(size: u16, power: f32) -> Texture2D {
    let mut img = blank(size, size);
    let r = size as f32 / 2.0;
    for y in 0..size as u32 {
        for x in 0..size as u32 {
            let dx = (x as f32 + 0.5 - r) / r;
            let dy = (y as f32 + 0.5 - r) / r;
            let d = (dx * dx + dy * dy).sqrt().min(1.0);
            let a = (1.0 - d).powf(power);
            if a > 0.002 {
                img.set_pixel(x, y, Color::new(1.0, 1.0, 1.0, a));
            }
        }
    }
    finish_tex(&img, true)
}

/// Full-screen vignette: transparent through the middle, gently darkening
/// toward the corners. Baked so the falloff is analytic instead of the visible
/// concentric rings a stacked-circles approach produces.
fn bake_vignette(w: u16, h: u16) -> Texture2D {
    let mut img = blank(w, h);
    let (hw, hh) = (w as f32 / 2.0, h as f32 / 2.0);
    let maxd = (hw * hw + hh * hh).sqrt();
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let dx = x as f32 + 0.5 - hw;
            let dy = y as f32 + 0.5 - hh;
            let d = (dx * dx + dy * dy).sqrt() / maxd;
            let t = ((d - 0.42) / 0.62).clamp(0.0, 1.0);
            // 0.44: at 0.60 the corners of the darker biomes crushed to pure
            // black, which read as the frame being clipped rather than lit
            let a = t * t * (3.0 - 2.0 * t) * 0.44;
            if a > 0.002 {
                img.set_pixel(x, y, Color::new(0.02, 0.01, 0.05, a));
            }
        }
    }
    finish_tex(&img, true)
}

/// Star-shaped iris mask used for scene transitions: opaque outside a star
/// boundary, transparent inside. Scaling it down closes the iris.
fn bake_iris(size: u16) -> Texture2D {
    let mut img = blank(size, size);
    let r = size as f32 / 2.0;
    for y in 0..size as u32 {
        for x in 0..size as u32 {
            let dx = x as f32 + 0.5 - r;
            let dy = y as f32 + 0.5 - r;
            let d = (dx * dx + dy * dy).sqrt();
            let ang = dy.atan2(dx);
            // 8-point star boundary with a gentle lens-flare style bulge
            let edge = r * (0.90 + 0.10 * (ang * 8.0).cos());
            let t = ((d - edge + 5.0) / 10.0).clamp(0.0, 1.0);
            let a = t * t * (3.0 - 2.0 * t);
            img.set_pixel(x, y, Color::new(0.0, 0.0, 0.0, a));
        }
    }
    finish_tex(&img, true)
}

/// Animated film grain frame. Half the specks lift, half darken, so the grain
/// reads as texture rather than as a dirty screen.
fn bake_grain(size: u16, seed: u64) -> Texture2D {
    let mut img = blank(size, size);
    let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    for y in 0..size as u32 {
        for x in 0..size as u32 {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r = (s >> 33) as u32 % 1000;
            if r > 300 {
                continue;
            }
            // deliberately faint: grain should read as a texture on the frame,
            // never as noise sitting on top of the artwork
            let a = 0.008 + (r as f32 / 300.0) * 0.030;
            let lift = (s >> 20) % 2 == 0;
            let col = if lift {
                Color::new(1.0, 0.97, 0.92, a)
            } else {
                Color::new(0.05, 0.02, 0.08, a * 1.2)
            };
            img.set_pixel(x, y, col);
        }
    }
    finish_tex(&img, false)
}

/// A puffy cumulus baked from overlapping metaballs and thresholded, so the
/// silhouette is a single soft mass instead of the visible stack of hard
/// circles the old `cloud_` helper drew.
fn bake_cloud(size: u16) -> Texture2D {
    // (cx, cy, sigma) in normalised units
    const BLOBS: [(f32, f32, f32); 8] = [
        (0.30, 0.58, 0.180),
        (0.44, 0.46, 0.215),
        (0.60, 0.50, 0.190),
        (0.72, 0.60, 0.150),
        (0.38, 0.66, 0.165),
        (0.52, 0.66, 0.175),
        (0.65, 0.68, 0.140),
        (0.50, 0.60, 0.230),
    ];
    let mut img = blank(size, size);
    for y in 0..size as u32 {
        for x in 0..size as u32 {
            let u = x as f32 / size as f32;
            let v = y as f32 / size as f32;
            let mut f = 0.0f32;
            for (cx, cy, sg) in BLOBS {
                let dx = u - cx;
                let dy = (v - cy) * 1.35; // slight flatten
                f += (-(dx * dx + dy * dy) / (2.0 * sg * sg)).exp();
            }
            // threshold into a body, with a soft shoulder for anti-aliasing
            let t = ((f - 0.72) / 0.30).clamp(0.0, 1.0);
            let a = t * t * (3.0 - 2.0 * t);
            if a <= 0.004 {
                continue;
            }
            // shade: tops catch light, bellies hold a little shadow
            let shade = (1.0 - ((v - 0.40) / 0.60)).clamp(0.0, 1.0);
            let l = 0.82 + 0.18 * shade;
            img.set_pixel(x, y, Color::new(l, l, l, a));
        }
    }
    finish_tex(&img, true)
}

/// God-ray shaft: bright at the top, dissolving downward, feathered at the
/// sides. Tinted and rotated per use.
fn bake_beam(w: u16, h: u16) -> Texture2D {
    let mut img = blank(w, h);
    for y in 0..h as u32 {
        let v = y as f32 / h as f32;
        let fall = (1.0 - v).powf(1.5);
        for x in 0..w as u32 {
            let u = (x as f32 + 0.5) / w as f32;
            let side = 1.0 - ((u - 0.5).abs() * 2.0);
            let a = fall * side.powf(1.7) * 0.9;
            if a > 0.002 {
                img.set_pixel(x, y, Color::new(1.0, 1.0, 1.0, a));
            }
        }
    }
    finish_tex(&img, true)
}

// ---------------------------------------------------------------------------
// particle sprite kit
// ---------------------------------------------------------------------------

async fn fx_tex(name: &str) -> Texture2D {
    let path = format!("assets/game/fx/{}.png", name);
    let t = macroquad::texture::load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("missing fx sprite {}: {:?}", path, e));
    t.set_filter(FilterMode::Linear);
    t
}

macro_rules! fx_arr {
    ($prefix:expr, $n:literal) => {{
        let mut v = Vec::with_capacity($n);
        for i in 1..=$n {
            v.push(fx_tex(&format!("{}_{}", $prefix, i)).await);
        }
        let mut it = v.into_iter();
        [(); $n].map(|_| it.next().unwrap())
    }};
}

/// The CC0 particle sprite kit (Kenney Particle Pack, emitted at 128px by the
/// asset pipeline).
pub struct FxSprites {
    pub smoke: [Texture2D; 3],
    pub spark: [Texture2D; 3],
    pub star: [Texture2D; 4],
    pub magic: [Texture2D; 3],
    pub fire: [Texture2D; 2],
    pub flame: [Texture2D; 3],
    pub circle: [Texture2D; 3],
    pub trace: [Texture2D; 3],
    pub twirl: [Texture2D; 2],
    pub light: [Texture2D; 3],
    pub dirt: [Texture2D; 2],
    pub flare: Texture2D,
}

impl FxSprites {
    pub async fn load() -> Self {
        Self {
            smoke: fx_arr!("smoke", 3),
            spark: fx_arr!("spark", 3),
            star: fx_arr!("star", 4),
            magic: fx_arr!("magic", 3),
            fire: fx_arr!("fire", 2),
            flame: fx_arr!("flame", 3),
            circle: fx_arr!("circle", 3),
            trace: fx_arr!("trace", 3),
            twirl: fx_arr!("twirl", 2),
            light: fx_arr!("light", 3),
            dirt: fx_arr!("dirt", 2),
            flare: fx_tex("flare").await,
        }
    }

    /// Deterministic pick helper so effects vary without RNG plumbing.
    pub fn star_at(&self, k: usize) -> &Texture2D {
        &self.star[k % self.star.len()]
    }
    pub fn smoke_at(&self, k: usize) -> &Texture2D {
        &self.smoke[k % self.smoke.len()]
    }
    pub fn spark_at(&self, k: usize) -> &Texture2D {
        &self.spark[k % self.spark.len()]
    }
    pub fn light_at(&self, k: usize) -> &Texture2D {
        &self.light[k % self.light.len()]
    }
}

// ---------------------------------------------------------------------------
// post-processing compositor + camera feel
// ---------------------------------------------------------------------------

pub struct Fx {
    pub sprites: FxSprites,

    scene: RenderTarget,
    lo: RenderTarget,
    lo2: RenderTarget,
    /// The composited frame. Owning it means the effect stack can be read back
    /// (debug shots) without depending on the window's default framebuffer.
    out: RenderTarget,

    pub vignette: Texture2D,
    pub iris: Texture2D,
    pub glow: Texture2D,
    pub cloud: Texture2D,
    pub beam: Texture2D,
    grain: [Texture2D; 3],

    sky_candy: Texture2D,
    sky_aqua: Texture2D,
    sky_cosma: Texture2D,
    sky_home: Texture2D,

    pub w: f32,
    pub h: f32,

    /// Horizontal camera shake offset in pixels (read by the overworld draw).
    pub shake_x: f32,
    shake_amp: f32,
    shake_t: f32,
    shake_dur: f32,

    /// Punch-zoom: 1.0 is neutral, >1 pushes in.
    punch: f32,
    punch_v: f32,

    /// Seconds of hit-stop left (drives global slow-mo).
    pub freeze: f32,

    /// Full-screen flash left (0..1) plus its colour.
    flash: f32,
    flash_col: Color,

    /// Per-biome colour grade laid over the finished frame.
    pub grade: Color,

    /// Iris transition: 0 = open, 1 = closed.
    pub iris_close: f32,
    iris_target: f32,

    grain_seed: u32,
    pub t: f32,
}

impl Fx {
    pub async fn new(w: f32, h: f32) -> Self {
        let (wi, hi) = (w as u32, h as u32);
        let scene = render_target(wi, hi);
        scene.texture.set_filter(FilterMode::Linear);
        let lo = render_target(wi / 4, hi / 4);
        lo.texture.set_filter(FilterMode::Linear);
        let lo2 = render_target((wi / 8).max(1), (hi / 8).max(1));
        lo2.texture.set_filter(FilterMode::Linear);
        let out = render_target(wi, hi);
        out.texture.set_filter(FilterMode::Linear);

        Self {
            sprites: FxSprites::load().await,

            scene,
            lo,
            lo2,
            out,

            vignette: bake_vignette(256, 180),
            iris: bake_iris(256),
            glow: bake_glow(128, 2.6),
            cloud: bake_cloud(256),
            beam: bake_beam(64, 256),
            grain: [bake_grain(192, 11), bake_grain(192, 977), bake_grain(192, 5333)],

            // The candy ramp stays saturated all the way down: an earlier pass
            // faded to near-white at the horizon and the lower half of the
            // frame read as an empty bright void.
            sky_candy: bake_gradient(&[
                (0.00, Color::new(1.00, 0.53, 0.78, 1.0)),
                (0.34, Color::new(1.00, 0.63, 0.83, 1.0)),
                (0.66, Color::new(1.00, 0.74, 0.87, 1.0)),
                (0.88, Color::new(1.00, 0.84, 0.90, 1.0)),
                (1.00, Color::new(1.00, 0.90, 0.88, 1.0)),
            ]),
            sky_aqua: bake_gradient(&[
                (0.00, Color::new(0.10, 0.44, 0.70, 1.0)),
                (0.22, Color::new(0.06, 0.32, 0.58, 1.0)),
                (0.55, Color::new(0.035, 0.20, 0.42, 1.0)),
                (0.85, Color::new(0.02, 0.13, 0.31, 1.0)),
                (1.00, Color::new(0.016, 0.10, 0.25, 1.0)),
            ]),
            sky_cosma: bake_gradient(&[
                (0.00, Color::new(0.031, 0.020, 0.071, 1.0)),
                (0.45, Color::new(0.055, 0.031, 0.110, 1.0)),
                (0.80, Color::new(0.086, 0.043, 0.145, 1.0)),
                (1.00, Color::new(0.122, 0.059, 0.180, 1.0)),
            ]),
            sky_home: bake_gradient(&[
                (0.00, Color::new(0.94, 0.80, 0.60, 1.0)),
                (0.40, Color::new(0.98, 0.87, 0.68, 1.0)),
                (0.75, Color::new(0.99, 0.91, 0.76, 1.0)),
                (1.00, Color::new(0.91, 0.75, 0.55, 1.0)),
            ]),

            w,
            h,

            shake_x: 0.0,
            shake_amp: 0.0,
            shake_t: 0.0,
            shake_dur: 0.35,

            punch: 1.0,
            punch_v: 0.0,

            freeze: 0.0,

            flash: 0.0,
            flash_col: WHITE,

            grade: Color::new(0.0, 0.0, 0.0, 0.0),

            iris_close: 0.0,
            iris_target: 0.0,

            grain_seed: 1,
            t: 0.0,
        }
    }

    // -- camera feel --------------------------------------------------------

    /// Kick the camera. `amp` is peak displacement in pixels.
    pub fn shake_for(&mut self, amp: f32, dur: f32) {
        self.shake_dur = dur;
        self.shake_amp = self.shake_amp.max(amp);
        self.shake_t = dur;
    }

    /// Punch-zoom impulse (0.04 = a subtle 4% push, 0.16 = a hard snap).
    pub fn punch(&mut self, amt: f32) {
        self.punch_v += amt;
    }

    pub fn hit_stop(&mut self, secs: f32) {
        self.freeze = self.freeze.max(secs);
    }

    pub fn flash(&mut self, col: Color, a: f32) {
        self.flash = self.flash.max(a);
        self.flash_col = col;
    }

    /// Reopen the star iris (call when arriving somewhere new). The game's fade
    /// veil closes it automatically on every phase change, so only the open
    /// direction needs an explicit call.
    pub fn iris_open(&mut self) {
        self.iris_target = 0.0;
    }

    /// A bright, snappy full-screen accent used for puzzle results.
    pub fn accent(&mut self, good: bool) {
        if good {
            self.flash(Color::new(1.0, 0.98, 0.86, 1.0), 0.30);
            self.punch(0.030);
            self.hit_stop(0.09);
            // lens flare: a real sprite bloom rather than another flat rect
            let s = self.w * 0.9;
            draw_texture_ex(&self.sprites.flare, self.w * 0.5 - s * 0.5,
                self.h * 0.42 - s * 0.5, Color::new(1.0, 0.96, 0.78, 0.5),
                DrawTextureParams {
                    dest_size: Some(vec2(s, s)),
                    rotation: self.t * 0.6,
                    pivot: Some(vec2(s * 0.5, s * 0.5)),
                    ..Default::default()
                });
        } else {
            self.flash(Color::new(0.85, 0.18, 0.28, 1.0), 0.34);
            self.shake_for(9.0, 0.4);
            self.hit_stop(0.14);
        }
    }

    pub fn sky(&self, biome: Biome) -> &Texture2D {
        match biome {
            Biome::Candy => &self.sky_candy,
            Biome::Aquarius => &self.sky_aqua,
            Biome::Cosma => &self.sky_cosma,
            Biome::Home => &self.sky_home,
        }
    }

    // -- per-frame ----------------------------------------------------------

    pub fn tick(&mut self, dt: f32) {
        self.t += dt;

        // shake: linear decay with a sin wobble so it reads as an impact
        if self.shake_t > 0.0 {
            self.shake_t = (self.shake_t - dt).max(0.0);
            let k = if self.shake_dur > 0.0 { self.shake_t / self.shake_dur } else { 0.0 };
            let f = (self.t * 46.0).sin() * k * self.shake_amp;
            self.shake_x = f + (self.t * 31.0).sin() * k * self.shake_amp * 0.4;
            if self.shake_t <= 0.0 {
                self.shake_amp = 0.0;
                self.shake_x = 0.0;
            }
        } else {
            self.shake_x = 0.0;
        }

        // punch-zoom: critically damped spring back to 1.0
        self.punch_v -= (self.punch - 1.0) * 90.0 * dt;
        self.punch_v *= (1.0 - 9.0 * dt).max(0.0);
        self.punch += self.punch_v * dt;
        if (self.punch - 1.0).abs() < 0.0004 && self.punch_v.abs() < 0.0004 {
            self.punch = 1.0;
            self.punch_v = 0.0;
        }

        self.freeze = (self.freeze - dt).max(0.0);
        self.flash = (self.flash - dt * 2.6).max(0.0);

        // iris eases toward its target so transitions have weight
        let d = (self.iris_target - self.iris_close) * (dt * 5.2).min(1.0);
        self.iris_close += d;

        self.grain_seed = self.grain_seed.wrapping_mul(1664525).wrapping_add(1013904223);
    }

    /// Route every subsequent draw call into the offscreen scene buffer.
    pub fn begin(&mut self) {
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.w, self.h));
        // punch-zoom renders a smaller slice of the scene into the buffer,
        // which is a real camera push rather than a post-hoc scale
        let zoom = self.punch.max(0.6);
        let dw = self.w / zoom;
        let dh = self.h / zoom;
        cam.target = vec2(dw / 2.0 + self.shake_x * 0.35, dh / 2.0);
        cam.zoom = vec2(2.0 / dw, -2.0 / dh);
        cam.render_target = Some(self.scene.clone());
        set_camera(&cam);
    }

    fn blit(&self, src: &Texture2D, dst: &RenderTarget, tw: f32, th: f32) {
        let cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, tw, th));
        set_camera(&cam);
        let _ = &dst.render_pass;
        draw_texture_ex(src, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(tw, th)),
            flip_y: FLIP,
            ..Default::default()
        });
    }

    /// Composite the scene buffer into our own frame buffer, then hand that to
    /// the window. `fade` is the game's existing 0..1 intro veil, reused here to
    /// also drive the star iris so every phase change gets a transition free.
    pub fn finish(&mut self, fade: f32) {
        let (w, h) = (self.w, self.h);

        // 1. bloom pyramid: 1/4 then 1/8. The downsamples are minified with a
        //    linear filter, which is a free wide blur.
        let scene_tex = self.scene.texture.clone();
        let (lw, lh) = (w / 4.0, h / 4.0);
        let lo_clone = self.lo.clone();
        self.blit(&scene_tex, &lo_clone, lw, lh);

        let (sw, sh) = (w / 8.0, h / 8.0);
        let lo1 = self.lo.texture.clone();
        let lo2_clone = self.lo2.clone();
        self.blit(&lo1, &lo2_clone, sw, sh);

        // composite the whole stack into our own frame buffer
        {
            let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h));
            cam.render_target = Some(self.out.clone());
            set_camera(&cam);
            self.composite(fade);
        }
        // hand the finished frame to the window
        set_default_camera();
        let out_tex = self.out.texture.clone();
        self.screen_quad(&out_tex, WHITE);
    }

    /// The fully composited frame, for debug captures.
    pub fn capture(&self) -> Image {
        self.out.texture.get_texture_data()
    }

    /// Bloom, colour grade, vignette, grain, flash, iris and veil. Runs with the
    /// camera already pointed at the frame buffer that `finish` hands on.
    fn composite(&self, fade: f32) {
        let (w, h) = (self.w, self.h);

        // the frame itself
        let scene_tex = self.scene.texture.clone();
        self.screen_quad(&scene_tex, WHITE);

        // bloom: two saturating passes per level (tight glow + wide halo). Kept
        // low - without a shader there is no brightness threshold, so a strong
        // mix would haze the whole frame rather than only the highlights. The
        // authored light sprites do the heavy lifting.
        let tight = Color::new(1.0, 1.0, 1.0, 0.115);
        let wide = Color::new(1.0, 1.0, 1.0, 0.075);
        for _ in 0..2 {
            self.screen_quad(&self.lo.texture, tight);
        }
        for _ in 0..2 {
            self.screen_quad(&self.lo2.texture, wide);
        }

        // per-biome colour grade
        if self.grade.a > 0.001 {
            draw_rectangle(0.0, 0.0, w, h, self.grade);
        }

        // vignette
        draw_texture_ex(&self.vignette, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });

        // 6. animated grain, tiled at native resolution and jittered per
        //    ~1/15s so it never resolves into a static pattern
        let gi = ((self.t * 15.0) as usize) % 3;
        let g = &self.grain[gi];
        let gs = 192.0;
        let jx = ((self.grain_seed >> 8) % 192) as f32;
        let jy = ((self.grain_seed >> 20) % 192) as f32;
        let mut y = -jy;
        while y < h {
            let mut x = -jx;
            while x < w {
                draw_texture_ex(g, x, y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(gs, gs)),
                    ..Default::default()
                });
                x += gs;
            }
            y += gs;
        }

        // 7. impact flash
        if self.flash > 0.001 {
            let mut c = self.flash_col;
            c.a = self.flash * self.flash_col.a;
            draw_rectangle(0.0, 0.0, w, h, c);
        }

        // 8. star iris transition (also driven by the game's intro veil)
        let k = self.iris_close.max(1.0 - fade);
        if k > 0.001 {
            let r = (1.0 - k) * (w.max(h) * 0.78) + 14.0;
            let d = r * 2.0;
            let mut pr = DrawTextureParams {
                dest_size: Some(vec2(d, d)),
                rotation: 0.0,
                flip_y: false,
                pivot: None,
                ..Default::default()
            };
            pr.pivot = Some(vec2(d / 2.0, d / 2.0));
            pr.rotation = (1.0 - k) * 1.15;
            draw_texture_ex(&self.iris, w / 2.0 - d / 2.0, h / 2.0 - d / 2.0, WHITE, pr);
        }

        // 9. flat veil on top (kept subtle - the iris carries the transition)
        if fade < 1.0 {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.02, 0.0, 0.05, (1.0 - fade) * 0.45));
        }
    }

    /// Draw a full-screen quad from a buffer texture.
    fn screen_quad(&self, tex: &Texture2D, tint: Color) {
        draw_texture_ex(tex, 0.0, 0.0, tint, DrawTextureParams {
            dest_size: Some(vec2(self.w, self.h)),
            flip_y: FLIP,
            ..Default::default()
        });
    }

    // -- authored light helpers --------------------------------------------

    /// Draw the baked sky ramp for a biome across a screen rect.
    pub fn sky_draw(&self, biome: Biome, x: f32, y: f32, w: f32, h: f32) {
        draw_texture_ex(self.sky(biome), x, y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
    }

    /// Soft light pool stretched to a rect, centred on (x, y). `w`/`h` are the
    /// full screen extents.
    pub fn glow_rect(&self, x: f32, y: f32, w: f32, h: f32, col: Color) {
        draw_texture_ex(&self.glow, x - w / 2.0, y - h / 2.0, col, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
    }

    /// Soft light pool. `radius` is the full diameter on screen.
    pub fn light_pool(&self, x: f32, y: f32, radius: f32, col: Color) {
        draw_texture_ex(&self.glow, x - radius / 2.0, y - radius / 2.0, col,
            DrawTextureParams {
                dest_size: Some(vec2(radius, radius)),
                ..Default::default()
            });
    }

    /// Same, using one of the Kenney light sprites for a more organic falloff.
    pub fn light_sprite(&self, x: f32, y: f32, radius: f32, col: Color, k: usize) {
        let t = self.sprites.light_at(k);
        draw_texture_ex(t, x - radius / 2.0, y - radius / 2.0, col,
            DrawTextureParams {
                dest_size: Some(vec2(radius, radius)),
                ..Default::default()
            });
    }

    /// Soft warm cloud. Draws a shadowed underside under a lit body.
    pub fn cloud(&self, x: f32, y: f32, w: f32, body: Color, shade: Color) {
        let h = w * 0.55;
        draw_texture_ex(&self.cloud, x - w / 2.0, y - h / 2.0 + h * 0.10, shade,
            DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        draw_texture_ex(&self.cloud, x - w / 2.0, y - h / 2.0, body,
            DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
    }

    /// God-ray shaft, optionally leaning.
    pub fn beam_strip(&self, x: f32, top: f32, w: f32, len: f32, col: Color, lean: f32) {
        let pivot = vec2(w / 2.0, 0.0);
        draw_texture_ex(&self.beam, x - w / 2.0, top, col, DrawTextureParams {
            dest_size: Some(vec2(w, len)),
            rotation: lean,
            pivot: Some(pivot),
            ..Default::default()
        });
    }
}
