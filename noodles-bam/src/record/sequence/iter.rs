use std::{array, iter::FusedIterator, slice};

use super::decode_base;

/// A BAM record sequence bases iterator.
pub(super) struct Iter<'a> {
    iter: slice::Iter<'a, u8>,
    base_count: usize,
    front: Option<array::IntoIter<u8, 2>>,
    back: Option<array::IntoIter<u8, 2>>,
}

impl<'a> Iter<'a> {
    pub(super) fn new(bases: &'a [u8], base_count: usize) -> Self {
        let mut iter = bases.iter();

        // This assumes `bases.len() * 2` is only ever `base_count` or `base_count` + 1.
        let back = if bases.len() * 2 > base_count {
            iter.next_back().map(|&n| discard_back_decoded_bases(n))
        } else {
            None
        };

        Self {
            iter,
            base_count,
            front: None,
            back,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(base) = self.front.as_mut().and_then(|iter| iter.next()) {
                return Some(base);
            }

            self.front = match self.iter.next() {
                Some(n) => Some(decoded_bases(*n)),
                None => return self.back.as_mut().and_then(|iter| iter.next()),
            };
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.base_count, Some(self.base_count))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(base) = self.back.as_mut().and_then(|iter| iter.next_back()) {
                return Some(base);
            }

            self.back = match self.iter.next_back() {
                Some(n) => Some(decoded_bases(*n)),
                None => return self.front.as_mut().and_then(|iter| iter.next()),
            };
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> FusedIterator for Iter<'a> {}

pub(super) fn decoded_bases(n: u8) -> array::IntoIter<u8, 2> {
    [decode_base(n >> 4), decode_base(n)].into_iter()
}

fn discard_back_decoded_bases(n: u8) -> array::IntoIter<u8, 2> {
    let mut bases = decoded_bases(n);
    bases.next_back();
    bases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        let mut iter = Iter::new(&[], 0);
        assert!(iter.next().is_none());

        let mut iter = Iter::new(&[0x12, 0x40], 3);
        assert_eq!(iter.next(), Some(b'A'));
        assert_eq!(iter.next(), Some(b'C'));
        assert_eq!(iter.next(), Some(b'G'));
        assert!(iter.next().is_none());

        let mut iter = Iter::new(&[0x12, 0x48], 4);
        assert_eq!(iter.next(), Some(b'A'));
        assert_eq!(iter.next(), Some(b'C'));
        assert_eq!(iter.next(), Some(b'G'));
        assert_eq!(iter.next(), Some(b'T'));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_size_hint() {
        let iter = Iter::new(&[], 4);
        assert_eq!(iter.size_hint(), (4, Some(4)));
    }

    #[test]
    fn test_next_back() {
        let mut iter = Iter::new(&[], 0);
        assert!(iter.next_back().is_none());

        let mut iter = Iter::new(&[0x12, 0x40], 3);
        assert_eq!(iter.next_back(), Some(b'G'));
        assert_eq!(iter.next_back(), Some(b'C'));
        assert_eq!(iter.next_back(), Some(b'A'));
        assert!(iter.next_back().is_none());

        let mut iter = Iter::new(&[0x12, 0x48], 4);
        assert_eq!(iter.next_back(), Some(b'T'));
        assert_eq!(iter.next_back(), Some(b'G'));
        assert_eq!(iter.next_back(), Some(b'C'));
        assert_eq!(iter.next_back(), Some(b'A'));
        assert!(iter.next_back().is_none());
    }
}
