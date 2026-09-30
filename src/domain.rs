//! The decisions: when a key press unlocks, when an input earns a photo, and
//! what the photo is called. No camera, no input devices, no clock. The caller
//! passes the key and the time in.

use jiff::Zoned;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Matches key presses against a secret sequence, one press at a time.
#[derive(Debug, Clone)]
pub struct Unlock<K> {
    secret: Vec<K>,
    recent: VecDeque<K>,
}

impl<K: PartialEq + Copy> Unlock<K> {
    pub fn new(secret: impl Into<Vec<K>>) -> Self {
        let secret = secret.into();
        Self {
            recent: VecDeque::with_capacity(secret.len()),
            secret,
        }
    }

    /// Feeds one key press. True when the last presses spell the secret.
    /// An empty secret never unlocks.
    pub fn press(&mut self, key: K) -> bool {
        if self.secret.is_empty() {
            return false;
        }
        if self.recent.len() == self.secret.len() {
            self.recent.pop_front();
        }
        self.recent.push_back(key);
        let unlocked = self.recent.iter().eq(self.secret.iter());
        if unlocked {
            self.recent.clear();
        }
        unlocked
    }
}

/// Allows one shot per interval. The first request always fires.
#[derive(Debug, Clone)]
pub struct Shutter {
    interval: Duration,
    last: Option<Instant>,
}

impl Shutter {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: None,
        }
    }

    /// True when a shot is allowed at `now`, which then starts the next interval.
    pub fn fire(&mut self, now: Instant) -> bool {
        if self
            .last
            .is_some_and(|last| now.duration_since(last) < self.interval)
        {
            return false;
        }
        self.last = Some(now);
        true
    }
}

/// `2026-09-30_09-08-37.123.png`: sorts by time in a directory listing.
pub fn file_name(at: &Zoned) -> String {
    format!("{}.png", at.strftime("%Y-%m-%d_%H-%M-%S%.3f"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::{Timestamp, tz::TimeZone};

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Key {
        A,
        B,
        Esc,
    }

    #[test]
    fn a_single_key_secret_unlocks_on_that_key() {
        let mut unlock = Unlock::new([Key::Esc]);
        assert!(!unlock.press(Key::A));
        assert!(unlock.press(Key::Esc));
    }

    #[test]
    fn a_sequence_unlocks_only_when_typed_in_order() {
        let mut unlock = Unlock::new([Key::A, Key::B, Key::Esc]);
        assert!(!unlock.press(Key::A));
        assert!(!unlock.press(Key::B));
        assert!(!unlock.press(Key::B));
        assert!(!unlock.press(Key::Esc));
        assert!(!unlock.press(Key::A));
        assert!(!unlock.press(Key::B));
        assert!(unlock.press(Key::Esc));
    }

    #[test]
    fn a_wrong_key_does_not_lose_a_restart_inside_it() {
        // A, A, A, B ends in the secret even though the third A was "wrong".
        let mut unlock = Unlock::new([Key::A, Key::A, Key::B]);
        for key in [Key::A, Key::A, Key::A] {
            assert!(!unlock.press(key));
        }
        assert!(unlock.press(Key::B));
    }

    #[test]
    fn unlocking_starts_the_match_over() {
        let mut unlock = Unlock::new([Key::A, Key::A]);
        assert!(!unlock.press(Key::A));
        assert!(unlock.press(Key::A));
        assert!(
            !unlock.press(Key::A),
            "the second A of the last unlock does not count again"
        );
        assert!(unlock.press(Key::A));
    }

    #[test]
    fn an_empty_secret_never_unlocks() {
        let mut unlock = Unlock::<Key>::new([]);
        assert!(!unlock.press(Key::Esc));
        assert!(!unlock.press(Key::Esc));
    }

    #[test]
    fn the_first_shot_always_fires() {
        let mut shutter = Shutter::new(Duration::from_secs(1));
        assert!(shutter.fire(Instant::now()));
    }

    #[test]
    fn a_shot_inside_the_interval_is_refused_and_one_after_it_fires() {
        let mut shutter = Shutter::new(Duration::from_secs(1));
        let start = Instant::now();
        assert!(shutter.fire(start));
        assert!(!shutter.fire(start + Duration::from_millis(999)));
        assert!(shutter.fire(start + Duration::from_secs(1)));
        assert!(!shutter.fire(start + Duration::from_millis(1500)));
    }

    #[test]
    fn a_refused_shot_does_not_move_the_interval() {
        let mut shutter = Shutter::new(Duration::from_secs(1));
        let start = Instant::now();
        assert!(shutter.fire(start));
        assert!(!shutter.fire(start + Duration::from_millis(900)));
        assert!(shutter.fire(start + Duration::from_secs(1)));
    }

    #[test]
    fn a_zero_interval_always_fires() {
        let mut shutter = Shutter::new(Duration::ZERO);
        let now = Instant::now();
        assert!(shutter.fire(now));
        assert!(shutter.fire(now));
    }

    #[test]
    fn the_file_name_carries_the_time_to_the_millisecond() {
        let at = "2026-09-30T09:08:37.123Z"
            .parse::<Timestamp>()
            .expect("valid timestamp")
            .to_zoned(TimeZone::UTC);
        assert_eq!(file_name(&at), "2026-09-30_09-08-37.123.png");
    }

    #[test]
    fn file_names_sort_by_time() {
        let name = |s: &str| {
            file_name(
                &s.parse::<Timestamp>()
                    .expect("valid timestamp")
                    .to_zoned(TimeZone::UTC),
            )
        };
        let mut names = [
            name("2026-09-30T10:00:00Z"),
            name("2026-09-30T09:59:59.999Z"),
            name("2025-12-31T23:59:59Z"),
        ];
        names.sort();
        assert_eq!(
            names,
            [
                "2025-12-31_23-59-59.000.png",
                "2026-09-30_09-59-59.999.png",
                "2026-09-30_10-00-00.000.png",
            ]
        );
    }
}
