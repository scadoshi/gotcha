//! Linux: grabs the keyboards and mice under `/dev/input` with `evdev` and
//! polls them. Only those devices, since grabbing everything takes Bluetooth
//! and network controllers with it.

use crate::{domain::Unlock, gotcha::Gotcha};
use evdev::{Device, EventSummary, EventType, KeyCode, RelativeAxisCode};
use nix::poll::{PollFd, PollFlags, PollTimeout};
use std::{os::fd::AsFd, path::Path};

/// The key presses, in order, that release the grab.
const SECRET: &[KeyCode] = &[KeyCode::KEY_ESC];

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

pub fn run(output_dir: &Path) -> anyhow::Result<()> {
    let mut gotcha = Gotcha::new(output_dir)?;
    let mut unlock = Unlock::new(SECRET);

    let mut devices: Vec<Device> = evdev::enumerate()
        .map(|(_, device)| device)
        .filter(|device| device.is_probably_mouse() || device.is_probably_keyboard())
        .collect();
    for device in &mut devices {
        device.grab()?;
    }
    println!("Grabbed {} devices", devices.len());

    'grabbed: loop {
        // Built each round because a PollFd borrows its device.
        let mut poll_fds: Vec<PollFd> = devices
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
                // A key event's value is 1 on press, 0 on release, 2 on repeat.
                if let EventSummary::Key(_, key, 1) = event.destructure()
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
