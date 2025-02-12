//! Indexed CRAM reader.

mod builder;

pub use self::builder::Builder;

use std::io::{self, Read, Seek};

use noodles_core::Region;
use noodles_fasta as fasta;
use noodles_sam as sam;

use super::{
    reader::{Container, Query, Records},
    Reader,
};
use crate::{crai, FileDefinition};

/// An indexed CRAM reader.
pub struct IndexedReader<R> {
    inner: Reader<R>,
    index: crai::Index,
}

impl<R> IndexedReader<R>
where
    R: Read,
{
    /// Creates an indexed CRAM reader.
    pub fn new(inner: R, index: crai::Index) -> Self {
        Self {
            inner: Reader::new(inner),
            index,
        }
    }

    /// Returns a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut()
    }

    /// Unwraps and returns the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }

    /// Returns the reference sequence repository.
    pub fn reference_sequence_repository(&self) -> &fasta::Repository {
        self.inner.reference_sequence_repository()
    }

    /// Reads the CRAM file definition.
    pub fn read_file_definition(&mut self) -> io::Result<FileDefinition> {
        self.inner.read_file_definition()
    }

    /// Reads the SAM header.
    pub fn read_file_header(&mut self) -> io::Result<sam::Header> {
        self.inner.read_file_header()
    }

    /// Reads the SAM header.
    pub fn read_header(&mut self) -> io::Result<sam::Header> {
        self.inner.read_header()
    }

    /// Reads a container.
    pub fn read_container(&mut self, container: &mut Container) -> io::Result<usize> {
        self.inner.read_container(container)
    }

    /// Reads a container.
    #[deprecated(
        since = "0.78.0",
        note = "Use `IndexedReader::read_container` instead."
    )]
    pub fn read_data_container(&mut self) -> io::Result<Option<Container>> {
        #[allow(deprecated)]
        self.inner.read_data_container()
    }

    /// Returns a iterator over records starting from the current stream position.
    pub fn records<'r>(&'r mut self, header: &'r sam::Header) -> Records<'r, R> {
        self.inner.records(header)
    }

    /// Returns the associated index.
    pub fn index(&self) -> &crai::Index {
        &self.index
    }
}

impl<R> IndexedReader<R>
where
    R: Read + Seek,
{
    /// Returns an iterator over records that intersects the given region.
    pub fn query<'a>(
        &'a mut self,
        header: &'a sam::Header,
        region: &Region,
    ) -> io::Result<Query<'a, R>> {
        self.inner.query(header, &self.index, region)
    }
}
