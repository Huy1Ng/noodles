//! Async CRAM header reader.

pub mod container;
mod file_id;
mod format_version;
mod magic_number;

use noodles_sam as sam;
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncRead, BufReader};

use self::{
    file_id::read_file_id, format_version::read_format_version, magic_number::read_magic_number,
};
use crate::{file_definition::Version, FileDefinition, MAGIC_NUMBER};

/// A CRAM header reader.
pub struct Reader<R> {
    inner: R,
}

impl<R> Reader<R>
where
    R: AsyncRead + Unpin,
{
    pub(super) fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Reads the magic number.
    pub async fn read_magic_number(&mut self) -> io::Result<[u8; MAGIC_NUMBER.len()]> {
        read_magic_number(&mut self.inner).await
    }

    /// Reads the format version.
    pub async fn read_format_version(&mut self) -> io::Result<Version> {
        read_format_version(&mut self.inner).await
    }

    /// Reads the file ID.
    pub async fn read_file_id(&mut self) -> io::Result<[u8; 20]> {
        read_file_id(&mut self.inner).await
    }

    /// Returns a header container reader.
    pub async fn container_reader(&mut self) -> io::Result<container::Reader<&mut R>> {
        let len = container::read_header(&mut self.inner).await?;
        Ok(container::Reader::new(&mut self.inner, len))
    }
}

pub(super) async fn read_header<R>(reader: &mut R) -> io::Result<sam::Header>
where
    R: AsyncRead + Unpin,
{
    read_file_definition(reader).await?;
    read_file_header(reader).await
}

pub(super) async fn read_file_definition<R>(reader: &mut R) -> io::Result<FileDefinition>
where
    R: AsyncRead + Unpin,
{
    let mut header_reader = Reader::new(reader);
    read_file_definition_inner(&mut header_reader).await
}

async fn read_file_definition_inner<R>(reader: &mut Reader<R>) -> io::Result<FileDefinition>
where
    R: AsyncRead + Unpin,
{
    use crate::io::reader::header::magic_number;

    reader
        .read_magic_number()
        .await
        .and_then(magic_number::validate)?;

    let version = reader.read_format_version().await?;
    let file_id = reader.read_file_id().await?;

    Ok(FileDefinition::new(version, file_id))
}

pub(super) async fn read_file_header<R>(reader: &mut R) -> io::Result<sam::Header>
where
    R: AsyncRead + Unpin,
{
    let mut header_reader = Reader::new(reader);
    read_file_header_inner(&mut header_reader).await
}

async fn read_file_header_inner<R>(reader: &mut Reader<R>) -> io::Result<sam::Header>
where
    R: AsyncRead + Unpin,
{
    let mut container_reader = reader.container_reader().await?;

    let header = {
        let mut raw_sam_header_reader = container_reader.raw_sam_header_reader().await?;
        let header = read_sam_header(&mut raw_sam_header_reader).await?;
        raw_sam_header_reader.discard_to_end().await?;
        header
    };

    container_reader.discard_to_end().await?;

    Ok(header)
}

async fn read_sam_header<R>(reader: &mut R) -> io::Result<sam::Header>
where
    R: AsyncRead + Unpin,
{
    let mut parser = sam::header::Parser::default();

    let mut header_reader = BufReader::new(reader);
    let mut buf = Vec::new();

    while read_line(&mut header_reader, &mut buf).await? != 0 {
        parser
            .parse_partial(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    }

    Ok(parser.finish())
}

async fn read_line<R>(reader: &mut R, dst: &mut Vec<u8>) -> io::Result<usize>
where
    R: AsyncBufRead + Unpin,
{
    const LINE_FEED: u8 = b'\n';
    const CARRIAGE_RETURN: u8 = b'\r';

    dst.clear();

    match reader.read_until(LINE_FEED, dst).await? {
        0 => Ok(0),
        n => {
            if dst.ends_with(&[LINE_FEED]) {
                dst.pop();

                if dst.ends_with(&[CARRIAGE_RETURN]) {
                    dst.pop();
                }
            }

            Ok(n)
        }
    }
}
