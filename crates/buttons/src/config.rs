use crate::{Id, Ms};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RepeatedPressMode {
    Immediate,
    Deferred,
}

pub struct Config<'a> {
    pub short_press_duration: Ms,
    pub medium_press_duration: Ms,
    pub long_press_duration: Ms,
    pub very_long_press_duration: Ms,
    pub hold_event_interval: Ms,
    pub repeated_press_threshold_duration: Ms,
    pub buttons_with_repeated_press_support: Option<&'a [Id]>,
    pub repeated_press_mode: RepeatedPressMode,
    pub enable_raw_press_release_events: bool,
}
