//! Binning index builder.

use super::{Header, Index, ReferenceSequence, reference_sequence};

/// A binning index builder.
pub struct Builder<I> {
    min_shift: u8,
    depth: u8,
    header: Option<Header>,
    reference_sequences: Vec<ReferenceSequence<I>>,
    unplaced_unmapped_record_count: Option<u64>,
}

impl<I> Builder<I>
where
    I: reference_sequence::Index,
{
    /// Sets a min shift.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::{self as csi, BinningIndex};
    /// let index = csi::Index::builder().set_min_shift(13).build();
    /// assert_eq!(index.min_shift(), 13);
    /// ```
    pub fn set_min_shift(mut self, min_shift: u8) -> Self {
        self.min_shift = min_shift;
        self
    }

    /// Sets a max depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::{self as csi, BinningIndex};
    /// let index = csi::Index::builder().set_depth(8).build();
    /// assert_eq!(index.depth(), 8);
    /// ```
    pub fn set_depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }

    /// Sets a tabix header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::{self as csi, binning_index::index::Header, BinningIndex};
    /// let header = Header::default();
    /// let index = csi::Index::builder().set_header(header.clone()).build();
    /// assert_eq!(index.header(), Some(&header));
    /// ```
    pub fn set_header(mut self, header: Header) -> Self {
        self.header = Some(header);
        self
    }

    /// Sets reference sequences.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::{self as csi, binning_index::index::ReferenceSequence};
    ///
    /// let reference_sequences = vec![ReferenceSequence::new(Default::default(), Default::default(), None)];
    /// let index = csi::Index::builder()
    ///     .set_reference_sequences(reference_sequences.clone())
    ///     .build();
    ///
    /// assert_eq!(index.reference_sequences(), &reference_sequences);
    /// ```
    pub fn set_reference_sequences(
        mut self,
        reference_sequences: Vec<ReferenceSequence<I>>,
    ) -> Self {
        self.reference_sequences = reference_sequences;
        self
    }

    /// Sets an unplaced, unmapped record count.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi::{self as csi, BinningIndex};
    /// let index = csi::Index::builder().set_unplaced_unmapped_record_count(21).build();
    /// assert_eq!(index.unplaced_unmapped_record_count(), Some(21));
    /// ```
    pub fn set_unplaced_unmapped_record_count(
        mut self,
        unplaced_unmapped_record_count: u64,
    ) -> Self {
        self.unplaced_unmapped_record_count = Some(unplaced_unmapped_record_count);
        self
    }

    /// Builds a binning index.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_csi as csi;
    /// let index = csi::Index::builder().build();
    /// ```
    pub fn build(self) -> Index<I> {
        Index {
            min_shift: self.min_shift,
            depth: self.depth,
            header: self.header,
            reference_sequences: self.reference_sequences,
            unplaced_unmapped_record_count: self.unplaced_unmapped_record_count,
        }
    }
}

impl<I> Default for Builder<I> {
    fn default() -> Self {
        Self {
            min_shift: 14,
            depth: 5,
            header: None,
            reference_sequences: Vec::new(),
            unplaced_unmapped_record_count: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binning_index::index::reference_sequence::index::BinnedIndex;

    #[test]
    fn test_default() {
        let builder = Builder::<BinnedIndex>::default();

        assert_eq!(builder.min_shift, 14);
        assert_eq!(builder.depth, 5);
        assert!(builder.header.is_none());
        assert!(builder.reference_sequences.is_empty());
        assert!(builder.unplaced_unmapped_record_count.is_none());
    }
}
