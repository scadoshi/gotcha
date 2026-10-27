# Vendored crates

## nokhwa-bindings-macos 0.2.3

The macOS camera bindings for `nokhwa`, copied from crates.io and patched, by the Nokhwa contributors under the Apache License 2.0 (see the header in `build.rs`).

The patch is in `AVCaptureDevice::set_all` in `src/lib.rs`, marked "gotcha patch". The original picks the last format at the requested resolution but the frame rate range from whichever format matched the requested rate, so on a camera with several formats at one resolution it can set a frame duration the active format does not support. AVFoundation throws, and the exception aborts the process. The patch chooses the format and its range together.

Drop the vendored copy and the `[patch.crates-io]` entry once a `nokhwa-bindings-macos` release fixes this.
