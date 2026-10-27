//! macOS: grabs every input event through the Accessibility API with `rdev`.
//! The grab has no stop call, so unlocking exits the process.

use std::path::Path;
use std::rc::Rc;
use std::sync::Mutex;

use anyhow::{Context, anyhow};
use rdev::{Event, EventType, Key, grab};

use crate::domain::Unlock;
use crate::gotcha::Gotcha;

/// The key for one character of the secret: a letter (lowercase) or a digit.
fn key_for(c: char) -> Option<Key> {
    Some(match c {
        'a' => Key::KeyA,
        'b' => Key::KeyB,
        'c' => Key::KeyC,
        'd' => Key::KeyD,
        'e' => Key::KeyE,
        'f' => Key::KeyF,
        'g' => Key::KeyG,
        'h' => Key::KeyH,
        'i' => Key::KeyI,
        'j' => Key::KeyJ,
        'k' => Key::KeyK,
        'l' => Key::KeyL,
        'm' => Key::KeyM,
        'n' => Key::KeyN,
        'o' => Key::KeyO,
        'p' => Key::KeyP,
        'q' => Key::KeyQ,
        'r' => Key::KeyR,
        's' => Key::KeyS,
        't' => Key::KeyT,
        'u' => Key::KeyU,
        'v' => Key::KeyV,
        'w' => Key::KeyW,
        'x' => Key::KeyX,
        'y' => Key::KeyY,
        'z' => Key::KeyZ,
        '0' => Key::Num0,
        '1' => Key::Num1,
        '2' => Key::Num2,
        '3' => Key::Num3,
        '4' => Key::Num4,
        '5' => Key::Num5,
        '6' => Key::Num6,
        '7' => Key::Num7,
        '8' => Key::Num8,
        '9' => Key::Num9,
        _ => return None,
    })
}

pub(crate) fn run(output_dir: &Path, secret: &str) -> anyhow::Result<()> {
    let keys: Vec<Key> = secret
        .chars()
        .map(key_for)
        .collect::<Option<_>>()
        .context("the secret holds a character with no key")?;
    // The callback is a Fn, so both live behind a Mutex; the camera is not
    // Send, so Rc rather than Arc.
    let gotcha = Rc::new(Mutex::new(Gotcha::new(output_dir)?));
    let unlock = Rc::new(Mutex::new(Unlock::new(keys)));

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
