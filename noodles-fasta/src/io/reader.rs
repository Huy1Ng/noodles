//! FASTA reader and iterators.

mod builder;
mod definition;
mod records;
pub mod sequence;

pub use self::{builder::Builder, records::Records};

use std::{
    io::{self, BufRead, Seek, SeekFrom},
    ops::Range,
};

use noodles_core::{region::Interval, Region};

use self::definition::read_definition;
use crate::{fai, Record};

pub(crate) const DEFINITION_PREFIX: u8 = b'>';

/// A FASTA reader.
pub struct Reader<R> {
    inner: R,
}

impl<R> Reader<R>
where
    R: BufRead,
{
    /// Creates a FASTA reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fasta as fasta;
    /// let data = b">sq0\nACGT\n>sq1\nNNNN\nNNNN\nNN\n";
    /// let mut reader = fasta::io::Reader::new(&data[..]);
    /// ```
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fasta as fasta;
    /// let reader = fasta::io::Reader::new(&[][..]);
    /// assert!(reader.get_ref().is_empty());
    /// ```
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Returns a mutable reference to the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fasta as fasta;
    /// let mut reader = fasta::io::Reader::new(&[][..]);
    /// assert!(reader.get_mut().is_empty());
    /// ```
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_fasta as fasta;
    /// let reader = fasta::io::Reader::new(&[][..]);
    /// assert!(reader.into_inner().is_empty());
    /// ```
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Reads a raw definition line.
    ///
    /// The given buffer will not include the trailing newline. It can subsequently be parsed as a
    /// [`crate::record::Definition`].
    ///
    /// The position of the stream is expected to be at the start or at the start of another
    /// definition.
    ///
    /// If successful, this returns the number of bytes read from the stream. If the number of
    /// bytes read is 0, the stream reached EOF.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_fasta as fasta;
    ///
    /// let data = b">sq0\nACGT\n>sq1\nNNNN\nNNNN\nNN\n";
    /// let mut reader = fasta::io::Reader::new(&data[..]);
    ///
    /// let mut buf = String::new();
    /// reader.read_definition(&mut buf)?;
    ///
    /// assert_eq!(buf, ">sq0");
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_definition(&mut self, buf: &mut String) -> io::Result<usize> {
        read_definition(&mut self.inner, buf)
    }

    /// Reads a sequence.
    ///
    /// The given buffer consumes a sequence without newlines until another definition or EOF is
    /// reached.
    ///
    /// The position of the stream is expected to be at the start of a sequence, which is directly
    /// after a definition.
    ///
    /// If successful, this returns the number of bases read from the stream. If the number of
    /// bases read is 0, the stream reached EOF (though this case is likely an error).
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_fasta as fasta;
    ///
    /// let data = b">sq0\nACGT\n>sq1\nNNNN\nNNNN\nNN\n";
    /// let mut reader = fasta::io::Reader::new(&data[..]);
    /// reader.read_definition(&mut String::new())?;
    ///
    /// let mut buf = Vec::new();
    /// reader.read_sequence(&mut buf)?;
    ///
    /// assert_eq!(buf, b"ACGT");
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn read_sequence(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        use self::sequence::read_sequence;
        read_sequence(&mut self.inner, buf)
    }

    /// Returns a sequence reader.
    ///
    /// A [`sequence::Reader`] can be used for lower-level reading of the raw sequence.
    ///
    /// The position of the stream is expected to be at the start of a sequence to read a full
    /// sequence or within a sequence to read a partial one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{self, Read};
    /// use noodles_fasta as fasta;
    ///
    /// let data = b">sq0\nACGT\n>sq1\nNNNN\nNNNN\nNN\n";
    /// let mut reader = fasta::io::Reader::new(&data[..]);
    /// reader.read_definition(&mut String::new())?;
    ///
    /// let mut sequence_reader = reader.sequence_reader();
    /// let mut buf = vec![0; 2];
    /// sequence_reader.read_exact(&mut buf)?;
    ///
    /// assert_eq!(buf, b"AC");
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn sequence_reader(&mut self) -> sequence::Reader<'_, R> {
        sequence::Reader::new(self.get_mut())
    }

    /// Returns an iterator over records starting from the current stream position.
    ///
    /// The position of the stream is expected to be at the start or at the start of another
    /// definition.
    ///
    /// ```
    /// # use std::io;
    /// use noodles_fasta::{self as fasta, record::{Definition, Sequence}};
    ///
    /// let data = b">sq0\nACGT\n>sq1\nNNNN\nNNNN\nNN\n";
    /// let mut reader = fasta::io::Reader::new(&data[..]);
    ///
    /// let mut records = reader.records();
    ///
    /// assert_eq!(records.next().transpose()?, Some(fasta::Record::new(
    ///     Definition::new("sq0", None),
    ///     Sequence::from(b"ACGT".to_vec()),
    /// )));
    ///
    /// assert_eq!(records.next().transpose()?, Some(fasta::Record::new(
    ///     Definition::new("sq1", None),
    ///     Sequence::from(b"NNNNNNNNNN".to_vec()),
    /// )));
    ///
    /// assert!(records.next().is_none());
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn records(&mut self) -> Records<'_, R> {
        Records::new(self)
    }
}

