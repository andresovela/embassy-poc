#![no_std]
#![feature(async_fn_in_trait)]

use register::Register;
pub use error::Error as AwError;

mod register;
mod error;

pub struct Aw9523b<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C, E> Aw9523b<I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
{
    /// Creates a new instance of an AW9523B driver.
    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self {
            i2c,
            addr,
        }
    }

    /// Sends a command to perform a software reset.
    pub async fn software_reset(&mut self) -> Result<(), AwError<E>> {
        self.write_register(Register::SwRstn, 0x00).await
    }

    /// Reads the port input state.
    pub async fn read_port(&mut self, port: Port) -> Result<u8, AwError<E>> {
        let register = match port {
            Port::Port0 => Register::InputPort0,
            Port::Port1 => Register::InputPort1,
        };
        self.read_register(register).await
    }

    pub async fn read_pin(&mut self, pin: Pin) -> Result<PinState, AwError<E>> {
        let port_state = self.read_port(pin.0).await?;
        let pin_state = ((port_state >> pin.1 as u8) & 0x01) != 0;
        Ok(pin_state.into())
    }

    pub async fn get_port_output_state(&mut self, port: Port) -> Result<u8, AwError<E>> {
        let register = match port {
            Port::Port0 => Register::OutputPort0,
            Port::Port1 => Register::OutputPort1,
        };
        self.read_register(register).await
    }

    pub async fn set_port_output_state(&mut self, port: Port, value: u8) -> Result<(), AwError<E>> {
        let register = match port {
            Port::Port0 => Register::OutputPort0,
            Port::Port1 => Register::OutputPort1,
        };
        self.write_register(register, value).await
    }

    pub async fn get_pin_output_state(&mut self, pin: Pin) -> Result<PinState, AwError<E>> {
        let port_state = self.get_port_output_state(pin.0).await?;
        let pin_state = ((port_state >> pin.1 as u8) & 0x01) != 0;
        Ok(pin_state.into())
    }

    pub async fn set_pin_output_state(&mut self, pin: Pin, state: PinState) -> Result<(), AwError<E>> {
        let register = match pin.0 {
            Port::Port0 => Register::OutputPort0,
            Port::Port1 => Register::OutputPort1,
        };

        let bits = 1 << pin.1 as u8;

        match state {
            PinState::Low => self.clear_register_bits(register, bits).await,
            PinState::High => self.set_register_bits(register, bits).await,
        }
    }

    pub async fn get_port_config(&mut self, port: Port) -> Result<u8, AwError<E>> {
        let register = match port {
            Port::Port0 => Register::ConfigPort0,
            Port::Port1 => Register::ConfigPort1,
        };
        self.read_register(register).await
    }

    pub async fn set_port_config(&mut self, port: Port, value: u8) -> Result<(), AwError<E>> {
        let register = match port {
            Port::Port0 => Register::ConfigPort0,
            Port::Port1 => Register::ConfigPort1,
        };
        self.write_register(register, value).await
    }

    pub async fn get_pin_config(&mut self, pin: Pin) -> Result<PinMode, AwError<E>> {
        let port_config = self.get_port_config(pin.0).await?;
        let led_mode_switch = self.get_port_led_mode_switch(pin.0).await?;
        let pin_config = ((port_config >> pin.1 as u8) & 0x01) != 0;
        let pin_led_mode_switch = (led_mode_switch >> pin.1 as u8) != 0;

        let pin_mode = match (pin_config, pin_led_mode_switch) {
            (true, _) => PinMode::Input,
            (false, true) => PinMode::Output,
            (false, false) => PinMode::Led,
        };

        Ok(pin_mode)
    }

    pub async fn set_pin_config(&mut self, pin: Pin, mode: PinMode) -> Result<(), AwError<E>> {
        let config_register = match pin.0 {
            Port::Port0 => Register::ConfigPort0,
            Port::Port1 => Register::ConfigPort1,
        };

        let led_mode_switch_register = match pin.0 {
            Port::Port0 => Register::LedModeSwitchP0,
            Port::Port1 => Register::LedModeSwitchP1,
        };

        let bits = 1 << pin.1 as u8;

        match mode {
            PinMode::Input => {
                self.set_register_bits(config_register, bits).await?;
                self.set_register_bits(led_mode_switch_register, bits).await
            },
            PinMode::Output => {
                self.clear_register_bits(config_register, bits).await?;
                self.set_register_bits(led_mode_switch_register, bits).await
            },
            PinMode::Led => {
                self.clear_register_bits(config_register, bits).await?;
                self.clear_register_bits(led_mode_switch_register, bits).await
            },
        }
    }

    pub async fn set_pin_led_pwm(&mut self, pin: Pin, pwm: u8) -> Result<(), AwError<E>> {
        let dim_registers = [
            Register::Dim4,     // P0.0
            Register::Dim5,     // P0.1
            Register::Dim6,     // P0.2
            Register::Dim7,     // P0.3
            Register::Dim8,     // P0.4
            Register::Dim9,     // P0.5
            Register::Dim10,    // P0.6
            Register::Dim11,    // P0.7
            Register::Dim0,     // P1.0
            Register::Dim1,     // P1.1
            Register::Dim2,     // P1.2
            Register::Dim3,     // P1.3
            Register::Dim12,    // P1.4
            Register::Dim13,    // P1.5
            Register::Dim14,    // P1.6
            Register::Dim15,    // P1.7
        ];

        let register = dim_registers[(pin.0 as usize * 8) + pin.1 as usize];
        self.write_register(register, pwm).await
    }

    pub async fn get_port_interrupt_config(&mut self, port: Port) -> Result<u8, AwError<E>> {
        let register = match port {
            Port::Port0 => Register::IntPort0,
            Port::Port1 => Register::IntPort1,
        };
        self.read_register(register).await
    }

    pub async fn set_port_interrupt_config(&mut self, port: Port, value: u8) -> Result<(), AwError<E>> {
        let register = match port {
            Port::Port0 => Register::IntPort0,
            Port::Port1 => Register::IntPort1,
        };
        self.write_register(register, value).await
    }

    pub async fn get_pin_interrupt_config(&mut self, pin: Pin) -> Result<bool, AwError<E>> {
        let port_config = self.get_port_interrupt_config(pin.0).await?;
        let pin_config = ((port_config >> pin.1 as u8) & 0x01) != 0;
        Ok(pin_config.into())
    }

    pub async fn enable_pin_interrupt(&mut self, pin: Pin, enable: bool) -> Result<(), AwError<E>> {
        let register = match pin.0 {
            Port::Port0 => Register::IntPort0,
            Port::Port1 => Register::IntPort1,
        };

        let bits = 1 << pin.1 as u8;

        match enable {
            true => self.clear_register_bits(register, bits).await,
            false => self.set_register_bits(register, bits).await,
        }
    }

    pub async fn read_device_id(&mut self) -> Result<u8, AwError<E>> {
        self.read_register(Register::Id).await
    }

    pub async fn get_port0_drive_mode(&mut self) -> Result<Port0OutputDriveMode, AwError<E>> {
        let value = self.read_register(Register::Ctl).await?;
        let port0_drive_mode = ((value >> 4) & 0x01) != 0;
        Ok(port0_drive_mode.into())
    }

    pub async fn set_port0_drive_mode(&mut self, mode: Port0OutputDriveMode) -> Result<(), AwError<E>> {
        match mode {
            Port0OutputDriveMode::OpenDrain => self.clear_register_bits(Register::Ctl, 0x10).await,
            Port0OutputDriveMode::PushPull => self.set_register_bits(Register::Ctl, 0x10).await,
        }
    }

    pub async fn get_drive_current(&mut self) -> Result<DriveCurrent, AwError<E>> {
        let value = self.read_register(Register::Ctl).await?;
        let drive_current = value & 0x03;
        let drive_current = match drive_current {
            0 => DriveCurrent::Max,
            1 => DriveCurrent::High,
            2 => DriveCurrent::Mid,
            3 => DriveCurrent::Low,
            _ => unreachable!(),
        };
        Ok(drive_current)
    }

    pub async fn set_drive_current(&mut self, drive_current: DriveCurrent) -> Result<(), AwError<E>> {
        let f = |v| {
            let other_bits = v & !0x03;
            other_bits | drive_current as u8
        };
        self.modify_register(Register::Ctl, f).await
    }

    pub async fn get_port_led_mode_switch(&mut self, port: Port) -> Result<u8, AwError<E>> {
        let register = match port {
            Port::Port0 => Register::LedModeSwitchP0,
            Port::Port1 => Register::LedModeSwitchP1,
        };
        self.read_register(register).await
    }

    pub async fn set_port_led_mode_switch(&mut self, port: Port, value: u8) -> Result<(), AwError<E>> {
        let register = match port {
            Port::Port0 => Register::LedModeSwitchP0,
            Port::Port1 => Register::LedModeSwitchP1,
        };
        self.write_register(register, value).await
    }
}

