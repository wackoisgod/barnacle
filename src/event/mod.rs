use std::io;

mod events;
mod keys;

pub use self::events::CrosstermEvents;
pub use keys::{KeyCode, KeyEvent, KeyModifiers};

pub fn get_events() -> impl EventIterator {

    return CrosstermEvents::new();
}

pub trait EventIterator {
    /// Get the next event
    fn next_event(&mut self) -> io::Result<KeyEvent>;
}
