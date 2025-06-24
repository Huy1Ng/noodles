/// Variant record IDs.
pub trait Ids {
    /// Returns whether there are any IDs.
    fn is_empty(&self) -> bool;

    /// Returns the number of IDs.
    fn len(&self) -> usize;

    /// Returns an iterator over IDs.
    fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_>;
}

impl Ids for Box<dyn Ids + '_> {
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        (**self).iter()
    }
}