impl<R> Reader<R>
where
    R: BufRead + Seek,
{
    /// Returns a record of the given region.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// use noodles_core::Region;
    /// use noodles_fasta::{self as fasta, fai, record::{Definition, Sequence}};
    ///
    /// let data = b">sq0\nNNNN\n>sq1\nACGT\n>sq2\nNNNN\n";
    /// let index = vec![
    ///     fai::Record::new("sq0", 4, 5, 4, 5),
    ///     fai::Record::new("sq1", 4, 15, 4, 5),
    ///     fai::Record::new("sq2", 4, 25, 4, 5),
    /// ];
    ///
    /// let mut reader = fasta::io::Reader::new(Cursor::new(data));
    ///
    /// let region = Region::new("sq1", ..);
    /// let record = reader.query(&index, &region)?;
    /// assert_eq!(record, fasta::Record::new(
    ///     Definition::new("sq1", None),
    ///     Sequence::from(b"ACGT".to_vec()),
    /// ));
    ///
    /// let region = "sq1:2-3".parse()?;
    /// let record = reader.query(&index, &region)?;
    /// assert_eq!(record, fasta::Record::new(
    ///     Definition::new("sq1:2-3", None),
    ///     Sequence::from(b"CG".to_vec()),
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn query(&mut self, index: &fai::Index, region: &Region) -> io::Result<Record> {
        use self::sequence::read_sequence_limit;
        use crate::record::{Definition, Sequence};

        let index_record = index
            .iter()
            .find(|record| record.name() == region.name())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "invalid reference sequence name: {}",
                        String::from_utf8_lossy(region.name())
                    ),
                )
            })?;

        let seq_len = index_record.length() as usize;
        let range = interval_to_slice_range(region.interval(), seq_len);

        let pos = index_record.offset()
            + range.start as u64 / index_record.line_bases() * index_record.line_width()
            + range.start as u64 % index_record.line_bases();
        self.get_mut().seek(SeekFrom::Start(pos))?;

        let definition = Definition::new(region.to_string(), None);

        let mut raw_sequence = Vec::new();
        read_sequence_limit(&mut self.inner, range.len(), &mut raw_sequence)?;

        let sequence = Sequence::from(raw_sequence);

        Ok(Record::new(definition, sequence))
    }
}

// Reads all bytes until a line feed ('\n') or EOF is reached.
//
// The buffer will not include the trailing newline ('\n' or '\r\n').
pub(crate) fn read_line<R>(reader: &mut R, buf: &mut String) -> io::Result<usize>
where
    R: BufRead,
{
    const LINE_FEED: char = '\n';
    const CARRIAGE_RETURN: char = '\r';

    match reader.read_line(buf) {
        Ok(0) => Ok(0),
        Ok(n) => {
            if buf.ends_with(LINE_FEED) {
                buf.pop();

                if buf.ends_with(CARRIAGE_RETURN) {
                    buf.pop();
                }
            }

            Ok(n)
        }
        Err(e) => Err(e),
    }
}

// Shifts a 1-based interval to a 0-based range for slicing.
fn interval_to_slice_range<I>(interval: I, len: usize) -> Range<usize>
where
    I: Into<Interval>,
{
    let interval = interval.into();

    let start = interval
        .start()
        .map(|position| usize::from(position) - 1)
        .unwrap_or(usize::MIN);

    let end = interval.end().map(usize::from).unwrap_or(len);

    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_definition() -> io::Result<()> {
        let data = b">sq0\nACGT\n";
        let mut reader = Reader::new(&data[..]);

        let mut description_buf = String::new();
        reader.read_definition(&mut description_buf)?;

        assert_eq!(description_buf, ">sq0");

        Ok(())
    }

    #[test]
    fn test_read_line() -> io::Result<()> {
        let mut buf = String::new();

        let data = b"noodles\n";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, "noodles");

        let data = b"noodles\r\n";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, "noodles");

        let data = b"noodles";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, "noodles");

        Ok(())
    }

    #[test]
    fn test_interval_to_slice_range() -> Result<(), noodles_core::position::TryFromIntError> {
        use noodles_core::Position;

        const LENGTH: usize = 21;

        let start = Position::try_from(8)?;
        let end = Position::try_from(13)?;

        assert_eq!(interval_to_slice_range(start..=end, LENGTH), 7..13);
        assert_eq!(interval_to_slice_range(start.., LENGTH), 7..21);
        assert_eq!(interval_to_slice_range(..=end, LENGTH), 0..13);
        assert_eq!(interval_to_slice_range(.., LENGTH), 0..21);

        Ok(())
    }
}
