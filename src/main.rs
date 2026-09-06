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
    let mut game = game::run::Game::new().await;
    loop {
        game.update(get_frame_time());
        game.draw();
        next_frame().await
    }
}
