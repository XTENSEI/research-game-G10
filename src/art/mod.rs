//! Procedural pixel art. All sprites drawn into Image buffers at boot.

use macroquad::prelude::*;
use crate::game::script::Biome;

const W: f32 = 1024.0;
const H: f32 = 720.0;

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
    pub star: Texture2D,
    pub dying_star: Texture2D,
}

/// Draw one pixel-span helper: rect on an image.
fn px(img: &mut Image, x: i32, y: i32, w: i32, h: i32, c: Color) {
    for yy in y..y + h {
        for xx in x..x + w {
            if xx >= 0 && yy >= 0 {
                img.set_pixel(xx as u32, yy as u32, c);
            }
        }
    }
}

const T: Color = Color::new(0.0, 0.0, 0.0, 0.0); // transparent

fn skin(img: &mut Image, c: Color) { px(img, 6, 2, 4, 3, c); }
fn hair(img: &mut Image, c: Color) { px(img, 5, 1, 6, 2, c); }
fn body(img: &mut Image, c: Color) { px(img, 5, 6, 6, 8, c); }
fn legs(img: &mut Image, c: Color) {
    px(img, 6, 14, 2, 6, c);
    px(img, 9, 14, 2, 6, c);
}
fn arms(img: &mut Image, c: Color) {
    px(img, 3, 6, 2, 6, c);
    px(img, 12, 6, 2, 6, c);
}
fn eyes(img: &mut Image, c: Color) {
    px(img, 7, 3, 1, 1, c);
    px(img, 9, 3, 1, 1, c);
}
fn outline(img: &mut Image) {
    let d = Color::from_rgba(20, 12, 24, 255);
    px(img, 4, 1, 8, 1, d);
    px(img, 4, 20, 8, 1, d);
}

fn new_img() -> Image {
    Image::gen_image_color(16, 24, T)
}

fn make_river() -> Texture2D {
    let mut img = new_img();
        hair(&mut img, Color::from_rgba(43, 31, 42, 255));
    skin(&mut img, Color::from_rgba(233, 196, 160, 255));
    eyes(&mut img, Color::from_rgba(30, 20, 30, 255));
    body(&mut img, Color::from_rgba(91, 60, 196, 90)); // bold purple tunic
    arms(&mut img, Color::from_rgba(233, 196, 160, 255));
    legs(&mut img, Color::from_rgba(52, 40, 70, 255));
    outline(&mut img);
    Texture2D::from_image(&img)
}

fn make_jin() -> Texture2D {
    let mut img = new_img();
        let dough = Color::from_rgba(198, 124, 78, 255);
    let icing = Color::from_rgba(240, 240, 245, 255);
    skin(&mut img, dough);
    eyes(&mut img, Color::from_rgba(40, 20, 10, 255));
    // icing trim
    px(&mut img, 6, 5, 4, 1, icing);
    px(&mut img, 6, 8, 1, 1, icing);
    px(&mut img, 9, 8, 1, 1, icing);
    px(&mut img, 7, 4, 1, 1, icing);
    px(&mut img, 8, 4, 1, 1, icing);
    body(&mut img, dough);
    arms(&mut img, dough);
    legs(&mut img, dough);
    px(&mut img, 6, 20, 1, 1, icing);
    px(&mut img, 9, 20, 1, 1, icing);
    outline(&mut img);
    Texture2D::from_image(&img)
}

fn make_kandy() -> Texture2D {
    let mut img = new_img();
        hair(&mut img, Color::from_rgba(255, 105, 180, 255));
    skin(&mut img, Color::from_rgba(255, 220, 200, 255));
    eyes(&mut img, Color::from_rgba(120, 20, 60, 255));
    body(&mut img, Color::from_rgba(255, 20, 147, 255));
    arms(&mut img, Color::from_rgba(255, 220, 200, 255));
    legs(&mut img, Color::from_rgba(180, 0, 90, 255));
    // crown
    px(&mut img, 5, 0, 6, 1, Color::from_rgba(255, 215, 0, 255));
    px(&mut img, 6, 0, 1, 1, Color::from_rgba(255, 215, 0, 255));
    px(&mut img, 8, 0, 1, 1, Color::from_rgba(255, 215, 0, 255));
    // necklace gem
    px(&mut img, 7, 7, 2, 1, Color::from_rgba(0, 255, 200, 255));
    outline(&mut img);
    Texture2D::from_image(&img)
}

