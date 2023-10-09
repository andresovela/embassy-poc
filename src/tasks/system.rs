use crate::bsp::{power::Power, PowerHoldGpio, Watchdog};

use defmt::{info, Format};
use embassy_time::{Duration, Timer};

pub const QUEUE_SIZE: usize = 3;
pub const IDLE_TIMEOUT_MS: u64 = 1000;

#[derive(Format)]
pub enum Message {
    PowerOn,
    PowerOff,
}

pub struct System {
    power: Power,
    watchdog: Watchdog,
}

impl System {
    pub fn new(power_hold_gpio: PowerHoldGpio, watchdog: Watchdog) -> Self {
        let power = Power::new(power_hold_gpio);
        Self { power, watchdog }
    }
}

#[embassy_executor::task]
pub async fn task(mut system: System) {
    system.watchdog.unleash();

    system.power.hold();

    loop {
        info!("tick");
        system.watchdog.pet();
        Timer::after(Duration::from_secs(1)).await;
    }
}
