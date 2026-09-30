//! macOS: grabs every input event through the Accessibility API with `rdev`.
//! The grab has no stop call, so unlocking exits the process.

use crate::{domain::Unlock, gotcha::Gotcha};
use anyhow::anyhow;
use rdev::{Event, EventType, Key, grab};
use std::{path::Path, rc::Rc, sync::Mutex};

/// The key presses, in order, that release the grab.
const SECRET: &[Key] = &[Key::Escape];

pub fn run(output_dir: &Path) -> anyhow::Result<()> {
    // The callback is a Fn, so both live behind a Mutex; the camera is not
    // Send, so Rc rather than Arc.
    let gotcha = Rc::new(Mutex::new(Gotcha::new(output_dir)?));
    let unlock = Rc::new(Mutex::new(Unlock::new(SECRET)));

    let callback = move |event: Event| -> Option<Event> {
        let unlocked = match event.event_type {
            EventType::KeyPress(key) => unlock.lock().is_ok_and(|mut unlock| unlock.press(key)),
            _ => false,
        };
        if unlocked {
            println!("Secret entered, exiting");
            std::process::exit(0);
        }
        if let Err(e) = gotcha
            .lock()
            .map_err(|e| anyhow!("{e}"))
            .and_then(|mut gotcha| gotcha.on_input())
        {
            eprintln!("Failed to shoot: {e:?}");
        }
        // Returning None swallows the event, so nothing reaches the desktop.
        None
    };

    println!("Grabbing input");
    grab(callback).map_err(|e| anyhow!("grab failed: {e:?}"))
}
