#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod art;
mod embedded;
mod game;

use macroquad::prelude::*;

fn desktop_icon() -> Option<macroquad::miniquad::conf::Icon> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return Some(macroquad::miniquad::conf::Icon {
            small: *include_bytes!("../assets/game/ui/icon_16.rgba"),
            medium: *include_bytes!("../assets/game/ui/icon_32.rgba"),
            big: *include_bytes!("../assets/game/ui/icon_64.rgba"),
        });
    }
    #[cfg(target_arch = "wasm32")]
    None
}

fn conf() -> Conf {
    Conf {
        window_title: "Gift of Education: A Literacy RPG".to_owned(),
        window_width: 1024,
        window_height: 720,
        high_dpi: true,
        icon: desktop_icon(),
        ..Default::default()
    }
}

/// Movement QA: exercises left/right/up/down clamping and click-to-walk on native runs.
fn run_move_qa(game: &mut game::run::Game) {
    let mut fails = 0u32;
    let mut check = |name: &str, ok: bool, detail: String| {
        if !ok {
            fails += 1;
        }
        println!("[move] {} {} {}", if ok { "PASS" } else { "FAIL" }, name, detail);
    };
    let hold = |game: &mut game::run::Game, dx: f32, dy: f32, frames: u32| {
        game.qa_dir(dx, dy);
        for _ in 0..frames {
            game.update(1.0 / 60.0);
            game.draw();
        }
        game.qa_dir(0.0, 0.0);
    };

    game.qa_boot("kelp");
    game.update(1.0 / 60.0);
    game.draw();
    let start = game.qa_player();
    let len = game.qa_world_len();
    println!("[move] biome={} start={:?} world_len={}", game.qa_biome_name(), start, len);

    hold(game, 1.0, 0.0, 60);
    let right = game.qa_player();
    check("walk right", right.0 > start.0 + 40.0, format!("dx={:+.1} {}", right.0 - start.0, game.qa_pose()));
    check("face right", game.qa_pose().contains("facing=+1"), game.qa_pose());

    hold(game, -1.0, 0.0, 60);
    let left = game.qa_player();
    check("walk left", left.0 < right.0 - 40.0, format!("dx={:+.1}", left.0 - right.0));
    check("face left", game.qa_pose().contains("facing=-1"), game.qa_pose());

    hold(game, 0.0, -1.0, 90);
    let up = game.qa_player();
    check("walk up clamps at 538", (up.1 - 538.0).abs() < 0.8, format!("y={:.1}", up.1));
    hold(game, 0.0, 1.0, 120);
    let down = game.qa_player();
    check("walk down clamps at 588", (down.1 - 588.0).abs() < 0.8, format!("y={:.1}", down.1));

    game.qa_teleport(60.0);
    hold(game, -1.0, 0.0, 60);
    check("left edge stops at 40", (game.qa_player().0 - 40.0).abs() < 0.8, format!("x={:.1}", game.qa_player().0));
    game.qa_teleport(len - 60.0);
    hold(game, 1.0, 0.0, 60);
    check("right edge stops at len-40", (game.qa_player().0 - (len - 40.0)).abs() < 0.8, format!("x={:.1}", game.qa_player().0));

    game.qa_teleport(400.0);
    game.qa_click(760.0);
    for _ in 0..240 {
        game.update(1.0 / 60.0);
        game.draw();
    }
    let walked = game.qa_player().0;
    check(
        "click-to-walk arrives",
        (walked - 760.0).abs() < 14.0 && game.qa_click_target().is_none(),
        format!("x={:.1} target={:?}", walked, game.qa_click_target()),
    );

    println!("[move] {} failure(s)", fails);
    if fails > 0 {
        std::process::exit(1);
    }
}

#[macroquad::main(conf)]
async fn main() {
    let shot = std::env::args().any(|a| a == "--shot");
    let shot_ui = std::env::args().any(|a| a == "--shot-ui");
    let shot_cosma = std::env::args().any(|a| a == "--shot-cosma");
    let shot_aqua = std::env::args().any(|a| a == "--shot-aqua");
    // --shot=<screen> boots any phase for QA (see Game::qa_boot).
    let shot_scene = std::env::args().find_map(|a| a.strip_prefix("--shot=").map(str::to_string));
    // --seq=<screen> samples a run of frames and prints the player's pose, so
    // animation work can be checked as a sequence instead of a still.
    let seq = std::env::args().find_map(|a| a.strip_prefix("--seq=").map(str::to_string));
    // --move drives the movement branches and checks every direction and edge.
    let move_qa = std::env::args().any(|a| a == "--move");
    let with_bot = std::env::args().any(|a| a == "--bot");

    let mut game = game::run::Game::new().await;
    if std::env::args().any(|a| a == "--autoplay") {
        game.autoplay();
    }

    if let Some(name) = seq.as_ref() {
        game.qa_boot(name);
        if with_bot {
            game.qa_enable_bot();
        }
    } else if let Some(name) = &shot_scene {
        game.qa_boot(name);
    } else if shot_ui {
        game.demo_dialog();
    } else if shot_cosma {
        game.boot_cosma();
    } else if shot_aqua {
        game.boot_aqua();
    } else if shot {
        game.boot_candy();
    }
    if move_qa {
        run_move_qa(&mut game);
        return;
    }

    let mut frames: u32 = 0;
    let mut seq_i: u32 = 0;
    let out = if let Some(name) = &shot_scene {
        Box::leak(format!("shot_{}.png", name).into_boxed_str()) as &str
    } else if shot_cosma {
        "shot_cosma.png"
    } else if shot_aqua {
        "shot_aqua.png"
    } else if shot_ui {
        "shot_ui.png"
    } else {
        "shot_kelp.png"
    };

    loop {
        if seq.is_some() {
            game.update(1.0 / 60.0);
        } else {
            game.update(get_frame_time());
        }
        game.draw();
        next_frame().await;

        if let Some(name) = seq.as_deref() {
            frames += 1;
            let (first, gap, cap) = if with_bot { (30u32, 40u32, 20u32) } else { (40u32, 6u32, 14u32) };
            if frames >= first && (frames - first) % gap == 0 && seq_i < cap {
                let img = game.shot_image();
                img.export_png(&format!("shot_seq_{}_{:02}.png", name, seq_i));
                println!("[seq] {:02} {} | {}", seq_i, game.qa_pose(), game.qa_state());
                seq_i += 1;
            }
            if seq_i >= cap {
                println!("[seq] done {}", name);
                std::process::exit(0);
            }
            continue;
        }
        if shot || shot_ui || shot_cosma || shot_aqua || shot_scene.is_some() {
            frames += 1;
            if frames == 45 {
                let img = game.shot_image();
                img.export_png(out);
                std::process::exit(0);
            }
        }
    }
}
