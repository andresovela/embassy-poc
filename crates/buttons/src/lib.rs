#![no_std]

use core::ops::{Add, Sub};

mod config;
pub use config::*;

mod event;
pub use event::*;

mod fmt;
pub use fmt::*;

enum State {
    Released,
    Debouncing,
    Pressed,
}

pub struct Buttons<'a, T: Handler> {
    handler: T,
    config: Config<'a>,
    state: State,
    last_button_pressed: Option<Id>,
    last_press_start_timestamp: Option<Ms>,
    last_hold_event_timestamp: Option<Ms>,
    last_press_event_sent: Option<Event>,
    consecutive_press_count: u8,
}

impl<'a, T: Handler> Buttons<'a, T> {
    pub fn new(handler: T, config: Config<'a>) -> Self {
        Self {
            handler,
            config,
            state: State::Released,
            last_button_pressed: None,
            last_press_start_timestamp: None,
            last_press_event_sent: None,
            last_hold_event_timestamp: None,
            consecutive_press_count: 0,
        }
    }

    pub fn update_input(&mut self, input: Option<Id>) {
        self.state = match self.state {
            State::Released => self.released_state(input),
            State::Debouncing => self.debouncing_state(input),
            State::Pressed => self.pressed_state(input),
        };
    }

    fn released_state(&mut self, input: Option<Id>) -> State {
        let Some(input) = input else {
            // We had one press but didn't send any events because the press was released too quickly
            // before we could determine if the repeated presses had ended
            if self.last_press_event_sent.is_none() {
                let current_time = self.handler.get_current_timestamp();
                let time_since_last_press_started = current_time.elapsed_since(self.last_press_start_timestamp.unwrap());

                if time_since_last_press_started > self.config.repeated_press_threshold_duration {
                    let event = if let Some(last_press_event_sent) = self.last_press_event_sent {
                        last_press_event_sent.with_length(Length::Short)
                    } else {
                        // The first press event counts as the reference time to start sending hold events
                        self.last_hold_event_timestamp = Some(self.handler.get_current_timestamp());

                        match self.consecutive_press_count {
                            0 => unreachable!(),
                            1 => Event::Press(Kind::Single(Length::Short)),
                            2 => Event::Press(Kind::Double(Length::Short)),
                            3 => Event::Press(Kind::Triple(Length::Short)),
                            n => Event::Press(Kind::Repeated(Length::Short, n)),
                        }
                    };

                    if Some(event) != self.last_press_event_sent {
                        self.last_press_event_sent = Some(event);
                        self.handler.on_event(self.last_button_pressed.unwrap(), event);
                    }
                }
            }
            return State::Released;
        };

        debug!("Button pressed {}", input);
        if let Some(last_press_start_timestamp) = self.last_press_start_timestamp {
            let current_time = self.handler.get_current_timestamp();
            let time_since_last_press = current_time.elapsed_since(last_press_start_timestamp);

            if Some(input) != self.last_button_pressed
                || !self.input_supports_repeated_press_detection(input)
                || time_since_last_press > self.config.repeated_press_threshold_duration
            {
                trace!("Consecutive count reset");
                self.consecutive_press_count = 0;
            }
        }

        self.last_press_start_timestamp = Some(self.handler.get_current_timestamp());
        self.start_press(input);

        if self.config.enable_raw_press_release_events {
            self.handler.on_event(input, Event::Press(Kind::Raw));
        }

        if self.config.short_press_duration > Ms(0) {
            return State::Debouncing;
        } else {
            // Call the press state immediately because we want to start handling the press
            return self.pressed_state(Some(input));
        }
    }

    fn debouncing_state(&mut self, input: Option<Id>) -> State {
        let Some(input) = input else {
            debug!("Button released");

            if self.config.enable_raw_press_release_events {
                self.handler.on_event(self.last_button_pressed.unwrap(), Event::Release(Kind::Raw));
            }

            return State::Released;
        };

        let current_time = self.handler.get_current_timestamp();

        // If the input changes mid-press, reset the press timestamp
        if Some(input) != self.last_button_pressed {
            self.last_press_start_timestamp = Some(current_time);
        }

        let time_since_press_started =
            current_time.elapsed_since(self.last_press_start_timestamp.unwrap());

        if time_since_press_started > self.config.short_press_duration {
            // Reset the consecutive press count if the last button press does not match the current one
            // We do that here too in case the button combination changes mid-press without releasing first
            if Some(input) != self.last_button_pressed {
                self.consecutive_press_count = 0;
            }

            self.start_press(input);

            // Call the press state immediately because we want to start handling the press
            return self.pressed_state(Some(input));
        }

        State::Debouncing
    }

