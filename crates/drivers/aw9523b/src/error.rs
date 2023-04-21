#[derive(Debug)]
pub enum Error<I2cError> {
    /// I2C bus error.
    I2c(I2cError),

    /// Attempted to write to a read-only register.
    WriteToReadOnly,
}
