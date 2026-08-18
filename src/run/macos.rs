use anyhow::anyhow;
use rdev::{Event, EventType, Key, grab};

use crate::state::GotchaState;
use std::{rc::Rc, sync::Mutex};

pub fn run() -> anyhow::Result<()> {
    let state = Rc::new(Mutex::new(GotchaState::new()?));
    println!("GotchaState initialized");

    let clone = state.clone();
    let callback = move |event: Event| -> Option<Event> {
        match event.event_type {
            EventType::KeyPress(Key::Escape) => {
                println!("Secret key pressed");
                println!("Exiting process");
                std::process::exit(0);
            }
            _ => {
                if let Err(e) = clone
                    .lock()
                    .map_err(|e| anyhow!("{}", e))
                    .and_then(|mut s| s.maybe_shoot_and_save().map_err(|e| anyhow!("{}", e)))
                {
                    eprintln!("Failed to shoot: {:?}", e);
                }
                None
            }
        }
    };

    println!("Initiating input grab");
    if let Err(e) = grab(callback) {
        eprintln!("Grab failed: {:?}", e);
    }

    Ok(())
}