    fn pressed_state(&mut self, input: Option<Id>) -> State {
        let Some(input) = input else {
            debug!("{} press released", input);

            if self.config.enable_raw_press_release_events {
                self.handler.on_event(self.last_button_pressed.unwrap(), Event::Release(Kind::Raw));
            }

            // Only send the press released event if we actually got to send a press event before the button was released
            if let Some(press_event) = &self.last_press_event_sent {
                self.handler.on_event(self.last_button_pressed.unwrap(), press_event.into_release());
            }

            return State::Released;
        };

        // The button press/combination changed without releasing first
        if Some(input) != self.last_button_pressed {
            trace!("Button press changed to {} without releasing first", input);

            if self.config.enable_raw_press_release_events {
                self.handler
                    .on_event(self.last_button_pressed.unwrap(), Event::Release(Kind::Raw));
            }

            // Only send the press released event if we actually got to send a press event before the button was released
            if let Some(press_event) = &self.last_press_event_sent {
                self.handler.on_event(
                    self.last_button_pressed.unwrap(),
                    press_event.into_release(),
                );
            }

            // The input changed to something else,
            // therefore it can't be a repeated press
            self.consecutive_press_count = 0;
            self.last_press_start_timestamp = Some(self.handler.get_current_timestamp());
            self.start_press(input);

            if self.config.short_press_duration > Ms(0) {
                return State::Debouncing;
            }
        }

        let current_time = self.handler.get_current_timestamp();
        let time_since_press_started =
            current_time.elapsed_since(self.last_press_start_timestamp.unwrap());

        // Very long press
        let length = if time_since_press_started >= self.config.very_long_press_duration {
            Length::VeryLong
        } else if time_since_press_started >= self.config.long_press_duration {
            Length::Long
        } else if time_since_press_started >= self.config.medium_press_duration {
            Length::Medium
        } else {
            Length::Short
        };

        // Check if enough time has passed to determine whether this is a normal or repeated press
        let can_send_repeat_press = self.config.repeated_press_mode == RepeatedPressMode::Immediate
            || time_since_press_started > self.config.repeated_press_threshold_duration;

        if !self.input_supports_repeated_press_detection(input) || can_send_repeat_press {
            let event = if let Some(last_press_event_sent) = self.last_press_event_sent {
                last_press_event_sent.with_length(length)
            } else {
                // The first press event counts as the reference time to start sending hold events
                self.last_hold_event_timestamp = Some(self.handler.get_current_timestamp());

                match self.consecutive_press_count {
                    0 => unreachable!(),
                    1 => Event::Press(Kind::Single(length)),
                    2 => Event::Press(Kind::Double(length)),
                    3 => Event::Press(Kind::Triple(length)),
                    n => Event::Press(Kind::Repeated(length, n)),
                }
            };

            if Some(event) != self.last_press_event_sent {
                self.last_press_event_sent = Some(event);
                self.handler.on_event(input, event);
            }
        }

        if let Some(last_hold_event_timestamp) = self.last_hold_event_timestamp {
            let current_time = self.handler.get_current_timestamp();
            let time_since_last_hold_event = current_time.elapsed_since(last_hold_event_timestamp);
            if time_since_last_hold_event > self.config.hold_event_interval {
                let hold_time =
                    current_time.elapsed_since(self.last_press_start_timestamp.unwrap());

                self.handler.on_event(input, Event::Hold(hold_time));
                self.last_hold_event_timestamp = Some(self.handler.get_current_timestamp());
            }
        }

        State::Pressed
    }

    fn start_press(&mut self, input: Id) {
        self.last_button_pressed = Some(input);
        self.last_press_event_sent = None;
        self.last_hold_event_timestamp = None;
        self.consecutive_press_count += 1;
    }

    fn input_supports_repeated_press_detection(&self, input: Id) -> bool {
        let Some(buttons_with_repeated_press_support) = self.config.buttons_with_repeated_press_support else {
            // If the user provided no list in the configuration, all buttons support repeated presses by default
            return true;
        };

        buttons_with_repeated_press_support.contains(&input)
    }
}

pub trait Handler {
    fn on_event(&self, button: Id, event: Event);

    fn get_current_timestamp(&self) -> Ms;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ms(usize);

impl Ms {
    pub fn elapsed_since(&self, reference: Ms) -> Ms {
        Ms(self.0.wrapping_sub(reference.0))
    }
}

impl Add for Ms {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Ms {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Id(usize);
