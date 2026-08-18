# Gotcha

Security tool that catches people accessing your computer while you're away.

## How It Works

- Grabs all input devices (keyboard, mouse) — they stop reaching the desktop
- Snaps a webcam photo on any input, saves timestamped to `captures/`
- Debounces captures to max one per second
- Typing the secret key (currently Escape) exits and releases grab

## Current State — Working on macOS and Linux

- Working proof of concept on both platforms
- Camera module with warmup frames and shoot method
- `CaptureState` struct manages camera + debounce via `jiff` timestamps
- Timestamped photo filenames: `YYYY-MM-DD_HH-MM-SS.fff.png`
- Interior mutability via `Rc<Mutex<CaptureState>>` for callbacks

### Platform Implementations

**macOS** (`src/run/macos.rs`):
- Uses `rdev::grab` with `unstable_grab` feature
- Callback returns `None` to swallow events, `process::exit(0)` on Escape
- No clean stop API — must exit process to break grab loop

**Linux** (`src/run/linux.rs`):
- Uses `evdev` directly for selective device grabbing (not rdev — see Known Issues)
- Enumerates `/dev/input/event*`, filters to keyboards and mice only via capability heuristics
- `Identify` trait on `Device`: `is_probably_keyboard()` (EV_REPEAT + alpha keys), `is_probably_mouse()` (REL_X + REL_Y)
- `IsSecret` trait on `InputEvent`: matches `EventSummary::Key(_, KEY_ESC, 1)` (key press)
- Poll loop via `nix::poll` — rebuilds `PollFd` vec each iteration to avoid borrow conflicts
- Only reads from devices with `POLLIN` ready (avoids blocking on idle devices)
- Clean ungrab on exit — calls `device.ungrab()` on all devices before breaking loop

### TODO

- Configurable secret key sequence (not just Escape)
- Configurable debounce interval

## Project Structure

- `src/main.rs` — entry point, cfg-switches to platform-specific `run()`
- `src/run/macos.rs` — macOS grab via rdev
- `src/run/linux.rs` — Linux grab via evdev + nix poll
- `src/capture_state.rs` — `CaptureState` struct, debounce logic, photo save
- `src/camera.rs` — camera init, warmup, shoot
- `context/` — project context and rules

## Cross-Platform

- Compiled binary is always native to one platform
- Rust `#[cfg(target_os = "...")]` for conditional compilation
- Platform-specific deps in Cargo.toml via `[target.'cfg(...)'.dependencies]`
- Shared code: camera, CaptureState, image saving
- Platform-specific: grab mechanism and event loop

## Crates

Shared:
- `nokhwa` — webcam capture
- `image` — saving photos
- `anyhow` — error handling
- `jiff` — timestamps for debounce and filenames

macOS only:
- `rdev` (with `unstable_grab` feature) — input grab via Accessibility API / CGEventTap

Linux only:
- `evdev` — device enumeration, capability detection, grab/ungrab
- `nix` (poll feature) — poll loop across grabbed devices

## Known Issues

- `rdev` grab on Linux grabs ALL evdev devices including Bluetooth/network controllers, causing disconnects — this is why Linux uses `evdev` directly with selective grabbing instead
- `rdev` listen (not grab) does NOT work on Wayland — but grab does (uses evdev directly)
- macOS grab has no clean stop API — uses `process::exit(0)`

## Permissions

- Linux: user must be in `input` group for `/dev/input/*`
- macOS: app needs Accessibility API access (System Preferences > Security & Privacy > Privacy > Accessibility)
