use std::{io, iter};

use super::Keys;
use crate::{
    io::reader::record_buf::value::percent_decode,
    variant::record::samples::series::{value::Array, Value},
    Header,
};

/// A VCF record samples sample.
#[derive(Debug, Eq, PartialEq)]
pub struct Sample<'s> {
    src: &'s str,
    keys: Keys<'s>,
}

impl<'s> Sample<'s> {
    pub(super) fn new(src: &'s str, keys: Keys<'s>) -> Self {
        Self { src, keys }
    }

    /// Returns the value at the given index.
    pub fn get_index<'h: 's>(
        &self,
        header: &'h Header,
        i: usize,
    ) -> Option<Option<io::Result<Value<'s>>>> {
        self.values(header).nth(i)
    }

    /// Returns an iterator over values.
    pub fn values<'h: 's>(
        &self,
        header: &'h Header,
    ) -> impl Iterator<Item = Option<io::Result<Value<'s>>>> + '_ {
        self.iter(header)
            .map(|result| result.map(|(_, value)| value).transpose())
    }

    /// Returns an iterator over fields.
    pub fn iter<'h: 's>(
        &self,
        header: &'h Header,
    ) -> Box<dyn Iterator<Item = io::Result<(&str, Option<Value<'s>>)>> + '_> {
        const DELIMITER: char = ':';

        if self.as_ref().is_empty() {
            Box::new(iter::empty())
        } else {
            Box::new(
                self.keys
                    .iter()
                    .zip(self.src.split(DELIMITER))
                    .map(|(key, s)| parse_value(s, header, key).map(|value| (key, value))),
            )
        }
    }
}

impl<'a> AsRef<str> for Sample<'a> {
    fn as_ref(&self) -> &str {
        self.src
    }
}

impl<'r> crate::variant::record::samples::Sample for Sample<'r> {
    fn get<'a, 'h: 'a>(
        &'a self,
        header: &'h Header,
        key: &str,
    ) -> Option<io::Result<Option<Value<'a>>>> {
        for result in self.iter(header) {
            match result {
                Ok((k, v)) => {
                    if k == key {
                        return Some(Ok(v));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }

    fn get_index<'a, 'h: 'a>(
        &'a self,
        header: &'h Header,
        i: usize,
    ) -> Option<io::Result<Option<Value<'a>>>> {
        self.iter(header)
            .nth(i)
            .map(|result| result.map(|(_, value)| value))
    }

    fn iter<'a, 'h: 'a>(
        &'a self,
        header: &'h Header,
    ) -> Box<dyn Iterator<Item = io::Result<(&str, Option<Value<'a>>)>> + 'a> {
        Box::new(self.iter(header))
    }
}

fn parse_value<'a>(src: &'a str, header: &Header, key: &str) -> io::Result<Option<Value<'a>>> {
    use crate::{
        header::record::value::map::format::{definition::definition, Number, Type},
        variant::record::samples::keys::key,
    };

    const MISSING: &str = ".";

    if src == MISSING {
        return Ok(None);
    } else if key == key::GENOTYPE {
        return parse_genotype_value(src).map(Some);
    }

    let (number, ty) = header
        .formats()
        .get(key)
        .map(|format| (format.number(), format.ty()))
        .or_else(|| definition(header.file_format(), key).map(|(n, t, _)| (n, t)))
        .unwrap_or_default();

    let value = match (number, ty) {
        (Number::Count(0), _) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid number for type",
            ))
        }
        (Number::Count(1), Type::Integer) => parse_integer_value(src)?,
        (Number::Count(1), Type::Float) => parse_float_value(src)?,
        (Number::Count(1), Type::Character) => parse_character_value(src)?,
        (Number::Count(1), Type::String) => parse_string_value(src)?,
        (_, Type::Integer) => parse_integer_array_value(src)?,
        (_, Type::Float) => parse_float_array_value(src)?,
        (_, Type::Character) => parse_character_array_value(src)?,
        (_, Type::String) => parse_string_array_value(src)?,
    };

    Ok(Some(value))
}

fn parse_integer_value(src: &str) -> io::Result<Value<'_>> {
    src.parse()
        .map(Value::Integer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn parse_float_value(src: &str) -> io::Result<Value<'_>> {
    src.parse()
        .map(Value::Float)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn parse_character_value(src: &str) -> io::Result<Value<'_>> {
    let s = percent_decode(src).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut chars = s.chars();

    if let Some(c) = chars.next() {
        if chars.next().is_none() {
            return Ok(Value::Character(c));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid character",
    ))
}

fn parse_string_value(src: &str) -> io::Result<Value<'_>> {
    percent_decode(src)
        .map(Value::String)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn parse_genotype_value(src: &str) -> io::Result<Value<'_>> {
    use super::series::value::Genotype;

    Ok(Value::Genotype(Box::new(Genotype::new(src))))
}

fn parse_integer_array_value(src: &str) -> io::Result<Value<'_>> {
    Ok(Value::Array(Array::Integer(Box::new(src))))
}

fn parse_float_array_value(src: &str) -> io::Result<Value<'_>> {
    Ok(Value::Array(Array::Float(Box::new(src))))
}

fn parse_character_array_value(src: &str) -> io::Result<Value<'_>> {
    Ok(Value::Array(Array::Character(Box::new(src))))
}

fn parse_string_array_value(src: &str) -> io::Result<Value<'_>> {
    Ok(Value::Array(Array::String(Box::new(src))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values() {
        let header = Header::default();

        let keys = Keys::new("GT:GQ");
        let sample = Sample::new("0|0:.", keys);
        let mut iter = sample.values(&header);

        assert!(matches!(iter.next(), Some(Some(Ok(Value::Genotype(_))))));
        assert!(matches!(iter.next(), Some(None)));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter() {
        let header = Header::default();

        let keys = Keys::new("GT:GQ");
        let sample = Sample::new("0|0:.", keys);
        let mut iter = sample.iter(&header);

        assert!(matches!(
            iter.next(),
            Some(Ok(("GT", Some(Value::Genotype(_)))))
        ));

        assert!(matches!(iter.next(), Some(Ok(("GQ", None)))));

        assert!(iter.next().is_none());
    }
}
