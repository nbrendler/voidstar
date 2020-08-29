use std::thread::sleep;
use std::time::Duration;

use wasmlib::Game;

fn main() {
    let mut game = Game::new();

    loop {
        game.tick();
        sleep(Duration::from_millis(16));
    }
}
