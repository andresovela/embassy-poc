use super::PowerHoldGpio;

pub struct Power {
    power_hold_gpio: PowerHoldGpio,
}

impl Power {
    pub fn new(power_hold_gpio: PowerHoldGpio) -> Self {
        Self { power_hold_gpio }
    }

    pub fn hold(&mut self) {
        self.power_hold_gpio.set_high();
    }

    pub fn release(&mut self) {
        self.power_hold_gpio.set_low();
    }
}
