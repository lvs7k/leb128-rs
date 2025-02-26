use std::io::{self, Read, Write};

pub trait ToLeb128u {
    fn to_leb128u(&self, writer: &mut impl Write) -> io::Result<usize>;
}

macro_rules! impl_to_leb128u {
    ($($ty:ty),*) => {
        $(
            impl ToLeb128u for $ty {
                fn to_leb128u(&self, writer: &mut impl Write) -> io::Result<usize> {
                    let mut value = *self;
                    let mut count = 0;

                    loop {
                        let byte = (value & 0b01111111) as u8;
                        value >>= 7;

                        if value == 0 {
                            count += writer.write(&[byte])?;
                            break;
                        }

                        count += writer.write(&[byte | 0b10000000])?;
                    }

                    Ok(count)
                }
            }
        )*
    };
}

impl_to_leb128u!(u8, u16, u32, u64);

pub trait ToLeb128i {
    fn to_leb128i(&self, writer: &mut impl Write) -> io::Result<usize>;
}

macro_rules! impl_to_leb128i {
    ($($ty:ty),*) => {
        $(
            impl ToLeb128i for $ty {
                fn to_leb128i(&self, writer: &mut impl Write) -> io::Result<usize> {
                    let mut value = *self;
                    let mut count = 0;

                    loop {
                        let byte = (value & 0b01111111) as u8;
                        value >>= 7;

                        if value == 0 && (byte & 0b01000000) == 0 || value == -1 && (byte & 0b01000000) != 0 {
                            count += writer.write(&[byte])?;
                            break;
                        }

                        count += writer.write(&[byte | 0b10000000])?;
                    }

                    Ok(count)
                }
            }
        )*
    };
}

impl_to_leb128i!(i8, i16, i32, i64);

#[derive(Debug)]
pub enum FromLeb128Error {
    Malformed,
    Io(io::Error),
}

impl std::fmt::Display for FromLeb128Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromLeb128Error::Malformed => write!(f, "malformed bytes"),
            FromLeb128Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for FromLeb128Error {}

impl From<io::Error> for FromLeb128Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub trait FromLeb128u {
    fn from_leb128u(reader: &mut impl Read) -> Result<Self, FromLeb128Error>
    where
        Self: Sized;
}

macro_rules! impl_from_leb128u {
    ($($ty:ty),*) => {
        $(
            impl FromLeb128u for $ty {
                fn from_leb128u(reader: &mut impl Read) -> Result<Self, FromLeb128Error> {
                    let bit = std::mem::size_of::<$ty>() * 8;
                    let mut result = 0;
                    let mut shift = 0;
                    let mut buf = [0; 1];

                    loop {
                        reader.read_exact(&mut buf)?;
                        let b = (buf[0] & 0b01111111) as $ty;

                        if (shift >= bit - (bit % 7)) && (b >= (1 << (bit % 7))) {
                            return Err(FromLeb128Error::Malformed);
                        }

                        result |= b << shift;
                        shift += 7;

                        if buf[0] & 0b10000000 == 0 {
                            break;
                        }
                    }

                    Ok(result)
                }
            }
        )*
    };
}

impl_from_leb128u!(u8, u16, u32, u64);

pub trait FromLeb128i {
    fn from_leb128i(reader: &mut impl Read) -> Result<Self, FromLeb128Error>
    where
        Self: Sized;
}

macro_rules! impl_from_leb128i {
    ($($ty:ty),*) => {
        $(
            impl FromLeb128i for $ty {
                fn from_leb128i(reader: &mut impl Read) -> Result<Self, FromLeb128Error> {
                    let bit = std::mem::size_of::<$ty>() * 8;
                    let mut result = 0;
                    let mut shift = 0;
                    let mut buf = [0; 1];

                    loop {
                        reader.read_exact(&mut buf)?;
                        let b = (buf[0] & 0b01111111) as $ty;

                        if shift >= bit - (bit % 7) {
                            let is_positive = (b & 0b01000000) == 0;

                            if is_positive {
                                if b >= (1 << (bit % 7)) {
                                    return Err(FromLeb128Error::Malformed);
                                }
                            } else {
                                let mask = (!0 << (bit % 7)) & 0b01111111;
                                if b & mask != mask {
                                    return Err(FromLeb128Error::Malformed);
                                }
                            }
                        }

                        result |= b << shift;
                        shift += 7;

                        if buf[0] & 0b10000000 == 0 {
                            let is_negative = (b & 0b01000000) != 0;

                            if is_negative && shift <= bit {
                                result |= !0 << shift;
                            }
                            break;
                        }
                    }

                    Ok(result)
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
    fn to_leb_128u() {
        let mut buf = Vec::new();

        buf.clear();
        assert_eq!(127u32.to_leb128u(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0x7f]);

        buf.clear();
        assert_eq!(128u32.to_leb128u(&mut buf).unwrap(), 2);
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn to_leb_128i() {
        let mut buf = Vec::new();

        buf.clear();
        assert_eq!(63i32.to_leb128i(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0x3f]);

        buf.clear();
        assert_eq!(64i32.to_leb128i(&mut buf).unwrap(), 2);
        assert_eq!(buf, vec![0xc0, 0x00]);

        buf.clear();
        assert_eq!((-64i32).to_leb128i(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0x40]);

        buf.clear();
        assert_eq!((-65i32).to_leb128i(&mut buf).unwrap(), 2);
        assert_eq!(buf, vec![0xbf, 0x7f]);
    }

    #[test]
    fn from_leb_128u() {
        let mut buf = Vec::new();

        for i in 0..=u8::MAX {
            buf.clear();
            i.to_leb128u(&mut buf).unwrap();
            assert_eq!(i, u8::from_leb128u(&mut &buf[..]).unwrap());
        }

        for i in 0..=u16::MAX {
            buf.clear();
            i.to_leb128u(&mut buf).unwrap();
            assert_eq!(i, u16::from_leb128u(&mut &buf[..]).unwrap());
        }
    }

    #[test]
    fn from_leb_128i() {
        let mut buf = Vec::new();

        for i in i8::MIN..=i8::MAX {
            buf.clear();
            i.to_leb128i(&mut buf).unwrap();
            assert_eq!(i, i8::from_leb128i(&mut &buf[..]).unwrap());
        }

        for i in i16::MIN..=i16::MAX {
            buf.clear();
            i.to_leb128i(&mut buf).unwrap();
            assert_eq!(i, i16::from_leb128i(&mut &buf[..]).unwrap());
        }
    }
}
