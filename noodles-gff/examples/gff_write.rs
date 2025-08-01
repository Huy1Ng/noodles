//! Creates a new GFF file.
//!
//! This writes a GFF version directive and one (sparse) record to stdout.

use std::io;

use bstr::BString;
use noodles_gff::{
    self as gff, LineBuf,
    directive_buf::{Value, key},
};

fn main() -> io::Result<()> {
    let stdout = io::stdout().lock();
    let mut writer = gff::io::Writer::new(stdout);

    let version = gff::DirectiveBuf::new(
        key::GFF_VERSION,
        Some(Value::GffVersion(Default::default())),
    );

    writer.write_directive(&version)?;

    let comment = LineBuf::Comment(BString::from("format: gff3"));
    writer.write_line(&comment)?;

    let record = gff::feature::RecordBuf::default();
    writer.write_record(&record)?;

    Ok(())
}
