use std::{io, marker::PhantomData, mem};

use noodles_vcf as vcf;

use crate::record::codec::value::{Float, Int8, Int16, Int32};

pub(crate) struct Values<'a, N> {
    src: &'a [u8],
    _marker: PhantomData<N>,
}

impl<'a, N> Values<'a, N> {
    pub(crate) fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            _marker: PhantomData,
        }
    }
}

impl<N> AsRef<[u8]> for Values<'_, N> {
    fn as_ref(&self) -> &[u8] {
        self.src
    }
}

impl Values<'_, i8> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = Int8> + '_ {
        self.src.iter().map(|&n| Int8::from(n as i8))
    }
}

impl<'a> vcf::variant::record::info::field::value::array::Values<'a, i32> for Values<'a, i8> {
    fn len(&self) -> usize {
        self.src.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<i32>>> + '_> {
        Box::new(self.iter().map(|value| match value {
            Int8::Value(n) => Ok(Some(i32::from(n))),
            Int8::Missing => Ok(None),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }))
    }
}

impl<'a> vcf::variant::record::samples::series::value::array::Values<'a, i32> for Values<'a, i8> {
    fn len(&self) -> usize {
        self.src.len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<i32>>> + '_> {
        Box::new(self.iter().filter_map(|value| match value {
            Int8::Value(n) => Some(Ok(Some(i32::from(n)))),
            Int8::Missing => Some(Ok(None)),
            Int8::EndOfVector => None,
            Int8::Reserved(_) => Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
        }))
    }
}

impl Values<'_, i16> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = Int16> + '_ {
        self.src.chunks_exact(mem::size_of::<i16>()).map(|chunk| {
            // SAFETY: `chunk` is 2 bytes.
            let n = i16::from_le_bytes(chunk.try_into().unwrap());
            Int16::from(n)
        })
    }
}

impl<'a> vcf::variant::record::info::field::value::array::Values<'a, i32> for Values<'a, i16> {
    fn len(&self) -> usize {
        self.src.len() / mem::size_of::<i16>()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<i32>>> + '_> {
        Box::new(self.iter().map(|value| match value {
            Int16::Value(n) => Ok(Some(i32::from(n))),
            Int16::Missing => Ok(None),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }))
    }
}

impl<'a> vcf::variant::record::samples::series::value::array::Values<'a, i32> for Values<'a, i16> {
    fn len(&self) -> usize {
        self.src.len() / mem::size_of::<i16>()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<i32>>> + '_> {
        Box::new(self.iter().filter_map(|value| match value {
            Int16::Value(n) => Some(Ok(Some(i32::from(n)))),
            Int16::Missing => Some(Ok(None)),
            Int16::EndOfVector => None,
            Int16::Reserved(_) => Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
        }))
    }
}

impl Values<'_, i32> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = Int32> + '_ {
        self.src.chunks_exact(mem::size_of::<i32>()).map(|chunk| {
            // SAFETY: `chunk` is 4 bytes.
            let n = i32::from_le_bytes(chunk.try_into().unwrap());
            Int32::from(n)
        })
    }
}

impl<'a> vcf::variant::record::info::field::value::array::Values<'a, i32> for Values<'a, i32> {
    fn len(&self) -> usize {
        self.src.len() / mem::size_of::<i32>()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<i32>>> + '_> {
        Box::new(self.iter().map(|value| match value {
            Int32::Value(n) => Ok(Some(n)),
            Int32::Missing => Ok(None),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }))
    }
}

impl<'a> vcf::variant::record::samples::series::value::array::Values<'a, i32> for Values<'a, i32> {
    fn len(&self) -> usize {
        self.src.len() / mem::size_of::<i32>()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<i32>>> + '_> {
        Box::new(self.iter().filter_map(|value| match value {
            Int32::Value(n) => Some(Ok(Some(n))),
            Int32::Missing => Some(Ok(None)),
            Int32::EndOfVector => None,
            Int32::Reserved(_) => Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
        }))
    }
}

impl Values<'_, f32> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = Float> + '_ {
        self.src.chunks_exact(mem::size_of::<f32>()).map(|chunk| {
            // SAFETY: `chunk` is 4 bytes.
            let n = f32::from_le_bytes(chunk.try_into().unwrap());
            Float::from(n)
        })
    }
}

impl<'a> vcf::variant::record::info::field::value::array::Values<'a, f32> for Values<'a, f32> {
    fn len(&self) -> usize {
        self.src.len() / mem::size_of::<f32>()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<f32>>> + '_> {
        Box::new(self.iter().map(|value| match value {
            Float::Value(n) => Ok(Some(n)),
            Float::Missing => Ok(None),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }))
    }
}

impl<'a> vcf::variant::record::samples::series::value::array::Values<'a, f32> for Values<'a, f32> {
    fn len(&self) -> usize {
        self.src.len() / mem::size_of::<f32>()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Option<f32>>> + '_> {
        Box::new(self.iter().filter_map(|value| match value {
            Float::Value(n) => Some(Ok(Some(n))),
            Float::Missing => Some(Ok(None)),
            Float::EndOfVector => None,
            Float::Reserved(_) => Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
        }))
    }
}
