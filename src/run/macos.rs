//! macOS: an event tap that drops every keyboard and mouse event until the
//! secret is typed. The camera runs on its own thread, so the tap never waits on
//! a frame.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, anyhow};
use core_foundation::runloop::{CFRunLoop, kCFRunLoopDefaultMode};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult, EventField,
};

use crate::domain::Unlock;
use crate::gotcha::Gotcha;

/// How long the run loop runs between checks. Each check turns the tap back on,
/// since macOS disables a tap whose callback was slow.
const SLICE: Duration = Duration::from_millis(250);

/// Every keyboard and mouse event the tap drops.
const GRABBED: [CGEventType; 14] = [
    CGEventType::KeyDown,
    CGEventType::KeyUp,
    CGEventType::FlagsChanged,
    CGEventType::LeftMouseDown,
    CGEventType::LeftMouseUp,
    CGEventType::RightMouseDown,
    CGEventType::RightMouseUp,
    CGEventType::OtherMouseDown,
    CGEventType::OtherMouseUp,
    CGEventType::MouseMoved,
    CGEventType::LeftMouseDragged,
    CGEventType::RightMouseDragged,
    CGEventType::OtherMouseDragged,
    CGEventType::ScrollWheel,
];

/// The virtual keycode for one character of the secret: a letter (lowercase) or a
/// digit, by its position on a US (ANSI) keyboard.
fn key_for(c: char) -> Option<u16> {
    Some(match c {
        'a' => 0x00,
        's' => 0x01,
        'd' => 0x02,
        'f' => 0x03,
        'h' => 0x04,
        'g' => 0x05,
        'z' => 0x06,
        'x' => 0x07,
        'c' => 0x08,
        'v' => 0x09,
        'b' => 0x0b,
        'q' => 0x0c,
        'w' => 0x0d,
        'e' => 0x0e,
        'r' => 0x0f,
        'y' => 0x10,
        't' => 0x11,
        'o' => 0x1f,
        'u' => 0x20,
        'i' => 0x22,
        'p' => 0x23,
        'l' => 0x25,
        'j' => 0x26,
        'k' => 0x28,
        'n' => 0x2d,
        'm' => 0x2e,
        '1' => 0x12,
        '2' => 0x13,
        '3' => 0x14,
        '4' => 0x15,
        '6' => 0x16,
        '5' => 0x17,
        '9' => 0x19,
        '7' => 0x1a,
        '8' => 0x1c,
        '0' => 0x1d,
        _ => return None,
    })
}

/// Drops every keyboard and mouse event and shoots on each one until `secret` is
/// typed.
///
/// # Errors
///
/// When the secret holds a character with no key, the camera does not open, or
/// macOS refuses the event tap, which it does without Accessibility access.
#[expect(clippy::print_stdout, reason = "the CLI's own progress lines")]
pub(crate) fn run(output_dir: &Path, secret: &str) -> anyhow::Result<()> {
    let keys: Vec<u16> = secret
        .chars()
        .map(key_for)
        .collect::<Option<_>>()
        .context("the secret holds a character with no key")?;
    let unlock = Mutex::new(Unlock::new(keys));
    let unlocked = Arc::new(AtomicBool::new(false));
    let (shots, shooter) = spawn_shooter(output_dir.to_path_buf())?;

    let seen_unlock = Arc::clone(&unlocked);
    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        GRABBED.to_vec(),
        move |_, kind, event| {
            if matches!(kind, CGEventType::KeyDown) {
                let pressed = u16::try_from(
                    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE),
                );
                let unlocks = pressed.is_ok_and(|key| {
                    unlock.lock().is_ok_and(|mut unlock| unlock.press(key))
                });
                if unlocks {
                    seen_unlock.store(true, Ordering::Relaxed);
                    CFRunLoop::get_current().stop();
                    return CallbackResult::Drop;
                }
            }
            // The camera thread may have stopped; the grab holds either way.
            shots.send(()).ok();
            CallbackResult::Drop
        },
    )
    .map_err(|()| {
        anyhow!("macOS refused the event tap; give your terminal Accessibility access in System Settings > Privacy & Security")
    })?;

    let source = tap
        .mach_port()
        .create_runloop_source(0)
        .map_err(|()| anyhow!("could not attach the event tap to the run loop"))?;
    // SAFETY: reads an immutable CoreFoundation constant.
    let mode = unsafe { kCFRunLoopDefaultMode };
    let run_loop = CFRunLoop::get_current();
    run_loop.add_source(&source, mode);
    tap.enable();
    println!("Grabbing input");

    while !unlocked.load(Ordering::Relaxed) {
        CFRunLoop::run_in_mode(mode, SLICE, false);
        tap.enable();
    }
    println!("Secret entered, releasing input");

    drop(tap);
    shooter
        .join()
        .map_err(|_| anyhow!("the camera thread panicked"))?;
    Ok(())
}

/// Starts the camera thread and waits until the camera is warm. Each `()` sent
/// asks for a shot; dropping every sender ends the thread.
#[expect(clippy::print_stderr, reason = "the CLI's own error lines")]
fn spawn_shooter(output_dir: PathBuf) -> anyhow::Result<(mpsc::Sender<()>, JoinHandle<()>)> {
    let (shots, requests) = mpsc::channel::<()>();
    let (ready, started) = mpsc::sync_channel::<anyhow::Result<()>>(1);
    let shooter = thread::spawn(move || {
        let mut gotcha = match Gotcha::new(&output_dir) {
            Ok(gotcha) => gotcha,
            Err(error) => {
                ready.send(Err(error)).ok();
                return;
            }
        };
        ready.send(Ok(())).ok();
        for () in requests {
            if let Err(error) = gotcha.on_input() {
                eprintln!("Failed to shoot: {error:?}");
            }
        }
    });
    started
        .recv()
        .context("the camera thread stopped before it started")??;
    Ok((shots, shooter))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_letter_and_digit_has_its_own_key() {
        let chars = ('a'..='z').chain('0'..='9');
        let keys: HashSet<u16> = chars.clone().filter_map(key_for).collect();
        assert_eq!(
            keys.len(),
            chars.count(),
            "a character is missing or shares a key"
        );
    }

    #[test]
    fn the_default_secret_maps_to_t_e_s_t() {
        let keys: Option<Vec<u16>> = crate::domain::DEFAULT_SECRET.chars().map(key_for).collect();
        assert_eq!(keys, Some(vec![0x11, 0x0e, 0x01, 0x11]));
    }
}