fn make_matt() -> Texture2D {
    let mut img = new_img();
        hair(&mut img, Color::from_rgba(80, 50, 30, 255));
    skin(&mut img, Color::from_rgba(240, 200, 170, 255));
    eyes(&mut img, Color::from_rgba(30, 30, 30, 255));
    body(&mut img, Color::from_rgba(120, 90, 60, 255)); // brown attendant vest
    arms(&mut img, Color::from_rgba(240, 200, 170, 255));
    legs(&mut img, Color::from_rgba(60, 45, 30, 255));
    outline(&mut img);
    Texture2D::from_image(&img)
}

fn make_genie() -> Texture2D {
    let mut img = new_img();
    // flame body
    let flame = Color::from_rgba(255, 120, 40, 255);
    px(&mut img, 5, 4, 6, 12, flame);
    px(&mut img, 4, 6, 8, 8, flame);
    px(&mut img, 6, 2, 4, 2, Color::from_rgba(255, 220, 80, 255)); // flame tip
    px(&mut img, 6, 4, 4, 3, Color::from_rgba(255, 220, 80, 255)); // face
    px(&mut img, 7, 5, 1, 1, Color::from_rgba(180, 40, 0, 255));
    px(&mut img, 9, 5, 1, 1, Color::from_rgba(180, 40, 0, 255));
    px(&mut img, 5, 8, 2, 4, Color::from_rgba(255, 180, 60, 255));
    px(&mut img, 10, 8, 2, 4, Color::from_rgba(255, 180, 60, 255));
    Texture2D::from_image(&img)
}

fn make_guard(lolli: Color) -> Texture2D {
    let mut img = new_img();
        let candy_red = Color::from_rgba(230, 60, 60, 255);
    hair(&mut img, Color::from_rgba(250, 250, 250, 255));
    skin(&mut img, Color::from_rgba(250, 220, 190, 255));
    eyes(&mut img, Color::from_rgba(30, 30, 30, 255));
    body(&mut img, candy_red);
    arms(&mut img, candy_red);
    legs(&mut img, Color::from_rgba(150, 30, 30, 255));
    // lollipop weapon
    px(&mut img, 14, 4, 1, 10, Color::from_rgba(240, 240, 240, 255));
    px(&mut img, 13, 2, 3, 3, lolli);
    outline(&mut img);
    Texture2D::from_image(&img)
}

fn make_nadia() -> Texture2D {
    let mut img = new_img();
        hair(&mut img, Color::from_rgba(60, 40, 35, 255));
    skin(&mut img, Color::from_rgba(235, 195, 165, 255));
    eyes(&mut img, Color::from_rgba(40, 40, 60, 255));
    body(&mut img, Color::from_rgba(255, 170, 190, 255)); // pink dress
    px(&mut img, 4, 10, 8, 5, Color::from_rgba(255, 170, 190, 255));
    arms(&mut img, Color::from_rgba(235, 195, 165, 255));
    legs(&mut img, Color::from_rgba(235, 195, 165, 255));
    outline(&mut img);
    Texture2D::from_image(&img)
}

fn make_grandparent() -> Texture2D {
    let mut img = new_img();
        hair(&mut img, Color::from_rgba(230, 230, 230, 255));
    skin(&mut img, Color::from_rgba(220, 185, 155, 255));
    eyes(&mut img, Color::from_rgba(40, 40, 40, 255));
    body(&mut img, Color::from_rgba(110, 90, 140, 255));
    arms(&mut img, Color::from_rgba(220, 185, 155, 255));
    legs(&mut img, Color::from_rgba(70, 60, 90, 255));
    // book in hand
    px(&mut img, 12, 8, 3, 3, Color::from_rgba(140, 40, 40, 255));
    outline(&mut img);
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
    pub fn load() -> Self {
        Self {
            river: make_river(),
            jin: make_jin(),
            kandy: make_kandy(),
            matt: make_matt(),
            genie: make_genie(),
            guard_a: make_guard(Color::from_rgba(255, 100, 160, 255)),
            guard_b: make_guard(Color::from_rgba(100, 160, 255, 255)),
            nadia: make_nadia(),
            grandparent: make_grandparent(),
            star: make_star(false),
            dying_star: make_star(true),
        }
    }

    /// Big dialogue portrait for a speaker name.
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
            _ => return None,
        })
    }
}

