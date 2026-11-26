//! Screen-space effects: baked atmosphere textures, the Kenney particle sprite
//! kit, GPU bloom/vignette/grain/iris GLSL compositor, and camera feel.
//!
//! Compositor alpha invariant:
//! Every render target is cleared *after* binding its camera, and the final
//! composite shader writes `alpha = 1.0` with blending disabled. This fixes the
//! translucent framebuffer washout caused by stacking alpha-blended overlays
//! onto an un-cleared render target.

use macroquad::prelude::*;
use crate::game::script::Biome;

/// Render-target textures are stored bottom-up, so sampling them back into
/// another render target or the screen uses `flip_y: true`.
const FLIP: bool = true;

// ---------------------------------------------------------------------------
// GLSL ES 1.00 post-processing shaders (OpenGL + WebGL1 compatible)
// ---------------------------------------------------------------------------

const QUAD_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying vec2 uv;
varying vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const BLOOM_EXTRACT_FRAGMENT: &str = r#"#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D Texture;
uniform vec2 u_texel;

void main() {
    vec3 acc = vec3(0.0);
    acc += texture2D(Texture, uv + vec2(-u_texel.x, -u_texel.y)).rgb;
    acc += texture2D(Texture, uv + vec2( u_texel.x, -u_texel.y)).rgb;
    acc += texture2D(Texture, uv + vec2(-u_texel.x,  u_texel.y)).rgb;
    acc += texture2D(Texture, uv + vec2( u_texel.x,  u_texel.y)).rgb;
    acc *= 0.25;
    float lum = dot(acc, vec3(0.299, 0.587, 0.114));
    float w = smoothstep(0.84, 0.99, lum);
    gl_FragColor = vec4(acc * w * 0.85, 1.0);
}
"#;

const BLOOM_BLUR_FRAGMENT: &str = r#"#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D Texture;
uniform vec2 u_texel;

void main() {
    vec3 c = texture2D(Texture, uv).rgb * 0.28;
    c += texture2D(Texture, uv + vec2(-u_texel.x * 1.5, 0.0)).rgb * 0.18;
    c += texture2D(Texture, uv + vec2( u_texel.x * 1.5, 0.0)).rgb * 0.18;
    c += texture2D(Texture, uv + vec2(0.0, -u_texel.y * 1.5)).rgb * 0.18;
    c += texture2D(Texture, uv + vec2(0.0,  u_texel.y * 1.5)).rgb * 0.18;
    gl_FragColor = vec4(c, 1.0);
}
"#;

const COMPOSITE_FRAGMENT: &str = r#"#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D Texture;
uniform sampler2D Bloom1;
uniform sampler2D Bloom2;

uniform vec4 u_grade;
uniform vec4 u_flash;
uniform float u_iris;
uniform float u_fade;
uniform vec2 u_seed;

void main() {
    vec3 base = texture2D(Texture, uv).rgb;
    vec3 b1 = texture2D(Bloom1, uv).rgb;
    vec3 b2 = texture2D(Bloom2, uv).rgb;

    // Soft additive highlight bloom (never lifts cream sheets or dark backgrounds)
    vec3 col = base + b1 * 0.16 + b2 * 0.14;

    // Per-biome subtle color grade
    col = mix(col, u_grade.rgb, u_grade.a);

    // Analytic soft corner vignette (does not touch framebuffer alpha)
    vec2 dp = uv - vec2(0.5, 0.5);
    float dist = length(dp * vec2(u_seed.x, 1.0));
    float vig = smoothstep(0.45, 1.05, dist);
    col = mix(col, vec3(0.015, 0.012, 0.045), vig * 0.36);

    // Subtle film grain jittered over time
    float n = fract(sin(dot(uv * vec2(1024.0, 720.0) + vec2(u_seed.y, u_seed.y * 0.37),
                            vec2(12.9898, 78.233))) * 43758.5453);
    col += (n - 0.5) * 0.016;

    // Impact flash
    col = mix(col, u_flash.rgb, u_flash.a);

    // 8-point star iris + scene fade veil
    float k = max(u_iris, clamp(1.0 - u_fade, 0.0, 1.0));
    if (k > 0.001) {
        vec2 p = dp * vec2(u_seed.x, 1.0);
        float d = length(p);
        float ang = atan(p.y, p.x + 0.00001) + (1.0 - k) * 1.15;
        float r = (1.0 - k) * 1.18 + 0.015;
        float star_edge = r * (0.90 + 0.10 * cos(ang * 8.0));
        float mask = smoothstep(star_edge - 0.015, star_edge + 0.015, d);
        col = mix(col, vec3(0.01, 0.01, 0.03), mask);
        col = mix(col, vec3(0.01, 0.01, 0.03), clamp(1.0 - u_fade, 0.0, 1.0) * 0.55);
    }

    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
"#;

const OPAQUE_COPY_FRAGMENT: &str = r#"#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D Texture;

void main() {
    gl_FragColor = vec4(texture2D(Texture, uv).rgb, 1.0);
}
"#;

// ---------------------------------------------------------------------------
// baked texture helpers
// ---------------------------------------------------------------------------

fn blank(w: u16, h: u16) -> Image {
    Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0))
}

