pub mod ty;

use std::{error, fmt, str};

pub use self::ty::read_type;
use crate::record::codec::{
    Value,
    value::{Array, Float, Int8, Int16, Int32, Type},
};

pub fn read_value<'a>(src: &mut &'a [u8]) -> Result<Option<Value<'a>>, DecodeError> {
    let ty = read_type(src).map_err(DecodeError::InvalidType)?;

    match ty {
        None => Ok(None),
        Some(Type::Int8(0)) => Ok(Some(Value::Int8(None))),
        Some(Type::Int8(1)) => read_i8_value(src),
        Some(Type::Int8(n)) => read_i8s_value(src, n),
        Some(Type::Int16(0)) => Ok(Some(Value::Int16(None))),
        Some(Type::Int16(1)) => read_i16_value(src),
        Some(Type::Int16(n)) => read_i16s_value(src, n),
        Some(Type::Int32(0)) => Ok(Some(Value::Int32(None))),
        Some(Type::Int32(1)) => read_i32_value(src),
        Some(Type::Int32(n)) => read_i32s_value(src, n),
        Some(Type::Float(0)) => Ok(Some(Value::Float(None))),
        Some(Type::Float(1)) => read_f32_value(src),
        Some(Type::Float(n)) => read_f32s_value(src, n),
        Some(Type::String(0)) => Ok(Some(Value::String(None))),
        Some(Type::String(n)) => read_string_value(src, n),
    }
}

fn read_i8_value<'a>(src: &mut &'a [u8]) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_i8;

    let n = read_i8(src).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Int8(Some(Int8::from(n)))))
}

fn read_i8s_value<'a>(src: &mut &'a [u8], len: usize) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_i8s;

    let values = read_i8s(src, len).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Array(Array::Int8(Box::new(values)))))
}

fn read_i16_value<'a>(src: &mut &'a [u8]) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_i16;

    let n = read_i16(src).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Int16(Some(Int16::from(n)))))
}

fn read_i16s_value<'a>(src: &mut &'a [u8], len: usize) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_i16s;

    let values = read_i16s(src, len).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Array(Array::Int16(Box::new(values)))))
}

fn read_i32_value<'a>(src: &mut &'a [u8]) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_i32;

    let n = read_i32(src).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Int32(Some(Int32::from(n)))))
}

fn read_i32s_value<'a>(src: &mut &'a [u8], len: usize) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_i32s;

    let values = read_i32s(src, len).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Array(Array::Int32(Box::new(values)))))
}

fn read_f32_value<'a>(src: &mut &'a [u8]) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_f32;

    let n = read_f32(src).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Float(Some(Float::from(n)))))
}

fn read_f32s_value<'a>(src: &mut &'a [u8], len: usize) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_f32s;

    let values = read_f32s(src, len).map_err(DecodeError::InvalidRawValue)?;
    Ok(Some(Value::Array(Array::Float(Box::new(values)))))
}

fn read_string_value<'a>(src: &mut &'a [u8], len: usize) -> Result<Option<Value<'a>>, DecodeError> {
    use super::raw_value::read_string;

    let buf = read_string(src, len).map_err(DecodeError::InvalidRawValue)?;
    let s = str::from_utf8(buf).map_err(DecodeError::InvalidString)?;
    Ok(Some(Value::String(Some(s))))
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    InvalidType(ty::DecodeError),
    InvalidRawValue(super::raw_value::DecodeError),
    InvalidString(str::Utf8Error),
}

impl error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidType(e) => Some(e),
            Self::InvalidRawValue(e) => Some(e),
            Self::InvalidString(e) => Some(e),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(_) => write!(f, "invalid type"),
            Self::InvalidRawValue(_) => write!(f, "invalid raw value"),
            Self::InvalidString(_) => write!(f, "invalid string"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn test_read_value() -> io::Result<()> {
        let mut src = &[0x00][..];
        assert!(matches!(read_value(&mut src), Ok(None)));

        let mut src = &[0x01][..];
        assert!(matches!(read_value(&mut src), Ok(Some(Value::Int8(None)))));

        let mut src = &[0x11, 0x05][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::Int8(Some(Int8::Value(5)))))
        ));

        let mut src = &[0x31, 0x05, 0x08, 0x0d][..];
        match read_value(&mut src) {
            Ok(Some(Value::Array(Array::Int8(values)))) => {
                assert_eq!(values.iter().collect::<Result<Vec<_>, _>>()?, [5, 8, 13]);
            }
            _ => panic!(),
        }

        let mut src = &[0x02][..];
        assert!(matches!(read_value(&mut src), Ok(Some(Value::Int16(None)))));

        let mut src = &[0x12, 0x79, 0x01][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::Int16(Some(Int16::Value(377)))))
        ));

        let mut src = &[0x32, 0x79, 0x01, 0x62, 0x02, 0xdb, 0x03][..];
        match read_value(&mut src) {
            Ok(Some(Value::Array(Array::Int16(values)))) => {
                assert_eq!(
                    values.iter().collect::<Result<Vec<_>, _>>()?,
                    [377, 610, 987]
                );
            }
            _ => panic!(),
        }

        let mut src = &[0x03][..];
        assert!(matches!(read_value(&mut src), Ok(Some(Value::Int32(None)))));

        let mut src = &[0x13, 0x11, 0x25, 0x01, 0x00][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::Int32(Some(Int32::Value(75025)))))
        ));

        let mut src = &[
            0x33, 0x11, 0x25, 0x01, 0x00, 0x31, 0xda, 0x01, 0x00, 0x42, 0xff, 0x02, 0x00,
        ][..];
        match read_value(&mut src) {
            Ok(Some(Value::Array(Array::Int32(values)))) => {
                assert_eq!(
                    values.iter().collect::<Result<Vec<_>, _>>()?,
                    [75025, 121393, 196418]
                );
            }
            _ => panic!(),
        }

        let mut src = &[0x05][..];
        assert!(matches!(read_value(&mut src), Ok(Some(Value::Float(None)))));

        let mut src = &[0x15, 0x00, 0x00, 0x00, 0x00][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::Float(Some(n)))) if n == Float::from(0.0)
        ));

        let mut src = &[0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f][..];
        match read_value(&mut src) {
            Ok(Some(Value::Array(Array::Float(values)))) => {
                assert_eq!(values.iter().collect::<Result<Vec<_>, _>>()?, [0.0, 0.5]);
            }
            _ => panic!(),
        }

        let mut src = &[0x07][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::String(None)))
        ));

        let mut src = &[0x17, b'n'][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::String(Some("n"))))
        ));

        let mut src = &[0x47, b'n', b'd', b'l', b's'][..];
        assert!(matches!(
            read_value(&mut src),
            Ok(Some(Value::String(Some("ndls"))))
        ));

        Ok(())
    }
}
