mod field;

use std::{fmt, io, iter};

use self::field::parse_field;

/// Raw GFF record attributes.
pub struct Attributes<'a>(&'a str);

impl<'a> Attributes<'a> {
    pub(super) fn new(buf: &'a str) -> Self {
        Self(buf)
    }

    /// Returns whether there are any attributes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the value of the given tag.
    pub fn get(&self, tag: &str) -> Option<io::Result<&str>> {
        for result in self.iter() {
            match result {
                Ok((t, value)) => {
                    if t == tag {
                        return Some(Ok(value));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }

    /// Returns an iterator over all tag-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = io::Result<(&str, &str)>> {
        let mut src = self.0;

        iter::from_fn(move || {
            if src.is_empty() {
                None
            } else {
                Some(parse_field(&mut src))
            }
        })
    }
}

impl<'a> AsRef<str> for Attributes<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> fmt::Debug for Attributes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut formatter = f.debug_map();

        for result in self.iter() {
            let (tag, value) = result.map_err(|_| fmt::Error)?;
            formatter.entry(&tag, &value);
        }

        formatter.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty() {
        let attributes = Attributes::new("");
        assert!(attributes.is_empty());

        let attributes = Attributes::new("gene_id=ndls0;gene_name=gene0");
        assert!(!attributes.is_empty());
    }

    #[test]
    fn test_get() {
        let attributes = Attributes::new("gene_id=ndls0;gene_name=gene0");
        assert!(attributes.get("gene_name").is_some());
        assert!(attributes.get("comment").is_none());
    }

    #[test]
    fn test_iter() -> io::Result<()> {
        let attributes = Attributes::new("");
        assert!(attributes.iter().next().is_none());

        let attributes = Attributes::new("gene_id=ndls0;gene_name=gene0");
        let actual: Vec<_> = attributes.iter().collect::<Result<_, _>>()?;
        let expected = vec![("gene_id", "ndls0"), ("gene_name", "gene0")];
        assert_eq!(actual, expected);

        Ok(())
    }
}