fn finish_tex(img: &Image, linear: bool) -> Texture2D {
    let t = Texture2D::from_image(img);
    t.set_filter(if linear { FilterMode::Linear } else { FilterMode::Nearest });
    t
}

/// Vertical multi-stop gradient stretched smoothly over the screen.
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

/// Radial falloff disc: workhorse for light pools, flares and contact glows.
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

/// Soft metaball cloud sprite for sky & nebulae.
fn bake_cloud(size: u16) -> Texture2D {
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
                let dy = (v - cy) * 1.35;
                f += (-(dx * dx + dy * dy) / (2.0 * sg * sg)).exp();
            }
            let t = ((f - 0.72) / 0.30).clamp(0.0, 1.0);
            let a = t * t * (3.0 - 2.0 * t);
            if a <= 0.004 {
                continue;
            }
            let shade = (1.0 - ((v - 0.40) / 0.60)).clamp(0.0, 1.0);
            let l = 0.82 + 0.18 * shade;
            img.set_pixel(x, y, Color::new(l, l, l, a));
        }
    }
    finish_tex(&img, true)
}

/// God-ray shaft: bright at the top, dissolving downward.
fn bake_beam(w: u16, h: u16) -> Texture2D {
    let mut img = blank(w, h);
    for y in 0..h as u32 {
        let v = y as f32 / h as u32 as f32;
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
    crate::embedded::load_tex(&path, FilterMode::Linear).await
}

/// The CC0 particle sprite kit (Kenney Particle Pack).
pub struct FxSprites {
    pub smoke: [Texture2D; 3],
    pub star: [Texture2D; 4],
    pub magic: [Texture2D; 3],
    pub fire: [Texture2D; 2],
    pub circle: [Texture2D; 3],
    pub light: [Texture2D; 3],
    pub flare: Texture2D,
}

impl FxSprites {
    pub async fn load() -> Self {
        let mut names = Vec::with_capacity(20);
        for i in 1..=3 { names.push(format!("smoke_{}", i)); }
        for i in 1..=4 { names.push(format!("star_{}", i)); }
        for i in 1..=3 { names.push(format!("magic_{}", i)); }
        for i in 1..=2 { names.push(format!("fire_{}", i)); }
        for i in 1..=3 { names.push(format!("circle_{}", i)); }
        for i in 1..=3 { names.push(format!("light_{}", i)); }
        names.push("flare".to_string());

        let futs: Vec<_> = names.iter().map(|n| fx_tex(n)).collect();
        let mut res = crate::game::join_all(futs).await.into_iter();
        let mut take = || res.next().unwrap();
        Self {
            smoke: [take(), take(), take()],
            star: [take(), take(), take(), take()],
            magic: [take(), take(), take()],
            fire: [take(), take()],
            circle: [take(), take(), take()],
            light: [take(), take(), take()],
            flare: take(),
        }
    }

    pub fn star_at(&self, k: usize) -> &Texture2D {
        &self.star[k % self.star.len()]
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
    out: RenderTarget,

    bloom_mat: Material,
    blur_mat: Material,
    composite_mat: Material,
    opaque_mat: Material,

    pub glow: Texture2D,
    pub cloud: Texture2D,
    pub beam: Texture2D,

    sky_kelp: Texture2D,
    sky_aqua: Texture2D,
    sky_slums: Texture2D,
    sky_home: Texture2D,

    pub w: f32,
    pub h: f32,

    /// Horizontal camera shake offset in pixels.
    pub shake_x: f32,
    /// Vertical camera lift in pixels (clears dialogue panels).
    pub lift: f32,
    shake_amp: f32,
    shake_t: f32,
    shake_dur: f32,

    /// Punch-zoom: 1.0 is neutral, >1 pushes in.
    punch: f32,
    punch_v: f32,

    /// Seconds of hit-stop left.
    pub freeze: f32,

    /// Full-screen flash intensity (0..1) + tint.
    flash: f32,
    flash_col: Color,

    /// Per-biome color grade overlaid gently in shader.
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
        let lo = render_target((wi / 4).max(1), (hi / 4).max(1));
        lo.texture.set_filter(FilterMode::Linear);
        let lo2 = render_target((wi / 8).max(1), (hi / 8).max(1));
        lo2.texture.set_filter(FilterMode::Linear);
        let out = render_target(wi, hi);
        out.texture.set_filter(FilterMode::Linear);

        let opaque_pipeline = PipelineParams {
            color_blend: None,
            ..Default::default()
        };

        let bloom_mat = load_material(
            ShaderSource::Glsl {
                vertex: QUAD_VERTEX,
                fragment: BLOOM_EXTRACT_FRAGMENT,
            },
            MaterialParams {
                pipeline_params: opaque_pipeline,
                uniforms: vec![UniformDesc::new("u_texel", UniformType::Float2)],
                ..Default::default()
            },
        )
        .expect("bloom shader");

        let blur_mat = load_material(
            ShaderSource::Glsl {
                vertex: QUAD_VERTEX,
                fragment: BLOOM_BLUR_FRAGMENT,
            },
            MaterialParams {
                pipeline_params: opaque_pipeline,
                uniforms: vec![UniformDesc::new("u_texel", UniformType::Float2)],
                ..Default::default()
            },
        )
        .expect("blur shader");

        let composite_mat = load_material(
            ShaderSource::Glsl {
                vertex: QUAD_VERTEX,
                fragment: COMPOSITE_FRAGMENT,
            },
            MaterialParams {
                pipeline_params: opaque_pipeline,
                textures: vec!["Bloom1".to_string(), "Bloom2".to_string()],
                uniforms: vec![
                    UniformDesc::new("u_grade", UniformType::Float4),
                    UniformDesc::new("u_flash", UniformType::Float4),
                    UniformDesc::new("u_iris", UniformType::Float1),
                    UniformDesc::new("u_fade", UniformType::Float1),
                    UniformDesc::new("u_seed", UniformType::Float2),
                ],
            },
        )
        .expect("composite shader");

        let opaque_mat = load_material(
            ShaderSource::Glsl {
                vertex: QUAD_VERTEX,
                fragment: OPAQUE_COPY_FRAGMENT,
            },
            MaterialParams {
                pipeline_params: opaque_pipeline,
                ..Default::default()
            },
        )
        .expect("opaque copy shader");

        Self {
            sprites: FxSprites::load().await,

            scene,
            lo,
            lo2,
            out,

            bloom_mat,
            blur_mat,
            composite_mat,
            opaque_mat,

            glow: bake_glow(128, 2.6),
            cloud: bake_cloud(256),
            beam: bake_beam(64, 256),

            // Biome 1 (Kelp Forest & Crab Castle): sunlit emerald-teal canopy -> deep jade kelp floor
            sky_kelp: bake_gradient(&[
                (0.00, Color::new(0.11, 0.56, 0.56, 1.0)),
                (0.32, Color::new(0.06, 0.42, 0.45, 1.0)),
                (0.68, Color::new(0.035, 0.28, 0.34, 1.0)),
                (1.00, Color::new(0.020, 0.16, 0.23, 1.0)),
            ]),
            // Biome 2 (Coral City / Aquarius): vibrant lagoon azure -> sapphire reef depth
            sky_aqua: bake_gradient(&[
                (0.00, Color::new(0.10, 0.48, 0.78, 1.0)),
                (0.30, Color::new(0.06, 0.34, 0.62, 1.0)),
                (0.66, Color::new(0.03, 0.20, 0.44, 1.0)),
                (1.00, Color::new(0.015, 0.10, 0.26, 1.0)),
            ]),
            // Biome 3 (Somber Slums / Rainbow Trench): midnight indigo abyssal gradient
            sky_slums: bake_gradient(&[
                (0.00, Color::new(0.018, 0.020, 0.058, 1.0)),
                (0.42, Color::new(0.032, 0.030, 0.086, 1.0)),
                (0.78, Color::new(0.050, 0.038, 0.118, 1.0)),
                (1.00, Color::new(0.072, 0.044, 0.145, 1.0)),
            ]),
            // House / Beach dusk: rich warm amber-mauve cottage walls / starry twilight
            sky_home: bake_gradient(&[
                (0.00, Color::new(0.24, 0.15, 0.24, 1.0)),
                (0.45, Color::new(0.34, 0.21, 0.26, 1.0)),
                (0.82, Color::new(0.46, 0.28, 0.25, 1.0)),
                (1.00, Color::new(0.32, 0.20, 0.21, 1.0)),
            ]),

            w,
            h,

            shake_x: 0.0,
            lift: 0.0,
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

    pub fn shake_for(&mut self, amp: f32, dur: f32) {
        self.shake_dur = dur;
        self.shake_amp = self.shake_amp.max(amp);
        self.shake_t = dur;
    }

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

    pub fn iris_open(&mut self) {
        self.iris_target = 0.0;
    }

    pub fn accent(&mut self, good: bool) {
        if good {
            self.flash(Color::new(1.0, 0.98, 0.86, 1.0), 0.28);
            self.punch(0.030);
            self.hit_stop(0.09);
            let s = self.w * 0.9;
            draw_texture_ex(&self.sprites.flare, self.w * 0.5 - s * 0.5,
                self.h * 0.42 - s * 0.5, Color::new(1.0, 0.96, 0.78, 0.48),
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
            Biome::Kelp => &self.sky_kelp,
            Biome::Coral => &self.sky_aqua,
            Biome::Slums => &self.sky_slums,
            Biome::Home => &self.sky_home,
        }
    }

    // -- per-frame ----------------------------------------------------------

    pub fn tick(&mut self, dt: f32) {
        self.t += dt;

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

        self.punch_v -= (self.punch - 1.0) * 90.0 * dt;
        self.punch_v *= (1.0 - 9.0 * dt).max(0.0);
        self.punch += self.punch_v * dt;
        if (self.punch - 1.0).abs() < 0.0004 && self.punch_v.abs() < 0.0004 {
            self.punch = 1.0;
            self.punch_v = 0.0;
        }

        self.freeze = (self.freeze - dt).max(0.0);
        self.flash = (self.flash - dt * 2.6).max(0.0);

        let d = (self.iris_target - self.iris_close) * (dt * 5.2).min(1.0);
        self.iris_close += d;

        self.grain_seed = self.grain_seed.wrapping_mul(1664525).wrapping_add(1013904223);
    }

    /// Route every subsequent draw call into the offscreen scene buffer.
    pub fn begin(&mut self) {
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.w, self.h));
        let zoom = self.punch.max(0.6);
        let dw = self.w / zoom;
        let dh = self.h / zoom;
        cam.target = vec2(dw / 2.0 + self.shake_x * 0.35, dh / 2.0 + self.lift / zoom);
        cam.zoom = vec2(2.0 / dw, -2.0 / dh);
        cam.render_target = Some(self.scene.clone());
        set_camera(&cam);
        // Clear *after* binding scene so every pixel starts opaque (a = 1.0).
        clear_background(Color::new(0.02, 0.03, 0.07, 1.0));
    }

    /// Re-point the camera at the same scene buffer with no lift and no punch:
    /// UI drawn after the world stays crisp and rock-steady.
    pub fn begin_ui(&self) {
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.w, self.h));
        cam.target = vec2(self.w / 2.0 + self.shake_x * 0.35, self.h / 2.0);
        cam.zoom = vec2(2.0 / self.w, -2.0 / self.h);
        cam.render_target = Some(self.scene.clone());
        set_camera(&cam);
    }

    fn blit_with(&self, src: &Texture2D, dst: &RenderTarget, tw: f32, th: f32, mat: &Material, texel: [f32; 2]) {
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, tw, th));
        cam.render_target = Some(dst.clone());
        set_camera(&cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 1.0));
        mat.set_uniform("u_texel", texel);
        gl_use_material(mat);
        draw_texture_ex(src, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(tw, th)),
            flip_y: FLIP,
            ..Default::default()
        });
        gl_use_default_material();
    }

    /// Composite scene -> bloom pyramid -> self.out -> window with guaranteed alpha = 1.0.
    pub fn finish(&mut self, fade: f32) {
        let (w, h) = (self.w, self.h);
        let scene_tex = self.scene.texture.clone();

        // 1. Thresholded 1/4 bloom extract
        let (lw, lh) = ((w / 4.0).max(1.0), (h / 4.0).max(1.0));
        let lo_clone = self.lo.clone();
        self.blit_with(&scene_tex, &lo_clone, lw, lh, &self.bloom_mat, [1.0 / w, 1.0 / h]);

        // 2. Wide 1/8 bloom blur
        let (sw, sh) = ((w / 8.0).max(1.0), (h / 8.0).max(1.0));
        let lo1 = self.lo.texture.clone();
        let lo2_clone = self.lo2.clone();
        self.blit_with(&lo1, &lo2_clone, sw, sh, &self.blur_mat, [1.0 / lw, 1.0 / lh]);

        // 3. Single opaque shader pass into self.out
        {
            let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h));
            cam.render_target = Some(self.out.clone());
            set_camera(&cam);
            clear_background(Color::new(0.0, 0.0, 0.0, 1.0));

            self.composite_mat.set_texture("Bloom1", self.lo.texture.clone());
            self.composite_mat.set_texture("Bloom2", self.lo2.texture.clone());
            self.composite_mat.set_uniform("u_grade", [
                self.grade.r, self.grade.g, self.grade.b, self.grade.a,
            ]);
            self.composite_mat.set_uniform("u_flash", [
                self.flash_col.r,
                self.flash_col.g,
                self.flash_col.b,
                self.flash * self.flash_col.a,
            ]);
            self.composite_mat.set_uniform("u_iris", self.iris_close);
            self.composite_mat.set_uniform("u_fade", fade);
            self.composite_mat.set_uniform("u_seed", [
                w / h.max(1.0),
                ((self.t * 15.0).floor() % 120.0) * 1.37,
            ]);

            gl_use_material(&self.composite_mat);
            self.screen_quad(&scene_tex, WHITE);
            gl_use_default_material();
        }

        // 4. Hand finished opaque frame to the window
        set_default_camera();
        let out_tex = self.out.texture.clone();
        gl_use_material(&self.opaque_mat);
        self.screen_quad(&out_tex, WHITE);
        gl_use_default_material();
    }

    pub fn capture(&self) -> Image {
        self.out.texture.get_texture_data()
    }

    fn screen_quad(&self, tex: &Texture2D, tint: Color) {
        draw_texture_ex(tex, 0.0, 0.0, tint, DrawTextureParams {
            dest_size: Some(vec2(self.w, self.h)),
            flip_y: FLIP,
            ..Default::default()
        });
    }

    // -- authored light helpers --------------------------------------------

    pub fn sky_draw(&self, biome: Biome, x: f32, y: f32, w: f32, h: f32) {
        draw_texture_ex(self.sky(biome), x, y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
    }

    pub fn glow_rect(&self, x: f32, y: f32, w: f32, h: f32, col: Color) {
        draw_texture_ex(&self.glow, x - w / 2.0, y - h / 2.0, col, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
    }

    pub fn light_pool(&self, x: f32, y: f32, radius: f32, col: Color) {
        draw_texture_ex(&self.glow, x - radius / 2.0, y - radius / 2.0, col,
            DrawTextureParams {
                dest_size: Some(vec2(radius, radius)),
                ..Default::default()
            });
    }

    pub fn cloud(&self, x: f32, y: f32, w: f32, body: Color, shade: Color) {
        let h = w * 0.55;
        draw_texture_ex(&self.cloud, x - w / 2.0, y - h / 2.0 + h * 0.10, shade,
            DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        draw_texture_ex(&self.cloud, x - w / 2.0, y - h / 2.0, body,
            DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
    }

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
