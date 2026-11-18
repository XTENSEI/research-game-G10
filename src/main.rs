mod audio;
mod art;
mod game;

use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: "Gift of Education: A Literacy RPG".to_owned(),
        window_width: 1024,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

/// Movement QA: hold each direction through the real movement branch and check
/// travel, facing, clamping and click-to-walk. Prints one PASS/FAIL per check.
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

    game.qa_boot("candy");
    game.update(1.0 / 60.0);
    game.draw();
    let start = game.qa_player();
    let len = game.qa_world_len();
    println!("[move] biome={} start={:?} world_len={}", game.qa_biome_name(), start, len);

    hold(game, 1.0, 0.0, 60);
    let right = game.qa_player();
    check("walk right", right.0 > start.0 + 40.0, format!("dx={:+.1} {} ", right.0 - start.0, game.qa_pose()));
    check("face right", game.qa_pose().contains("facing=+1"), game.qa_pose());

    hold(game, -1.0, 0.0, 60);
    let left = game.qa_player();
    check("walk left", left.0 < right.0 - 40.0, format!("dx={:+.1}", left.0 - right.0));
    check("face left", game.qa_pose().contains("facing=-1"), game.qa_pose());

    hold(game, 0.0, -1.0, 90);
    let up = game.qa_player();
    check("walk up clamps at 400", (up.1 - 400.0).abs() < 0.6, format!("y={:.1}", up.1));
    hold(game, 0.0, 1.0, 240);
    let down = game.qa_player();
    check("walk down clamps at 560", (down.1 - 560.0).abs() < 0.6, format!("y={:.1}", down.1));

    game.qa_teleport(80.0);
    hold(game, -1.0, 0.0, 60);
    check("left edge stops at 60", (game.qa_player().0 - 60.0).abs() < 0.6, format!("x={:.1}", game.qa_player().0));
    game.qa_teleport(len - 80.0);
    hold(game, 1.0, 0.0, 60);
    check("right edge stops at len-60", (game.qa_player().0 - (len - 60.0)).abs() < 0.6,
        format!("x={:.1}", game.qa_player().0));

    game.qa_teleport(400.0);
    game.qa_click(760.0);
    for _ in 0..240 {
        game.update(1.0 / 60.0);
        game.draw();
    }
    let walked = game.qa_player().0;
    check("click-to-walk arrives", (walked - 760.0).abs() < 12.0 && game.qa_click_target().is_none(),
        format!("x={:.1} target={:?}", walked, game.qa_click_target()));

    game.qa_boot("swim");
    for _ in 0..60 {
        game.update(1.0 / 60.0);
        game.draw();
    }
    let swim_a = game.qa_player().0;
    hold(game, 1.0, 0.0, 60);
    check("swim moves", game.qa_player().0 > swim_a + 40.0,
        format!("dx={:+.1} {}", game.qa_player().0 - swim_a, game.qa_pose()));

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
    let mut game = game::run::Game::new().await;
    if std::env::args().any(|a| a == "--autoplay") {
        game.autoplay();
    }
    if let Some(name) = seq.as_ref() {
        game.qa_boot(name);
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
        "shot_candy.png"
    };
    loop {
        // The sequence mode steps a fixed 1/60 s so the samples are comparable
        // between runs; real play keeps the frame clock.
        if seq.is_some() {
            game.update(1.0 / 60.0);
        } else {
            game.update(get_frame_time());
        }
        game.draw();
        next_frame().await;
        if let Some(name) = seq.as_deref() {
            frames += 1;
            if frames >= 40 && (frames - 40) % 6 == 0 && seq_i < 14 {
                let img = game.shot_image();
                img.export_png(&format!("shot_seq_{}_{:02}.png", name, seq_i));
                println!("[seq] {:02} {}", seq_i, game.qa_pose());
                seq_i += 1;
            }
            if seq_i >= 14 {
                println!("[seq] done {}", name);
                std::process::exit(0);
            }
            continue;
        }
        if shot || shot_ui || shot_cosma || shot_aqua || shot_scene.is_some() {
            frames += 1;
            if frames == 44 {
                clear_background(Color::new(0.95, 0.95, 0.95, 1.0));
                next_frame().await;
            }
            if frames == 45 {
                if std::env::var("SHOT_NO_READBACK").is_err() {
                    // Read back the effects layer's own frame buffer rather than
                    // the window's default one: after the offscreen passes the
                    // default framebuffer readback comes back black on some
                    // drivers, which silently produced empty QA shots.
                    let img = if std::env::var("SHOT_WINDOW").is_ok() {
                        get_screen_data()
                    } else {
                        game.shot_image()
                    };
                    img.export_png(out);
                }
                std::process::exit(0);
            }
        }
    }
}
