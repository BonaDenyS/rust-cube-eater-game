mod game;
mod sprite_loader;

use my_game_engine::init_window;
use game::GameState;

fn main() {
    init_window("Rust Cube Eater Game", 800, 600);
    let mut state = GameState::new();
    state.run();
}
