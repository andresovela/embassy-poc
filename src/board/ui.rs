#![allow(dead_code)]
use core::marker::PhantomData;
use aw9523b::{Aw9523b, AwError, Port, PortPin, Pin, PinMode, PinState};

const BT_BUTTON: Pin = Pin(Port::Port0, PortPin::P2);
const PLAY_BUTTON: Pin = Pin(Port::Port0, PortPin::P3);
const PLUS_BUTTON: Pin = Pin(Port::Port0, PortPin::P5);
const MINUS_BUTTON: Pin = Pin(Port::Port0, PortPin::P4);

const LED0_R: Pin = Pin(Port::Port1, PortPin::P0);
const LED0_G: Pin = Pin(Port::Port1, PortPin::P2);
const LED0_B: Pin = Pin(Port::Port1, PortPin::P1);

const LED1_R: Pin = Pin(Port::Port1, PortPin::P4);
const LED1_G: Pin = Pin(Port::Port1, PortPin::P6);
const LED1_B: Pin = Pin(Port::Port1, PortPin::P5);

struct ResetAsserted;
struct Uninitialized;
struct Ready;

pub struct IoExpander<T, R, I, I2C> {
    driver: Aw9523b<I2C>,
    reset_gpio: R,
    int_gpio: I,
    _marker: PhantomData<T>,
}


impl<R, I, E, I2C> IoExpander<ResetAsserted, R, I, I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
    R: embedded_hal::digital::OutputPin,
{
    pub fn new(driver: Aw9523b<I2C>, reset_gpio: R, int_gpio: I) -> Self {
        Self {
            driver,
            reset_gpio,
            int_gpio,
            _marker: PhantomData,
        }
    }

    pub fn enable(self) -> IoExpander<Uninitialized, R, I, I2C> {
        self.reset_gpio.set_low().unwrap();
        Self {
            _marker: PhantomData,
            ..self
        }
    }
}

impl<R, I, E, I2C> IoExpander<Uninitialized, R, I, I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
    R: embedded_hal::digital::OutputPin,
{
    pub fn reset(mut self) -> IoExpander<ResetAsserted, R, I, I2C> {
        self.reset_gpio.set_low().unwrap();
        Self {
            _marker: PhantomData,
            ..self
        }
    }

    pub async fn setup_for_normal_operation(self) -> Result<IoExpander<Ready, R, I, I2C>, AwError<E>> {
        self.driver.software_reset().await?;

        // Configure button GPIOs as inputs
        self.driver.set_pin_config(BT_BUTTON, PinMode::Input).await?;
        self.driver.set_pin_config(PLAY_BUTTON, PinMode::Input).await?;
        self.driver.set_pin_config(PLUS_BUTTON, PinMode::Input).await?;
        self.driver.set_pin_config(MINUS_BUTTON, PinMode::Input).await?;

        // Enable interrupts on button GPIOs
        self.driver.enable_pin_interrupt(BT_BUTTON, true).await?;
        self.driver.enable_pin_interrupt(PLAY_BUTTON, true).await?;
        self.driver.enable_pin_interrupt(PLUS_BUTTON, true).await?;
        self.driver.enable_pin_interrupt(MINUS_BUTTON, true).await?;

        // Configure LED0 GPIOs as LEDs
        self.driver.set_pin_config(LED0_R, PinMode::Led).await?;
        self.driver.set_pin_config(LED0_G, PinMode::Led).await?;
        self.driver.set_pin_config(LED0_B, PinMode::Led).await?;

        // Configure LED1 GPIOs as LEDs
        self.driver.set_pin_config(LED1_R, PinMode::Led).await?;
        self.driver.set_pin_config(LED1_G, PinMode::Led).await?;
        self.driver.set_pin_config(LED1_B, PinMode::Led).await?;

        Ok(Self {
            _marker: PhantomData,
            ..self
        })
    }
}

impl<R, I, E, I2C> IoExpander<Ready, R, I, I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
    R: embedded_hal::digital::OutputPin,
{
    pub async fn is_bt_pressed(&mut self) -> Result<bool, AwError<E>> {
        let pin = self.driver.read_pin(BT_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn is_play_pressed(&mut self) -> Result<bool, AwError<E>> {
        let pin = self.driver.read_pin(PLAY_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn is_plus_pressed(&mut self) -> Result<bool, AwError<E>> {
        let pin = self.driver.read_pin(PLUS_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn is_minus_pressed(&mut self) -> Result<bool, AwError<E>> {
        let pin = self.driver.read_pin(MINUS_BUTTON).await?;
        Ok(matches!(pin, PinState::Low))
    }

    pub async fn set_led0(&mut self, r: u8, g: u8, b: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED0_R, r).await?;
        self.driver.set_pin_led_pwm(LED0_G, g).await?;
        self.driver.set_pin_led_pwm(LED0_B, b).await?;
        Ok(())
    }

    pub async fn set_led1(&mut self, r: u8, g: u8, b: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED1_R, r).await?;
        self.driver.set_pin_led_pwm(LED1_G, g).await?;
        self.driver.set_pin_led_pwm(LED1_B, b).await?;
        Ok(())
    }

    pub async fn set_led0_r(&mut self, pwm: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED0_R, pwm).await?;
        Ok(())
    }

    pub async fn set_led0_g(&mut self, pwm: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED0_G, pwm).await?;
        Ok(())
    }

    pub async fn set_led0_b(&mut self, pwm: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED0_B, pwm).await?;
        Ok(())
    }

    pub async fn set_led1_r(&mut self, pwm: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED1_R, pwm).await?;
        Ok(())
    }

    pub async fn set_led1_g(&mut self, pwm: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED1_G, pwm).await?;
        Ok(())
    }

    pub async fn set_led1_b(&mut self, pwm: u8) -> Result<(), AwError<E>> {
        self.driver.set_pin_led_pwm(LED1_B, pwm).await?;
        Ok(())
    }
}
