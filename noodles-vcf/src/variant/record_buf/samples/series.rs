use std::io;

use super::sample::Value;
use crate::Header;

/// A variant record samples buffer series.
pub struct Series<'s> {
    name: &'s str,
    values: &'s [Vec<Option<Value>>],
    i: usize,
}

impl<'s> Series<'s> {
    pub(super) fn new(name: &'s str, values: &'s [Vec<Option<Value>>], i: usize) -> Self {
        Self { name, values, i }
    }

    /// Returns the value at the given index.
    pub fn get(&self, i: usize) -> Option<Option<&Value>> {
        self.values
            .get(i)
            .map(|sample| sample.get(self.i).and_then(|value| value.as_ref()))
    }
}

impl crate::variant::record::samples::Series for Series<'_> {
    fn name<'a, 'h: 'a>(&'a self, _: &'h Header) -> io::Result<&'a str> {
        Ok(self.name)
    }

    fn get<'a, 'h: 'a>(
        &'a self,
        _: &'h Header,
        i: usize,
    ) -> Option<Option<io::Result<crate::variant::record::samples::series::Value<'a>>>> {
        self.values.get(i).map(|sample| {
            sample
                .get(self.i)
                .and_then(|value| value.as_ref().map(|v| Ok(v.into())))
        })
    }

    fn iter<'a, 'h: 'a>(
        &'a self,
        _: &'h Header,
    ) -> Box<
        dyn Iterator<Item = io::Result<Option<crate::variant::record::samples::series::Value<'a>>>>
            + 'a,
    > {
        Box::new(self.values.iter().map(|sample| {
            Ok(sample
                .get(self.i)
                .and_then(|value| value.as_ref().map(|v| v.into())))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::record::samples::{keys::key, Series as _};

    #[test]
    fn test_name() {
        let header = Header::default();
        let series = Series::new(key::GENOTYPE, &[], 0);
        assert!(matches!(
            series.name(&header),
            Ok(name) if name == key::GENOTYPE
        ));
    }

    #[test]
    fn test_get() {
        let values = [
            vec![Some(Value::from("0|0")), Some(Value::from(7))],
            vec![Some(Value::from("1/1"))],
            vec![],
        ];

        let series = Series::new(key::GENOTYPE, &values, 0);
        assert_eq!(series.get(0), Some(Some(&Value::from("0|0"))));
        assert_eq!(series.get(1), Some(Some(&Value::from("1/1"))));
        assert_eq!(series.get(2), Some(None));
        assert_eq!(series.get(3), None);

        let series = Series::new(key::CONDITIONAL_GENOTYPE_QUALITY, &values, 1);
        assert_eq!(series.get(0), Some(Some(&Value::from(7))));
        assert_eq!(series.get(1), Some(None));
        assert_eq!(series.get(2), Some(None));
        assert_eq!(series.get(3), None);
    }
}
