#![allow(dead_code)]
use aw9523b::{Aw9523b, AwError, Pin, PinMode, PinState, Port, PortPin};
use embassy_time::{Duration, Timer};

const BT_BUTTON: Pin = Pin(Port::Port0, PortPin::P2);
const PLAY_BUTTON: Pin = Pin(Port::Port0, PortPin::P3);
const PLUS_BUTTON: Pin = Pin(Port::Port0, PortPin::P5);
const MINUS_BUTTON: Pin = Pin(Port::Port0, PortPin::P4);

const STATUS_LED_R: Pin = Pin(Port::Port1, PortPin::P0);
const STATUS_LED_G: Pin = Pin(Port::Port1, PortPin::P2);
const STATUS_LED_B: Pin = Pin(Port::Port1, PortPin::P1);

const SOURCE_LED_R: Pin = Pin(Port::Port1, PortPin::P4);
const SOURCE_LED_G: Pin = Pin(Port::Port1, PortPin::P6);
const SOURCE_LED_B: Pin = Pin(Port::Port1, PortPin::P5);

pub struct Ui<R, I, P, I2C> {
    is_initialized: bool,
    io_expander: Aw9523b<I2C>,
    io_exp_reset_gpio: R,
    io_exp_int_gpio: I,
    power_button_gpio: P,
}

#[derive(Debug)]
pub enum Error<E> {
    UsedBeforeInitialization,
    IoExpanderError(AwError<E>),
}

impl<E> From<AwError<E>> for Error<E> {
    fn from(value: AwError<E>) -> Self {
        Error::IoExpanderError(value)
    }
}

impl<R, I, P, I2C, E> Ui<R, I, P, I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal_async::i2c::I2c<Error = E>,
    R: embedded_hal::digital::OutputPin,
    P: embedded_hal::digital::InputPin,
{
    pub fn new(
        io_expander: Aw9523b<I2C>,
        io_exp_reset_gpio: R,
        io_exp_int_gpio: I,
        power_button_gpio: P,
    ) -> Self {
        Self {
            is_initialized: false,
            io_expander,
            io_exp_reset_gpio,
            io_exp_int_gpio,
            power_button_gpio,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn reset(mut self) {
        self.io_exp_reset_gpio.set_low().unwrap();
        self.is_initialized = false;
    }

    pub async fn initialize(&mut self) -> Result<(), Error<E>> {
        self.io_exp_reset_gpio.set_high().unwrap();

        Timer::after(Duration::from_millis(10)).await;

        self.io_expander.software_reset().await?;

        // Configure button GPIOs as inputs
        self.io_expander
            .set_pin_config(BT_BUTTON, PinMode::Input)
            .await?;
        self.io_expander
            .set_pin_config(PLAY_BUTTON, PinMode::Input)
            .await?;
        self.io_expander
            .set_pin_config(PLUS_BUTTON, PinMode::Input)
            .await?;
        self.io_expander
            .set_pin_config(MINUS_BUTTON, PinMode::Input)
            .await?;

        // Enable interrupts on button GPIOs
        self.io_expander
            .enable_pin_interrupt(BT_BUTTON, true)
            .await?;
        self.io_expander
            .enable_pin_interrupt(PLAY_BUTTON, true)
            .await?;
        self.io_expander
            .enable_pin_interrupt(PLUS_BUTTON, true)
            .await?;
        self.io_expander
            .enable_pin_interrupt(MINUS_BUTTON, true)
            .await?;

        // Configure Power LED GPIOs as LEDs
        self.io_expander
            .set_pin_config(STATUS_LED_R, PinMode::Led)
            .await?;
        self.io_expander
            .set_pin_config(STATUS_LED_G, PinMode::Led)
            .await?;
        self.io_expander
            .set_pin_config(STATUS_LED_B, PinMode::Led)
            .await?;

        // Configure BT LED GPIOs as LEDs
        self.io_expander
            .set_pin_config(SOURCE_LED_R, PinMode::Led)
            .await?;
        self.io_expander
            .set_pin_config(SOURCE_LED_G, PinMode::Led)
            .await?;
        self.io_expander
            .set_pin_config(SOURCE_LED_B, PinMode::Led)
            .await?;

        self.is_initialized = true;
        Ok(())
    }

    pub fn is_power_pressed(&mut self) -> Result<bool, Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }
        Ok(self.power_button_gpio.is_low().unwrap())
    }

    pub async fn is_bt_pressed(&mut self) -> Result<bool, Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }

        let pin = self.io_expander.read_pin(BT_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn is_play_pause_pressed(&mut self) -> Result<bool, Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }

        let pin = self.io_expander.read_pin(PLAY_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn is_plus_pressed(&mut self) -> Result<bool, Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }

        let pin = self.io_expander.read_pin(PLUS_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn is_minus_pressed(&mut self) -> Result<bool, Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }

        let pin = self.io_expander.read_pin(MINUS_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn set_status_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }

        self.io_expander.set_pin_led_pwm(STATUS_LED_R, r).await?;
        self.io_expander.set_pin_led_pwm(STATUS_LED_G, g).await?;
        self.io_expander.set_pin_led_pwm(STATUS_LED_B, b).await?;
        Ok(())
    }

    pub async fn set_source_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), Error<E>> {
        if !self.is_initialized {
            return Err(Error::UsedBeforeInitialization);
        }

        self.io_expander.set_pin_led_pwm(SOURCE_LED_R, r).await?;
        self.io_expander.set_pin_led_pwm(SOURCE_LED_G, g).await?;
        self.io_expander.set_pin_led_pwm(SOURCE_LED_B, b).await?;
        Ok(())
    }
}
