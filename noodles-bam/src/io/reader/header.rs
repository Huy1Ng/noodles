mod reference_sequences;

use std::io::{self, BufRead, BufReader, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use noodles_sam::{self as sam, header::ReferenceSequences};

use self::reference_sequences::read_reference_sequences;
use crate::MAGIC_NUMBER;

pub(super) fn read_header<R>(reader: &mut R) -> io::Result<sam::Header>
where
    R: Read,
{
    read_magic(reader)?;

    let mut header = read_header_inner(reader)?;
    let reference_sequences = read_reference_sequences(reader)?;

    if header.reference_sequences().is_empty() {
        *header.reference_sequences_mut() = reference_sequences;
    } else if !reference_sequences_eq(header.reference_sequences(), &reference_sequences) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "SAM header and binary reference sequence dictionaries mismatch",
        ));
    }

    Ok(header)
}

fn read_magic<R>(reader: &mut R) -> io::Result<()>
where
    R: Read,
{
    let mut magic = [0; 4];
    reader.read_exact(&mut magic)?;

    if magic == MAGIC_NUMBER {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid BAM header",
        ))
    }
}

fn read_header_inner<R>(reader: &mut R) -> io::Result<sam::Header>
where
    R: Read,
{
    let l_text = reader.read_u32::<LittleEndian>().map(u64::from)?;

    let mut parser = sam::header::Parser::default();

    let mut header_reader = BufReader::new(reader.take(l_text));
    let mut buf = Vec::new();

    while read_line(&mut header_reader, &mut buf)? != 0 {
        parser
            .parse_partial(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    }

    discard_padding(&mut header_reader)?;

    Ok(parser.finish())
}

fn read_line<R>(reader: &mut R, dst: &mut Vec<u8>) -> io::Result<usize>
where
    R: BufRead,
{
    const NUL: u8 = 0x00;
    const LINE_FEED: u8 = b'\n';
    const CARRIAGE_RETURN: u8 = b'\r';

    dst.clear();

    let src = reader.fill_buf()?;

    if src.is_empty() || src[0] == NUL {
        return Ok(0);
    }

    match reader.read_until(LINE_FEED, dst)? {
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

fn discard_padding<R>(reader: &mut R) -> io::Result<()>
where
    R: BufRead,
{
    loop {
        let src = reader.fill_buf()?;

        if src.is_empty() {
            return Ok(());
        }

        let len = src.len();
        reader.consume(len);
    }
}

pub(crate) fn reference_sequences_eq(
    header_reference_sequences: &ReferenceSequences,
    binary_reference_sequences: &ReferenceSequences,
) -> bool {
    header_reference_sequences.len() == binary_reference_sequences.len()
        && header_reference_sequences
            .iter()
            .zip(binary_reference_sequences)
            .all(|((h_name, h_map), (b_name, b_map))| {
                h_name == b_name && h_map.length() == b_map.length()
            })
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use bytes::BufMut;
    use noodles_sam::{
        self as sam,
        header::record::value::{
            map::{self, header::Version},
            Map,
        },
    };

    use super::*;

    const SQ0_LN: NonZeroUsize = match NonZeroUsize::new(8) {
        Some(length) => length,
        None => unreachable!(),
    };

    #[test]
    fn test_read_magic() -> io::Result<()> {
        let data = b"BAM\x01";
        let mut reader = &data[..];
        assert!(read_magic(&mut reader).is_ok());

        let data = [];
        let mut reader = &data[..];
        assert!(matches!(
            read_magic(&mut reader),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        let data = b"MThd";
        let mut reader = &data[..];
        assert!(matches!(
            read_magic(&mut reader),
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData
        ));

        Ok(())
    }

    #[test]
    fn test_read_header() -> io::Result<()> {
        let mut data = Vec::new();
        data.put_slice(MAGIC_NUMBER); // magic
        data.put_u32_le(27); // l_text
        data.put_slice(b"@HD\tVN:1.6\n@SQ\tSN:sq0\tLN:8\n"); // text
        data.put_u32_le(1); // n_ref
        data.put_u32_le(4); // ref[0].l_name
        data.put_slice(b"sq0\x00"); // ref[0].name
        data.put_u32_le(8); // ref[0].l_ref

        let mut reader = &data[..];
        let actual = read_header(&mut reader)?;

        let expected = sam::Header::builder()
            .set_header(Map::<map::Header>::new(Version::new(1, 6)))
            .add_reference_sequence("sq0", Map::<map::ReferenceSequence>::new(SQ0_LN))
            .build();

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_read_header_with_trailing_nul_padding_in_text() -> io::Result<()> {
        let mut raw_header = b"@HD\tVN:1.6\n".to_vec();
        raw_header.resize(16384, 0);

        let mut data = Vec::new();
        data.put_slice(MAGIC_NUMBER); // magic
        data.put_u32_le(16384); // l_text
        data.put_slice(&raw_header); // text
        data.put_u32_le(1); // n_ref
        data.put_u32_le(4); // ref[0].l_name
        data.put_slice(b"sq0\x00"); // ref[0].name
        data.put_u32_le(8); // ref[0].l_ref

        let mut reader = &data[..];
        let actual = read_header(&mut reader)?;

        let expected = sam::Header::builder()
            .set_header(Map::<map::Header>::new(Version::new(1, 6)))
            .add_reference_sequence("sq0", Map::<map::ReferenceSequence>::new(SQ0_LN))
            .build();

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_read_header_with_missing_sam_header_reference_sequence_dictionary() -> io::Result<()> {
        let mut data = Vec::new();
        data.put_slice(MAGIC_NUMBER); // magic
        data.put_u32_le(11); // l_text
        data.put_slice(b"@HD\tVN:1.6\n"); // text
        data.put_u32_le(1); // n_ref
        data.put_u32_le(4); // ref[0].l_name
        data.put_slice(b"sq0\x00"); // ref[0].name
        data.put_u32_le(8); // ref[0].l_ref

        let mut reader = &data[..];
        let actual = read_header(&mut reader)?;

        let expected = sam::Header::builder()
            .set_header(Map::<map::Header>::new(Version::new(1, 6)))
            .add_reference_sequence("sq0", Map::<map::ReferenceSequence>::new(SQ0_LN))
            .build();

        assert_eq!(actual, expected);

        Ok(())
    }
}
