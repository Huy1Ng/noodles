use std::io;

use crate::{
    container::{
        block,
        compression_header::encoding::{Decode, Encode},
    },
    huffman::CanonicalHuffmanDecoder,
    io::{
        BitReader, BitWriter,
        reader::{container::slice::records::ExternalDataReaders, num::read_itf8},
        writer::{container::slice::records::ExternalDataWriters, num::write_itf8},
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Integer {
    External {
        block_content_id: block::ContentId,
    },
    Golomb {
        offset: i32,
        m: i32,
    },
    Huffman {
        alphabet: Vec<i32>,
        bit_lens: Vec<u32>,
    },
    Beta {
        offset: i32,
        len: u32,
    },
    Subexp {
        offset: i32,
        k: i32,
    },
    GolombRice {
        offset: i32,
        log2_m: i32,
    },
    Gamma {
        offset: i32,
    },
}

impl<'de> Decode<'de> for Integer {
    type Value = i32;

    fn decode(
        &self,
        core_data_reader: &mut BitReader<'de>,
        external_data_readers: &mut ExternalDataReaders<'de>,
    ) -> io::Result<Self::Value> {
        match self {
            Self::External { block_content_id } => {
                let src = external_data_readers
                    .get_mut(block_content_id)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("missing external block: {block_content_id}"),
                        )
                    })?;

                read_itf8(src)
            }
            Self::Huffman { alphabet, bit_lens } => {
                if alphabet.len() == 1 {
                    Ok(alphabet[0])
                } else {
                    let decoder = CanonicalHuffmanDecoder::new(alphabet, bit_lens);
                    decoder.decode(core_data_reader)
                }
            }
            Self::Beta { offset, len } => core_data_reader.read_i32(*len).map(|i| i - offset),
            Self::Gamma { offset } => {
                let mut n = 0;

                while core_data_reader.read_bit()? == 0 {
                    n += 1;
                }

                let m = core_data_reader.read_i32(n)?;
                let x = (1 << n) + m;

                Ok(x - offset)
            }
            _ => todo!("decode_itf8: {:?}", self),
        }
    }
}

impl Encode<'_> for Integer {
    type Value = i32;

    fn encode(
        &self,
        _core_data_writer: &mut BitWriter,
        external_data_writers: &mut ExternalDataWriters,
        value: Self::Value,
    ) -> io::Result<()> {
        match self {
            Self::External { block_content_id } => {
                let dst = external_data_writers
                    .get_mut(block_content_id)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("missing external block: {block_content_id}"),
                        )
                    })?;

                write_itf8(dst, value)
            }
            _ => todo!("encode_itf8: {:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::compression_header::Encoding;

    #[test]
    fn test_decode() -> io::Result<()> {
        fn t(
            core_data: Option<&[u8]>,
            encoding: &Encoding<Integer>,
            expected: i32,
        ) -> io::Result<()> {
            let core_data = core_data.unwrap_or(&[0b10000000]);
            let mut core_data_reader = BitReader::new(core_data);

            let external_data = [0x0d];
            let mut external_data_readers = ExternalDataReaders::new();
            external_data_readers.insert(1, &external_data[..]);

            let actual = encoding.decode(&mut core_data_reader, &mut external_data_readers)?;

            assert_eq!(expected, actual);

            Ok(())
        }

        t(
            None,
            &Encoding::new(Integer::External {
                block_content_id: 1,
            }),
            13,
        )?;
        t(
            None,
            &Encoding::new(Integer::Huffman {
                alphabet: vec![0x4e],
                bit_lens: vec![0],
            }),
            0x4e,
        )?;
        t(None, &Encoding::new(Integer::Beta { offset: 1, len: 3 }), 3)?;
        t(
            Some(&[0b00011010]),
            &Encoding::new(Integer::Gamma { offset: 5 }),
            8,
        )?;

        Ok(())
    }

    #[test]
    fn test_encode() -> io::Result<()> {
        fn t(
            encoding: &Encoding<Integer>,
            value: i32,
            expected_core_data: &[u8],
            expected_external_data: &[u8],
        ) -> io::Result<()> {
            let mut core_data_writer = BitWriter::default();

            let block_content_id = 1;
            let mut external_data_writers = [(block_content_id, Vec::new())].into_iter().collect();

            encoding.encode(&mut core_data_writer, &mut external_data_writers, value)?;

            let actual_core_data = core_data_writer.finish()?;
            assert_eq!(actual_core_data, expected_core_data);

            let actual_external_data = &external_data_writers[&block_content_id];
            assert_eq!(actual_external_data, expected_external_data);

            Ok(())
        }

        t(
            &Encoding::new(Integer::External {
                block_content_id: 1,
            }),
            0x0d,
            &[],
            &[0x0d],
        )?;

        Ok(())
    }
}
