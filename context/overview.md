# Gotcha

Catches whoever touches the computer while you are away. Grabs the keyboard and mouse so the desktop stops responding, takes a webcam photo on any input, and releases when the secret key sequence is typed.

## Shape

```
src/
  main.rs        picks the platform's run() and names the output directory
  domain.rs      Unlock (secret sequence), Shutter (one shot per interval),
                 file_name. No I/O, all the tests
  gotcha.rs      Gotcha: camera + Shutter + output dir; on_input() shoots and saves
  camera.rs      nokhwa: opens the first camera, warms it up, shoots RGB frames
  run/macos.rs   CGEventTap on its own thread that drops every event; camera on main
  run/linux.rs   evdev grab of keyboards and mice only, nix poll loop; ungrabs on unlock
```

The platform modules own the input devices and the event loop, and hand each key press to `Unlock` and each event to `Gotcha::on_input`. Everything that decides something lives in `domain.rs`, where it takes the key or the time as an argument, so the tests never touch a camera or a device.

The secret comes from `--password <secret>` or `--pw <secret>` and defaults to `test` (`domain::secret_from_args`). Each platform's `key_for` turns its letters and digits into that platform's key type. `Unlock` is generic over the key so neither platform maps into a shared enum.

## Platform notes

macOS: a `core-graphics` event tap at the HID level that answers `Drop` for every keyboard and mouse event, which hands macOS a null event. The run loop runs in quarter-second slices and turns the tap back on after each, since macOS disables a tap whose callback was slow. The tap runs on its own thread and run loop and only sends the main thread a message per input, so a slow frame never stalls input. The camera stays on the main thread: `AVFoundation` raises an Objective-C exception configuring some cameras from any other thread, which aborts the process. Unlocking stops the run loop. Needs Accessibility access for the terminal; without it macOS refuses the tap and the program says so. The secret matches keys by their position on a US keyboard.

Linux: grabbing every evdev device takes Bluetooth and network controllers with it and disconnects them, so the Linux path picks keyboards and mice itself with `evdev`.

- It grabs only devices that look like a keyboard (repeat events plus A, Enter and Space) or a mouse (relative X and Y).
- The user has to be in the `input` group.
- It rebuilds the `PollFd` list every round, because each one borrows its device.

## Checks

```bash
cargo +nightly fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

`cargo fmt` needs nightly because rustfmt.toml uses unstable options.

The Linux module only compiles on Linux; `cargo check --target x86_64-unknown-linux-gnu` from a Mac fails in mozjpeg's build script before it reaches this crate.
