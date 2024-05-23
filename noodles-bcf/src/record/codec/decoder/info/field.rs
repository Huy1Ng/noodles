mod value;

use std::{error, fmt};

use noodles_vcf as vcf;

use self::value::read_value;
use crate::record::codec::decoder::string_map::{self, read_string_map_entry};

pub(crate) fn read_field(
    src: &mut &[u8],
    header: &vcf::Header,
) -> Result<(String, Option<vcf::variant::record_buf::info::field::Value>), DecodeError> {
    let raw_key = read_string_map_entry(src, header.string_maps().strings())
        .map_err(DecodeError::InvalidStringMap)?;

    let (key, info) = header
        .infos()
        .get_key_value(raw_key)
        .ok_or(DecodeError::MissingInfoMapEntry)?;

    let value = read_value(src, info.number(), info.ty()).map_err(DecodeError::InvalidValue)?;

    Ok((key.clone(), value))
}

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    InvalidStringMap(string_map::DecodeError),
    MissingInfoMapEntry,
    InvalidValue(value::DecodeError),
}

impl error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidStringMap(e) => Some(e),
            Self::MissingInfoMapEntry => None,
            Self::InvalidValue(e) => Some(e),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStringMap(_) => write!(f, "invalid string map"),
            Self::MissingInfoMapEntry => write!(f, "missing info map entry"),
            Self::InvalidValue(_) => write!(f, "invalid value"),
        }
    }
}