/// Vertical gradient backdrop per biome, drawn each frame (cheap, 2 rects + sky fill).
pub fn draw_biome_bg(biome: Biome, t: f32) {
    match biome {
        Biome::Candy => {
            // pinkish sky
            draw_rectangle(0.0, 0.0, W, 420.0, Color::from_rgba(255, 205, 225, 255));
            draw_rectangle(0.0, 0.0, W, 210.0, Color::from_rgba(255, 170, 204, 255));
            draw_rectangle(0.0, 420.0, W, H - 420.0, Color::from_rgba(250, 230, 120, 255)); // lemon land
            // cotton candy clouds (3 layers parallax)
            for i in 0..5 {
                let x = ((t * 8.0 + i as f32 * 240.0) % 1300.0) - 100.0;
                let y = 60.0 + (i % 3) as f32 * 70.0;
                draw_circle(x, y, 34.0, Color::from_rgba(255, 190, 215, 220));
                draw_circle(x + 30.0, y + 8.0, 26.0, Color::from_rgba(255, 205, 225, 200));
                draw_circle(x - 28.0, y + 10.0, 22.0, Color::from_rgba(255, 175, 205, 200));
            }
            // lollipop bushes
            for i in 0..7 {
                let x = 40.0 + i as f32 * 150.0;
                draw_circle(x, 430.0, 26.0, Color::from_rgba(255, 105, 180, 255));
                draw_rectangle(x - 2.0, 430.0, 4.0, 26.0, Color::from_rgba(120, 200, 90, 255));
            }
        }
        Biome::Aquarius => {
            draw_rectangle(0.0, 0.0, W, 360.0, Color::from_rgba(20, 90, 150, 255));
            draw_rectangle(0.0, 360.0, W, H - 360.0, Color::from_rgba(8, 40, 80, 255));
            // light rays
            for i in 0..4 {
                let x = 120.0 + i as f32 * 240.0 + (t * 10.0).sin() * 20.0;
                draw_rectangle(x, 0.0, 60.0, 500.0, Color::from_rgba(180, 220, 255, 28));
            }
            // sandy floor
            draw_rectangle(0.0, 640.0, 1024.0, 80.0, Color::from_rgba(194, 178, 128, 255));
            // corals
            for i in 0..8 {
                let x = 60.0 + i as f32 * 130.0;
                draw_circle(x, 640.0, 18.0, Color::from_rgba(255, 120, 160, 255));
                draw_circle(x + 16.0, 646.0, 12.0, Color::from_rgba(120, 220, 180, 255));
            }
        }
        Biome::Cosma => {
            draw_rectangle(0.0, 0.0, W, H, Color::from_rgba(6, 4, 14, 255));
            // neon stars: deterministic pseudo-random twinkle
            let mut seed = 7u64;
            for _ in 0..90 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let x = (seed >> 33) % 1024;
                let sy = (seed >> 20) % 720;
                let pick = (seed >> 8) % 4;
                let c = match pick {
                    0 => Color::from_rgba(255, 230, 90, 255),
                    1 => Color::from_rgba(90, 160, 255, 255),
                    2 => Color::from_rgba(255, 90, 110, 255),
                    _ => Color::from_rgba(240, 240, 255, 255),
                };
                let tw = 0.5 + 0.5 * (t * 2.0 + (x % 10) as f32).sin();
                draw_rectangle(x as f32, sy as f32, 2.0, 2.0, Color::new(c.r, c.g, c.b, c.a * tw));
            }
            // castle walkway
            draw_rectangle(0.0, 560.0, 1024.0, 24.0, Color::from_rgba(70, 66, 90, 255));
            draw_rectangle(0.0, 584.0, 1024.0, 136.0, Color::from_rgba(40, 38, 55, 255));
        }
    }
}
