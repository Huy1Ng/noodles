//! Prints BAM index statistics.
//!
//! The data is read from both the SAM header reference sequences and BAM index. It is printed as a
//! tab-delimited record with the following columns for a region: reference sequence name,
//! reference sequence length, number of mapped records, and number of unmapped records.
//!
//! The result matches the output of `samtools idxstats <src>`.

use std::{env, io, path::PathBuf};

use noodles_bam as bam;

fn main() -> io::Result<()> {
    let src = env::args().nth(1).map(PathBuf::from).expect("missing src");

    let mut reader = bam::io::indexed_reader::Builder::default().build_from_path(src)?;
    let header = reader.read_header()?;

    let index = reader.index();

    for ((reference_sequence_name, reference_sequence), index_reference_sequence) in header
        .reference_sequences()
        .iter()
        .zip(index.reference_sequences())
    {
        let (mapped_record_count, unmapped_record_count) = index_reference_sequence
            .metadata()
            .map(|m| (m.mapped_record_count(), m.unmapped_record_count()))
            .unwrap_or_default();

        println!(
            "{}\t{}\t{}\t{}",
            reference_sequence_name,
            reference_sequence.length(),
            mapped_record_count,
            unmapped_record_count
        );
    }

    let unmapped_record_count = index.unplaced_unmapped_record_count().unwrap_or_default();
    println!("*\t0\t0\t{unmapped_record_count}");

    Ok(())
}
