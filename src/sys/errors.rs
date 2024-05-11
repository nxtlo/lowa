use core::fmt;

#[derive(Debug, Copy, Clone)]
pub struct ConversionError<'a> {
    pub message: &'a str,
    pub bytes: &'a [u8],
}

impl<'a> ConversionError<'a> {
    pub const fn new(message: &'a str, bytes: &'a [u8]) -> Self {
        Self { message, bytes }
    }
}

impl fmt::Display for ConversionError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.message.fmt(f)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum KernelError<'a> {
    Write { message: &'a str, code: u16 },
    Read { message: &'a str, code: u16 },
    None,
}

impl fmt::Display for KernelError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Read { message, code } => {
                write!(f, "ReadError(message: {}, code: {})", message, code)
            }
            Self::Write { message, code } => {
                write!(f, "WriteError(message: {}, code: {})", message, code)
            }
            Self::None => write!(f, "None"),
        }
    }
}
