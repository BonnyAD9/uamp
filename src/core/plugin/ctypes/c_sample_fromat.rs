use raplay::reexp::SampleFormat;

#[repr(i32)]
#[derive(Debug)]
pub enum CSampleFormat {
    Unknown = 0,
    I8 = -8,
    I16 = -16,
    /// NOT SUPPORTED by raplay sample buffer
    I24 = -24,
    I32 = -32,
    I64 = -64,
    U8 = 8,
    U16 = 16,
    U32 = 32,
    U64 = 64,
    F32 = 3200,
    F64 = 6400,
}

impl CSampleFormat {
    pub fn from_value(v: i32) -> Self {
        match v {
            -8 => Self::I8,
            -16 => Self::I16,
            -24 => Self::I24,
            -32 => Self::I32,
            -64 => Self::I64,
            8 => Self::U8,
            16 => Self::U16,
            32 => Self::U16,
            64 => Self::U64,
            3200 => Self::F32,
            6400 => Self::F64,
            _ => Self::Unknown,
        }
    }
}

impl From<SampleFormat> for CSampleFormat {
    fn from(value: SampleFormat) -> Self {
        match value {
            SampleFormat::I8 => Self::I8,
            SampleFormat::I16 => Self::I16,
            SampleFormat::I24 => Self::I24,
            SampleFormat::I32 => Self::I32,
            SampleFormat::I64 => Self::I64,
            SampleFormat::U8 => Self::U8,
            SampleFormat::U16 => Self::U16,
            SampleFormat::U32 => Self::U32,
            SampleFormat::U64 => Self::U64,
            SampleFormat::F32 => Self::F32,
            SampleFormat::F64 => Self::F64,
            _ => Self::Unknown,
        }
    }
}

impl From<CSampleFormat> for SampleFormat {
    fn from(value: CSampleFormat) -> Self {
        match value {
            CSampleFormat::I8 => Self::I8,
            CSampleFormat::I16 => Self::I16,
            CSampleFormat::I24 => Self::I24,
            CSampleFormat::I32 => Self::I32,
            CSampleFormat::I64 => Self::I64,
            CSampleFormat::U8 => Self::U8,
            CSampleFormat::U16 => Self::U16,
            CSampleFormat::U32 => Self::U32,
            CSampleFormat::U64 => Self::U64,
            CSampleFormat::F32 => Self::F32,
            CSampleFormat::F64 => Self::F64,
            CSampleFormat::Unknown => Self::F32,
        }
    }
}