pub trait BasicOps {
    type Error;

    /// Writes a value to a given register.
    async fn write_register(&mut self, register: Register, value: u8) -> Result<(), AwError<Self::Error>>;

    /// Reads the value from the given register.
    async fn read_register(&mut self, register: Register) -> Result<u8, AwError<Self::Error>>;

    /// Modifies the value of a given register.
    async fn modify_register<F>(&mut self, register: Register, f: F) -> Result<(), AwError<Self::Error>>
    where
        F: FnOnce(u8) -> u8,
    {
        let value = self.read_register(register).await?;
        self.write_register(register, f(value)).await
    }

    /// Sets some bits of a given register.
    async fn set_register_bits(&mut self, register: Register, bits: u8) -> Result<(), AwError<Self::Error>> {
        self.modify_register(register, |v| v | bits).await
    }

    /// Clears some bits of a given register.
    async fn clear_register_bits(&mut self, register: Register, bits: u8) -> Result<(), AwError<Self::Error>> {
        self.modify_register(register, |v| v & !bits).await
    }
}

impl<I2C, E> BasicOps for Aw9523b<I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
{
    type Error = E;
    async fn write_register(&mut self, register: Register, value: u8) -> Result<(), AwError<Self::Error>> {
        if register.is_read_only() {
            return Err(AwError::WriteToReadOnly);
        }

        self.i2c.write(self.addr, &[register.addr(), value]).await.map_err(AwError::I2c)?;
        Ok(())
    }

