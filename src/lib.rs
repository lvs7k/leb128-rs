pub trait ToLeb128u {
    fn to_leb128u(self) -> Vec<u8>;
}

macro_rules! impl_to_leb128u {
    ($($uint:ty),*) => {
        $(
            impl ToLeb128u for $uint {
                fn to_leb128u(mut self) -> Vec<u8> {
                    let mut result = Vec::new();

                    loop {
                        let byte = (self & 0b01111111) as u8;
                        self >>= 7;

                        if self == 0 {
                            result.push(byte);
                            break;
                        }

                        result.push(byte | 0b10000000);
                    }

                    result
                }
            }
        )*
    };
}

impl_to_leb128u!(u8, u16, u32, u64);

pub trait ToLeb128i {
    fn to_leb128i(self) -> Vec<u8>;
}

macro_rules! impl_to_leb128i {
    ($($iint:ty),*) => {
        $(
            impl ToLeb128i for $iint {
                fn to_leb128i(mut self) -> Vec<u8> {
                    let mut result = Vec::new();

                    loop {
                        let byte = (self & 0b01111111) as u8;
                        self >>= 7;

                        if self == 0 && (byte & 0b01000000) == 0 || self == -1 && (byte & 0b01000000) != 0 {
                            result.push(byte);
                            break;
                        }

                        result.push(byte | 0b10000000);
                    }

                    result
                }
            }
        )*
    };
}

