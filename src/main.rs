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

#[macroquad::main(conf)]
async fn main() {
    let shot = std::env::args().any(|a| a == "--shot");
    let shot_ui = std::env::args().any(|a| a == "--shot-ui");
    let shot_cosma = std::env::args().any(|a| a == "--shot-cosma");
    let shot_aqua = std::env::args().any(|a| a == "--shot-aqua");
    let mut game = game::run::Game::new().await;
    if shot_ui {
        game.demo_dialog();
    } else if shot_cosma {
        game.boot_cosma();
    } else if shot_aqua {
        game.boot_aqua();
    } else if shot {
        game.boot_candy();
    }
    let mut frames: u32 = 0;
    let out = if shot_cosma {
        "shot_cosma.png"
    } else if shot_aqua {
        "shot_aqua.png"
    } else if shot_ui {
        "shot_ui.png"
    } else {
        "shot_candy.png"
    };
    loop {
        game.update(get_frame_time());
        game.draw();
        next_frame().await;
        if shot || shot_ui || shot_cosma || shot_aqua {
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
