# Gotcha

Catches whoever touches the computer while you are away. Grabs the keyboard and
mouse so the desktop stops responding, takes a webcam photo on any input, and
releases when the secret key sequence is typed.

## Shape

```
src/
  main.rs        picks the platform's run() and names the output directory
  domain.rs      Unlock (secret sequence), Shutter (one shot per interval),
                 file_name. No I/O, all the tests
  gotcha.rs      Gotcha: camera + Shutter + output dir; on_input() shoots and saves
  camera.rs      nokhwa: opens the first camera, warms it up, shoots RGB frames
  run/macos.rs   rdev grab through the Accessibility API; exits on unlock
  run/linux.rs   evdev grab of keyboards and mice only, nix poll loop; ungrabs on unlock
```

The platform modules own the input devices and the event loop, and hand each
key press to `Unlock` and each event to `Gotcha::on_input`. Everything that
decides something lives in `domain.rs`, where it takes the key or the time as an
argument, so the tests never touch a camera or a device.

The secret is `SECRET` in each platform module, a slice of that platform's key
type. `Unlock` is generic over the key so neither platform maps into a shared
enum. Escape alone today; a longer sequence is a longer slice.

## Platform notes

macOS: `rdev::grab` has no stop call, so unlocking calls `process::exit(0)`.
Needs Accessibility access for the terminal.

Linux: `rdev` grabs every evdev device, Bluetooth and network controllers
included, which disconnects them. So the Linux path uses `evdev` directly and
grabs only devices that look like a keyboard (repeat events plus A, Enter and
Space) or a mouse (relative X and Y). The user has to be in the `input` group.
Rebuilds the `PollFd` list every round because each one borrows its device.

## Checks

```bash
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

The Linux module only compiles on Linux; `cargo check --target
x86_64-unknown-linux-gnu` from a Mac fails in mozjpeg's build script before it
reaches this crate.
