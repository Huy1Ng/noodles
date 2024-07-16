//! Variant record.

mod alternate_bases;
mod filters;
mod ids;
pub mod info;
mod reference_bases;
pub mod samples;

use std::io;

use noodles_core::Position;

pub use self::{
    alternate_bases::AlternateBases, filters::Filters, ids::Ids, info::Info,
    reference_bases::ReferenceBases, samples::Samples,
};
use crate::Header;

/// A variant record.
pub trait Record {
    /// Returns the reference sequence name.
    fn reference_sequence_name<'a, 'h: 'a>(&'a self, header: &'h Header) -> io::Result<&'a str>;

    /// Returns the variant start position.
    ///
    /// This position is 1-based, inclusive.
    fn variant_start(&self) -> Option<io::Result<Position>>;

    /// Returns the IDs.
    fn ids(&self) -> Box<dyn Ids + '_>;

    /// Returns the reference bases.
    fn reference_bases(&self) -> Box<dyn ReferenceBases + '_>;

    /// Returns the alternate bases.
    fn alternate_bases(&self) -> Box<dyn AlternateBases + '_>;

    /// Returns the quality scores.
    fn quality_score(&self) -> Option<io::Result<f32>>;

    /// Returns the filters.
    fn filters(&self) -> Box<dyn Filters + '_>;

    /// Return the info fields.
    fn info(&self) -> Box<dyn Info + '_>;

    /// Returns the samples.
    fn samples(&self) -> io::Result<Box<dyn Samples + '_>>;

    /// Returns the variant span.
    fn variant_span(&self, header: &Header) -> io::Result<usize> {
        let start = self.variant_start().transpose()?.unwrap_or(Position::MIN);
        let end = self.variant_end(header)?;
        Ok(usize::from(end) - usize::from(start) + 1)
    }

    /// Returns or calculates the variant end position.
    ///
    /// If available, this returns the value of the `END` INFO field. Otherwise, it is calculated
    /// using the [variant start position] and [reference bases length].
    ///
    /// This position is 1-based, inclusive.
    ///
    /// [variant start position]: `Self::variant_start`
    /// [reference bases length]: `ReferenceBases::len`
    fn variant_end(&self, header: &Header) -> io::Result<Position> {
        use self::info::field::{key, Value};

        if let Some(Some(value)) = self.info().get(header, key::END_POSITION).transpose()? {
            match value {
                Value::Integer(n) => {
                    usize::try_from(n)
                        .and_then(Position::try_from)
                        .map_err(|_| {
                            io::Error::new(io::ErrorKind::InvalidData, "invalid INFO END position")
                        })
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid INFO END position value",
                )),
            }
        } else {
            let start = self.variant_start().transpose()?.unwrap_or(Position::MIN);
            let reference_bases = self.reference_bases();

            if reference_bases.is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid reference bases length",
                ))
            } else {
                let len = reference_bases.len();
                start
                    .checked_add(len - 1)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "position overflow"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::{record::info::field::key, record_buf::info::field::Value, RecordBuf};

    #[test]
    fn test_variant_span() -> io::Result<()> {
        let header = Header::default();

        let record = RecordBuf::builder()
            .set_info(
                [(String::from(key::END_POSITION), Some(Value::from(8)))]
                    .into_iter()
                    .collect(),
            )
            .build();

        assert_eq!(record.variant_span(&header)?, 8);

        let record = RecordBuf::builder().set_reference_bases("ACGT").build();
        assert_eq!(record.variant_span(&header)?, 4);

        Ok(())
    }

    #[test]
    fn test_variant_end() -> Result<(), Box<dyn std::error::Error>> {
        let header = Header::default();

        let record = RecordBuf::builder()
            .set_info(
                [(String::from(key::END_POSITION), Some(Value::from(8)))]
                    .into_iter()
                    .collect(),
            )
            .build();

        assert_eq!(
            Record::variant_end(&record, &header)?,
            Position::try_from(8)?
        );

        let record = RecordBuf::builder().set_reference_bases("ACGT").build();
        assert_eq!(
            Record::variant_end(&record, &header)?,
            Position::try_from(4)?
        );

        Ok(())
    }
}
