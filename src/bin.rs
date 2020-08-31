use glfw::{Action, Key, WindowEvent};

mod input;

use wasmlib::input::{InputEvent, KeyState};
use wasmlib::Game;

fn main() {
    env_logger::init();
    let mut game = Game::new();
    let mut event_buf = vec![];
    'app: loop {
        for (_, event) in game.iter_events() {
            match event {
                // If we close the window or press escape, quit the main loop (i.e. quit the application).
                WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                    break 'app
                }

                WindowEvent::Key(k, _, Action::Press, _) => {
                    event_buf.push(InputEvent::KeyboardEvent {
                        code: k.into(),
                        state: KeyState::Pressed,
                        repeated: false,
                    });
                }

                WindowEvent::Key(k, _, Action::Release, _) => {
                    event_buf.push(InputEvent::KeyboardEvent {
                        code: k.into(),
                        state: KeyState::Released,
                        repeated: false,
                    });
                }

                _ => {}
            }
        }

        for e in event_buf.drain(..) {
            game.log_event(e)
        }

        game.tick();
    }
}
