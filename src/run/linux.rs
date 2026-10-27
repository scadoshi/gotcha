//! Linux: grabs the keyboards and mice under `/dev/input` with `evdev` and
//! polls them. Only those devices, since grabbing everything takes Bluetooth
//! and network controllers with it.

use std::os::fd::AsFd;
use std::path::Path;

use anyhow::Context;
use evdev::{Device, EventSummary, EventType, KeyCode, RelativeAxisCode};
use nix::poll::{PollFd, PollFlags, PollTimeout};

use crate::domain::Unlock;
use crate::gotcha::Gotcha;

/// An evdev key event's value on press; release is 0 and repeat is 2.
const PRESSED: i32 = 1;

/// A keyboard repeats keys and has letters, enter and space; a mouse reports
/// relative movement on both axes.
trait Identify {
    fn is_probably_keyboard(&self) -> bool;
    fn is_probably_mouse(&self) -> bool;
}

impl Identify for Device {
    fn is_probably_keyboard(&self) -> bool {
        self.supported_events().contains(EventType::REPEAT)
            && self.supported_keys().is_some_and(|keys| {
                keys.contains(KeyCode::KEY_A)
                    && keys.contains(KeyCode::KEY_ENTER)
                    && keys.contains(KeyCode::KEY_SPACE)
            })
    }

    fn is_probably_mouse(&self) -> bool {
        self.supported_relative_axes().is_some_and(|axes| {
            axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
        })
    }
}

/// The key for one character of the secret: a letter (lowercase) or a digit.
fn key_for(c: char) -> Option<KeyCode> {
    Some(match c {
        'a' => KeyCode::KEY_A,
        'b' => KeyCode::KEY_B,
        'c' => KeyCode::KEY_C,
        'd' => KeyCode::KEY_D,
        'e' => KeyCode::KEY_E,
        'f' => KeyCode::KEY_F,
        'g' => KeyCode::KEY_G,
        'h' => KeyCode::KEY_H,
        'i' => KeyCode::KEY_I,
        'j' => KeyCode::KEY_J,
        'k' => KeyCode::KEY_K,
        'l' => KeyCode::KEY_L,
        'm' => KeyCode::KEY_M,
        'n' => KeyCode::KEY_N,
        'o' => KeyCode::KEY_O,
        'p' => KeyCode::KEY_P,
        'q' => KeyCode::KEY_Q,
        'r' => KeyCode::KEY_R,
        's' => KeyCode::KEY_S,
        't' => KeyCode::KEY_T,
        'u' => KeyCode::KEY_U,
        'v' => KeyCode::KEY_V,
        'w' => KeyCode::KEY_W,
        'x' => KeyCode::KEY_X,
        'y' => KeyCode::KEY_Y,
        'z' => KeyCode::KEY_Z,
        '0' => KeyCode::KEY_0,
        '1' => KeyCode::KEY_1,
        '2' => KeyCode::KEY_2,
        '3' => KeyCode::KEY_3,
        '4' => KeyCode::KEY_4,
        '5' => KeyCode::KEY_5,
        '6' => KeyCode::KEY_6,
        '7' => KeyCode::KEY_7,
        '8' => KeyCode::KEY_8,
        '9' => KeyCode::KEY_9,
        _ => return None,
    })
}

/// Grabs the keyboards and mice and shoots on their input until `secret` is
/// typed, then releases them.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the CLI's own progress and error lines"
)]
pub(crate) fn run(output_dir: &Path, secret: &str) -> anyhow::Result<()> {
    let keys: Vec<KeyCode> = secret
        .chars()
        .map(key_for)
        .collect::<Option<_>>()
        .context("the secret holds a character with no key")?;
    let mut devices: Vec<Device> = evdev::enumerate()
        .map(|(_, device)| device)
        .filter(|device| device.is_probably_mouse() || device.is_probably_keyboard())
        .collect();
    anyhow::ensure!(
        !devices.is_empty(),
        "no keyboard or mouse could be opened; is this user in the input group?"
    );
    let mut gotcha = Gotcha::new(output_dir)?;
    let mut unlock = Unlock::new(keys);
    for device in &mut devices {
        device.grab()?;
    }
    println!("Grabbed {} devices", devices.len());

    'grabbed: loop {
        // Built each round because a PollFd borrows its device.
        let mut poll_fds: Vec<PollFd<'_>> = devices
            .iter()
            .map(|device| PollFd::new(device.as_fd(), PollFlags::POLLIN))
            .collect();
        nix::poll::poll(&mut poll_fds, PollTimeout::NONE)?;
        let ready: Vec<usize> = poll_fds
            .iter()
            .enumerate()
            .filter(|(_, fd)| {
                fd.revents()
                    .is_some_and(|flags| flags.contains(PollFlags::POLLIN))
            })
            .map(|(i, _)| i)
            .collect();
        drop(poll_fds);

        for i in ready {
            let Some(device) = devices.get_mut(i) else {
                continue;
            };
            for event in device.fetch_events()? {
                if let EventSummary::Key(_, key, PRESSED) = event.destructure()
                    && unlock.press(key)
                {
                    println!("Secret entered, releasing devices");
                    break 'grabbed;
                }
                if let Err(e) = gotcha.on_input() {
                    eprintln!("Failed to shoot: {e:?}");
                }
            }
        }
    }

    for device in &mut devices {
        device.ungrab()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_letter_and_digit_has_its_own_key() {
        let chars = ('a'..='z').chain('0'..='9');
        let keys: HashSet<KeyCode> = chars.clone().filter_map(key_for).collect();
        assert_eq!(
            keys.len(),
            chars.count(),
            "a character is missing or shares a key"
        );
    }

    #[test]
    fn the_default_secret_maps_to_t_e_s_t() {
        let keys: Option<Vec<KeyCode>> =
            crate::domain::DEFAULT_SECRET.chars().map(key_for).collect();
        assert_eq!(
            keys,
            Some(vec![
                KeyCode::KEY_T,
                KeyCode::KEY_E,
                KeyCode::KEY_S,
                KeyCode::KEY_T
            ])
        );
    }
}