    async fn read_register(&mut self, register: Register) -> Result<u8, AwError<Self::Error>> {
        let mut buffer = [0u8; 1];
        self.i2c.write_read(self.addr, &[register.addr()], &mut buffer).await.map_err(AwError::I2c)?;
        Ok(buffer[0])
    }
}

pub struct Pin(pub Port, pub PortPin);

#[derive(Clone, Copy)]
pub enum Port {
    Port0,
    Port1,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PortPin {
    P0,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
}

pub enum PinMode {
    Input,
    Output,
    Led,
}

pub enum PinState {
    /// Low logic level.
    Low,

    /// High logic level.
    High,
}

impl From<bool> for PinState {
    fn from(value: bool) -> Self {
        if value {
            PinState::Low
        } else {
            PinState::High
        }
    }
}

pub enum Port0OutputDriveMode {
    /// Pins of port 0 set to open-drain mode.
    OpenDrain,

    /// Pins of port 0 set to push-pull mode.
    PushPull,
}

impl From<bool> for Port0OutputDriveMode {
    fn from(value: bool) -> Self {
        if value {
            Port0OutputDriveMode::OpenDrain
        } else {
            Port0OutputDriveMode::PushPull
        }
    }
}

pub enum DriveCurrent {
    /// I_max set to maximum of 37 mA.
    Max,

    /// I_max limited to 3/4 of I_max.
    High,

    /// I_max limited to 1/2 of I_max.
    Mid,

    /// I_max limited to 1/4 of I_max.
    Low,
}
