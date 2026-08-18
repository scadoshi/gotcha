# gotcha

Catches people trying to use your computer while you're away. Grabs all input devices so nothing works, snaps a timestamped webcam photo when someone tries, and unlocks with a secret key.

Working on macOS and Linux.

## Setup

### macOS

The app needs Accessibility API access to grab input devices.

1. Open **System Preferences > Security & Privacy > Privacy > Accessibility**
2. Add your terminal app (e.g. Terminal.app, iTerm2, Alacritty) to the list
3. Run with `cargo run`

### Linux

The app needs access to `/dev/input/*` devices to grab keyboards and mice.

1. Add your user to the `input` group:
   ```
   sudo usermod -aG input $USER
   ```
2. Log out and back in for the group change to take effect
3. Verify with `groups` — you should see `input` in the list
4. Run with `cargo run`

## Usage

1. Run `cargo run`
2. All keyboard and mouse input will be grabbed — the desktop becomes unresponsive
3. Any input triggers a timestamped webcam photo saved to `captures/`
4. Press **Escape** to release all devices and exit
