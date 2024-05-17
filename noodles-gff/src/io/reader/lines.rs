use std::io::{self, BufRead};

use crate::Line;

use super::Reader;

/// An iterator over lines of a GFF reader.
///
/// When using this, the caller is responsible to stop reading at either EOF or when the `FASTA`
/// directive is read, whichever comes first.
///
/// This is created by calling [`Reader::lines`].
pub struct Lines<'a, R> {
    inner: &'a mut Reader<R>,
    line_buf: String,
}

impl<'a, R> Lines<'a, R>
where
    R: BufRead,
{
    pub(crate) fn new(inner: &'a mut Reader<R>) -> Self {
        Self {
            inner,
            line_buf: String::new(),
        }
    }
}

impl<'a, R> Iterator for Lines<'a, R>
where
    R: BufRead,
{
    type Item = io::Result<Line>;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_buf.clear();

        match self.inner.read_line(&mut self.line_buf) {
            Ok(0) => None,
            Ok(_) => match self.line_buf.parse() {
                Ok(line) => Some(Ok(line)),
                Err(e) => Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
            },
            Err(e) => Some(Err(e)),
        }
    }
}
