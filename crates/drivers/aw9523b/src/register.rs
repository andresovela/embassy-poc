
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Register {
    InputPort0 = 0x00,
    InputPort1 = 0x01,
    OutputPort0 = 0x02,
    OutputPort1 = 0x03,
    ConfigPort0 = 0x04,
    ConfigPort1 = 0x05,
    IntPort0 = 0x06,
    IntPort1 = 0x07,
    Id = 0x10,
    Ctl = 0x11,
    LedModeSwitchP0 = 0x12,
    LedModeSwitchP1 = 0x13,
    Dim0 = 0x20,
    Dim1 = 0x21,
    Dim2 = 0x22,
    Dim3 = 0x23,
    Dim4 = 0x24,
    Dim5 = 0x25,
    Dim6 = 0x26,
    Dim7 = 0x27,
    Dim8 = 0x28,
    Dim9 = 0x29,
    Dim10 = 0x2A,
    Dim11 = 0x2B,
    Dim12 = 0x2C,
    Dim13 = 0x2D,
    Dim14 = 0x2E,
    Dim15 = 0x2F,
    SwRstn = 0x7F,
}

impl Register {
    /// Get the address of the register
    pub fn addr(self) -> u8 {
        self as u8
    }

    /// Checks if the register is read-only
    pub fn is_read_only(self) -> bool {
        matches!(
            self,
            Register::InputPort0 |
            Register::InputPort1 |
            Register::Id
        )
    }
}
