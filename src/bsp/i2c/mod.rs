use embassy_stm32::i2c::{I2c, Instance, RxDma, TxDma};
use embassy_time::{Duration, Instant, Timer};

use defmt::info;

pub mod address;

pub struct RobustI2c<'a, T: Instance, TXDMA, RXDMA> {
    i2c: I2c<'a, T, TXDMA, RXDMA>,
    timeout: Duration,
    retries: u8,
}

impl<'a, T: Instance, TXDMA, RXDMA> RobustI2c<'a, T, TXDMA, RXDMA> {
    pub fn new(i2c: I2c<'a, T, TXDMA, RXDMA>, timeout: Duration, retries: u8) -> Self {
        Self {
            i2c,
            timeout,
            retries,
        }
    }
}

impl<'a, T: Instance, TXDMA, RXDMA> embedded_hal::i2c::ErrorType
    for RobustI2c<'a, T, TXDMA, RXDMA>
{
    type Error = embassy_stm32::i2c::Error;
}

impl<'a, T: Instance, TXDMA, RXDMA> embedded_hal_async::i2c::I2c
    for RobustI2c<'a, T, TXDMA, RXDMA>
{
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        for _ in 0..self.retries {
            let result = self
                .i2c
                .blocking_read_timeout(address, read, timeout_fn(self.timeout));

            if result.is_ok() {
                return Ok(());
            }
        }
        Err(embassy_stm32::i2c::Error::Timeout)
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        info!("Write with {} retries", self.retries);
        for _ in 0..self.retries {
            info!("Trying I2C write");
            let result = self
                .i2c
                .blocking_write_timeout(address, write, timeout_fn(self.timeout));

            if result.is_ok() {
                return Ok(());
            }

            Timer::after(Duration::from_millis(20)).await;
        }
        Err(embassy_stm32::i2c::Error::Timeout)
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        for _ in 0..self.retries {
            let result = self.i2c.blocking_write_read_timeout(
                address,
                write,
                read,
                timeout_fn(self.timeout),
            );

            if result.is_ok() {
                return Ok(());
            }
        }
        Err(embassy_stm32::i2c::Error::Timeout)
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let _ = address;
        let _ = operations;
        todo!()
    }
}

fn timeout_fn(timeout: Duration) -> impl Fn() -> Result<(), embassy_stm32::i2c::Error> {
    let deadline = Instant::now() + timeout;
    move || {
        info!("Instant now {}, deadline {}", Instant::now(), deadline);
        if Instant::now() > deadline {
            Err(embassy_stm32::i2c::Error::Timeout)
        } else {
            Ok(())
        }
    }
}
