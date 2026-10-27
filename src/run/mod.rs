//! The platform's input grab and event loop; each module exposes one `run`.

#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "macos")]
pub(crate) mod macos;
