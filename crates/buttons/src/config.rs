use crate::Id;
use embassy_time::Duration;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RepeatedPressMode {
    Immediate,
    Deferred,
}

pub struct Config<'a> {
    pub short_press_duration: Duration,
    pub medium_press_duration: Duration,
    pub long_press_duration: Duration,
    pub very_long_press_duration: Duration,
    pub hold_event_interval: Duration,
    pub repeated_press_threshold_duration: Duration,
    pub buttons_with_repeated_press_support: Option<&'a [Id]>,
    pub repeated_press_mode: RepeatedPressMode,
    pub enable_raw_press_release_events: bool,
}