impl_to_leb128i!(i8, i16, i32, i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromLeb128Error {
    TooLongBytes,
    TryFromInt(std::num::TryFromIntError),
}

impl std::fmt::Display for FromLeb128Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FromLeb128Error::*;

        match self {
            TooLongBytes => write!(f, "LEB128 bytes too long"),
            TryFromInt(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for FromLeb128Error {}

pub trait FromLeb128u
where
    Self: Sized,
{
    fn from_leb128u(input: &[u8]) -> Result<Self, FromLeb128Error>;
}

macro_rules! impl_from_leb128u {
    ($($uint:ty),*) => {
        $(
            impl FromLeb128u for $uint {
                fn from_leb128u(input: &[u8]) -> Result<Self, FromLeb128Error> {
                    let mut result: u128 = 0;
                    let mut shift: usize = 0;
                    let size = std::mem::size_of_val(&result) * 8;

                    for &byte in input {
                        result |= ((byte & 0b01111111) as u128) << shift;
                        shift += 7;

                        if shift >= size {
                            return Err(FromLeb128Error::TooLongBytes);
                        }

                        if byte & 0b10000000 == 0 {
                            break;
                        }
                    }

                    <$uint>::try_from(result).map_err(FromLeb128Error::TryFromInt)
                }
            }
        )*
    };
}

impl_from_leb128u!(u8, u16, u32, u64);

pub trait FromLeb128i
where
    Self: Sized,
{
    fn from_leb128i(input: &[u8]) -> Result<Self, FromLeb128Error>;
}

macro_rules! impl_from_leb128i {
    ($($iint:ty),*) => {
        $(
            impl FromLeb128i for $iint {
                fn from_leb128i(input: &[u8]) -> Result<Self, FromLeb128Error> {
                    let mut result: i128 = 0;
                    let mut shift: usize = 0;
                    let size: usize = std::mem::size_of_val(&result) * 8;

                    for &byte in input {
                        result |= ((byte & 0b01111111) as i128) << shift;
                        shift += 7;

                        if shift >= size {
                            return Err(FromLeb128Error::TooLongBytes);
                        }

                        if byte & 0b10000000 == 0 {
                            if byte & 0b01000000 != 0 {
                                result |= !0 << shift;
                            }
                            break;
                        }
                    }

                    <$iint>::try_from(result).map_err(FromLeb128Error::TryFromInt)
                }
            }
        )*
    };
}

impl_from_leb128i!(i8, i16, i32, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_unsigned() {
        assert_eq!(ToLeb128u::to_leb128u(0u8), vec![0x00]);
        assert_eq!(ToLeb128u::to_leb128u(127u8), vec![0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(128u8), vec![0x80, 0x01]);
        assert_eq!(ToLeb128u::to_leb128u(255u8), vec![0xFF, 0x01]);

        assert_eq!(ToLeb128u::to_leb128u(0u16), vec![0x00]);
        assert_eq!(ToLeb128u::to_leb128u(127u16), vec![0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(128u16), vec![0x80, 0x01]);
        assert_eq!(ToLeb128u::to_leb128u(16383u16), vec![0xFF, 0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(16384u16), vec![0x80, 0x80, 0x01]);

        assert_eq!(ToLeb128u::to_leb128u(0u32), vec![0x00]);
        assert_eq!(ToLeb128u::to_leb128u(127u32), vec![0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(128u32), vec![0x80, 0x01]);
        assert_eq!(ToLeb128u::to_leb128u(16383u32), vec![0xFF, 0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(16384u32), vec![0x80, 0x80, 0x01]);

        assert_eq!(ToLeb128u::to_leb128u(0u64), vec![0x00]);
        assert_eq!(ToLeb128u::to_leb128u(127u64), vec![0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(128u64), vec![0x80, 0x01]);
        assert_eq!(ToLeb128u::to_leb128u(16383u64), vec![0xFF, 0x7F]);
        assert_eq!(ToLeb128u::to_leb128u(16384u64), vec![0x80, 0x80, 0x01]);
    }

    #[test]
    fn encode_signed() {
        assert_eq!(ToLeb128i::to_leb128i(0i8), vec![0x00]);
        assert_eq!(ToLeb128i::to_leb128i(63i8), vec![0x3F]);
        assert_eq!(ToLeb128i::to_leb128i(64i8), vec![0xC0, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(127i8), vec![0xFF, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(-64i8), vec![0x40]);
        assert_eq!(ToLeb128i::to_leb128i(-65i8), vec![0xBF, 0x7F]);

        assert_eq!(ToLeb128i::to_leb128i(0i16), vec![0x00]);
        assert_eq!(ToLeb128i::to_leb128i(63i16), vec![0x3F]);
        assert_eq!(ToLeb128i::to_leb128i(64i16), vec![0xC0, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(127i16), vec![0xFF, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(-64i16), vec![0x40]);
        assert_eq!(ToLeb128i::to_leb128i(-65i16), vec![0xBF, 0x7F]);

        assert_eq!(ToLeb128i::to_leb128i(0i32), vec![0x00]);
        assert_eq!(ToLeb128i::to_leb128i(63i32), vec![0x3F]);
        assert_eq!(ToLeb128i::to_leb128i(64i32), vec![0xC0, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(127i32), vec![0xFF, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(-64i32), vec![0x40]);
        assert_eq!(ToLeb128i::to_leb128i(-65i32), vec![0xBF, 0x7F]);

        assert_eq!(ToLeb128i::to_leb128i(0i64), vec![0x00]);
        assert_eq!(ToLeb128i::to_leb128i(63i64), vec![0x3F]);
        assert_eq!(ToLeb128i::to_leb128i(64i64), vec![0xC0, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(127i64), vec![0xFF, 0x00]);
        assert_eq!(ToLeb128i::to_leb128i(-64i64), vec![0x40]);
        assert_eq!(ToLeb128i::to_leb128i(-65i64), vec![0xBF, 0x7F]);
    }

    #[test]
    fn decode_unsigned() {
        for i in u8::MIN..=u8::MAX {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }

        for i in u16::MIN..=u16::MIN + 3 {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }
        for i in u16::MAX - 3..=u16::MAX {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }

        for i in u32::MIN..=u32::MIN + 3 {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }
        for i in u32::MAX - 3..=u32::MAX {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }

        for i in u64::MIN..=u64::MIN + 3 {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }
        for i in u64::MAX - 3..=u64::MAX {
            assert_eq!(FromLeb128u::from_leb128u(&ToLeb128u::to_leb128u(i)), Ok(i));
        }
    }

    #[test]
    fn decode_signed() {
        for i in i8::MIN..=i8::MAX {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }

        for i in i16::MIN..=i16::MIN + 3 {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }
        for i in i16::MAX - 3..=i16::MAX {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }

        for i in i32::MIN..=i32::MIN + 3 {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }
        for i in i32::MAX - 3..=i32::MAX {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }

        for i in i64::MIN..=i64::MIN + 3 {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }
        for i in i64::MAX - 3..=i64::MAX {
            assert_eq!(FromLeb128i::from_leb128i(&ToLeb128i::to_leb128i(i)), Ok(i));
        }
    }
}
