use std::io;

use noodles_bgzf as bgzf;
use noodles_core::region::Interval;
use noodles_csi::{self as csi, binning_index::index::reference_sequence::bin::Chunk};

use super::Reader;
use crate::{variant::Record as _, Header, Record};

/// An iterator over records of a VCF reader that intersects a given region.
///
/// This is created by calling [`Reader::query`].
pub struct Query<'r, 'h, R> {
    reader: Reader<csi::io::Query<'r, R>>,
    reference_sequence_name: Vec<u8>,
    interval: Interval,
    header: &'h Header,
    record: Record,
}

impl<'r, 'h, R> Query<'r, 'h, R>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    pub(super) fn new(
        reader: &'r mut R,
        chunks: Vec<Chunk>,
        reference_sequence_name: Vec<u8>,
        interval: Interval,
        header: &'h Header,
    ) -> Self {
        Self {
            reader: Reader::new(csi::io::Query::new(reader, chunks)),
            reference_sequence_name,
            interval,
            header,
            record: Record::default(),
        }
    }
}

impl<R> Iterator for Query<'_, '_, R>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    type Item = io::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        match next_record(
            &mut self.reader,
            &mut self.record,
            self.header,
            &self.reference_sequence_name,
            self.interval,
        ) {
            Ok(0) => None,
            Ok(_) => Some(Ok(self.record.clone())),
            Err(e) => Some(Err(e)),
        }
    }
}

pub(crate) fn intersects(
    header: &Header,
    record: &Record,
    reference_sequence_name: &[u8],
    region_interval: Interval,
) -> io::Result<bool> {
    let name = record.reference_sequence_name();

    let Some(start) = record.variant_start().transpose()? else {
        return Ok(false);
    };

    let end = record.variant_end(header)?;
    let record_interval = Interval::from(start..=end);

    Ok(name.as_bytes() == reference_sequence_name && record_interval.intersects(region_interval))
}

fn next_record<R>(
    reader: &mut Reader<csi::io::Query<'_, R>>,
    record: &mut Record,
    header: &Header,
    reference_sequence_name: &[u8],
    interval: Interval,
) -> io::Result<usize>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    loop {
        match reader.read_record(record)? {
            0 => return Ok(0),
            n => {
                if intersects(header, record, reference_sequence_name, interval)? {
                    return Ok(n);
                }
            }
        }
    }
}
