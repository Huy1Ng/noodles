use std::{error, fmt};

use bstr::BString;

use crate::header::parser::record::split_off_first;

/// An error returned when a SAM header record comment value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The delimiter is invalid.
    InvalidDelimiter,
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDelimiter => write!(f, "invalid delimiter"),
        }
    }
}

pub(super) fn parse_comment(src: &mut &[u8]) -> Result<BString, ParseError> {
    consume_delimiter(src)?;
    let (buf, rest) = src.split_at(src.len());
    *src = rest;
    Ok(buf.into())
}

fn consume_delimiter(src: &mut &[u8]) -> Result<(), ParseError> {
    const PREFIX: u8 = b'\t';

    if let Some(&PREFIX) = split_off_first(src) {
        Ok(())
    } else {
        Err(ParseError::InvalidDelimiter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comment() {
        let mut src = &b"\tnoodles"[..];
        assert_eq!(parse_comment(&mut src), Ok(BString::from("noodles")));

        let mut src = &b"noodles"[..];
        assert_eq!(parse_comment(&mut src), Err(ParseError::InvalidDelimiter));
    }
}
