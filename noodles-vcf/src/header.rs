//! VCF header and fields.

mod builder;
pub mod file_format;
pub mod parser;
pub mod record;
pub mod string_maps;

pub use self::{
    builder::Builder, file_format::FileFormat, parser::ParseError, parser::Parser, record::Record,
    string_maps::StringMaps,
};

use std::{hash::Hash, str::FromStr};

use indexmap::{IndexMap, IndexSet};

use self::record::value::{
    Map,
    map::{AlternativeAllele, Contig, Filter, Format, Info},
};

/// VCF header info records.
pub type Infos = IndexMap<String, Map<Info>>;

/// VCF header filter records.
pub type Filters = IndexMap<String, Map<Filter>>;

/// VCF header format records.
pub type Formats = IndexMap<String, Map<Format>>;

/// VCF header alternative allele records.
pub type AlternativeAlleles = IndexMap<String, Map<AlternativeAllele>>;

/// VCF header contig records.
pub type Contigs = IndexMap<String, Map<Contig>>;

/// VCF header sample names.
pub type SampleNames = IndexSet<String>;

/// VCF header generic records.
pub type OtherRecords = IndexMap<record::key::Other, record::value::Collection>;

/// A VCF header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    file_format: FileFormat,
    infos: Infos,
    filters: Filters,
    formats: Formats,
    alternative_alleles: AlternativeAlleles,
    contigs: Contigs,
    sample_names: SampleNames,
    other_records: OtherRecords,
    string_maps: StringMaps,
}

impl Header {
    /// Returns a builder to create a record from each of its fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf as vcf;
    /// let builder = vcf::Header::builder();
    /// ```
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the file format (`fileformat`) of the VCF.
    ///
    /// `fileformat` is a required meta record and is guaranteed to be set.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::FileFormat};
    ///
    /// let header = vcf::Header::builder()
    ///     .set_file_format(FileFormat::default())
    ///     .build();
    ///
    /// assert_eq!(header.file_format(), FileFormat::default());
    /// ```
    pub fn file_format(&self) -> FileFormat {
        self.file_format
    }

    /// Returns a mutable reference to the file format (`fileformat`) of the VCF.
    ///
    /// `fileformat` is a required meta record and is guaranteed to be set.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::FileFormat};
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let file_format = FileFormat::new(4, 2);
    /// *header.file_format_mut() = file_format;
    ///
    /// assert_eq!(header.file_format(), file_format);
    /// ```
    pub fn file_format_mut(&mut self) -> &mut FileFormat {
        &mut self.file_format
    }

    /// Returns a map of information records (`INFO`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::value::{map::Info, Map},
    ///     variant::record::info::field::key,
    /// };
    ///
    /// let id = key::SAMPLES_WITH_DATA_COUNT;
    /// let info = Map::<Info>::from(id);
    ///
    /// let header = vcf::Header::builder()
    ///     .add_info(id, info.clone())
    ///     .build();
    ///
    /// let infos = header.infos();
    /// assert_eq!(infos.len(), 1);
    /// assert_eq!(&infos[0], &info);
    /// ```
    pub fn infos(&self) -> &Infos {
        &self.infos
    }

    /// Returns a mutable reference to a map of information records (`INFO`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::value::{map::Info, Map},
    ///     variant::record::info::field::key,
    /// };
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let id = key::SAMPLES_WITH_DATA_COUNT;
    /// let info = Map::<Info>::from(id);
    /// header.infos_mut().insert(id.into(), info.clone());
    ///
    /// let infos = header.infos();
    /// assert_eq!(infos.len(), 1);
    /// assert_eq!(&infos[0], &info);
    /// ```
    pub fn infos_mut(&mut self) -> &mut Infos {
        &mut self.infos
    }

    /// Returns a map of filter records (`FILTER`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::record::value::{map::Filter, Map}};
    ///
    /// let filter = Map::<Filter>::new("Quality below 10");
    ///
    /// let header = vcf::Header::builder()
    ///     .add_filter("q10", filter.clone())
    ///     .build();
    ///
    /// let filters = header.filters();
    /// assert_eq!(filters.len(), 1);
    /// assert_eq!(&filters[0], &filter);
    /// ```
    pub fn filters(&self) -> &Filters {
        &self.filters
    }

    /// Returns a mutable reference to a map of filter records (`FILTER`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::record::value::{map::Filter, Map}};
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let filter = Map::<Filter>::new("Quality below 10");
    /// header.filters_mut().insert(String::from("q10"), filter.clone());
    ///
    /// let filters = header.filters();
    /// assert_eq!(filters.len(), 1);
    /// assert_eq!(&filters[0], &filter);
    /// ```
    pub fn filters_mut(&mut self) -> &mut Filters {
        &mut self.filters
    }

    /// Returns a list of genotype format records (`FORMAT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::value::{map::Format, Map},
    ///     variant::record::samples::keys::key,
    /// };
    ///
    /// let id = key::GENOTYPE;
    /// let format = Map::<Format>::from(id);
    ///
    /// let header = vcf::Header::builder()
    ///     .add_format(id, format.clone())
    ///     .build();
    ///
    /// let formats = header.formats();
    /// assert_eq!(formats.len(), 1);
    /// assert_eq!(&formats[0], &format);
    /// ```
    pub fn formats(&self) -> &Formats {
        &self.formats
    }

    /// Returns a mutable reference to a list of genotype format records (`FORMAT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::value::{map::Format, Map},
    ///     variant::record::samples::keys::key,
    /// };
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let id = key::GENOTYPE;
    /// let format = Map::<Format>::from(id);
    /// header.formats_mut().insert(id.into(), format.clone());
    ///
    /// let formats = header.formats();
    /// assert_eq!(formats.len(), 1);
    /// assert_eq!(&formats[0], &format);
    /// ```
    pub fn formats_mut(&mut self) -> &mut Formats {
        &mut self.formats
    }

    /// Returns a map of symbolic alternate alleles (`ALT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::value::{map::AlternativeAllele, Map},
    /// };
    ///
    /// let alt = Map::<AlternativeAllele>::new("Deletion");
    ///
    /// let header = vcf::Header::builder()
    ///     .add_alternative_allele("DEL", alt.clone())
    ///     .build();
    ///
    /// let alternative_alleles = header.alternative_alleles();
    /// assert_eq!(alternative_alleles.len(), 1);
    /// assert_eq!(&alternative_alleles[0], &alt);
    /// ```
    pub fn alternative_alleles(&self) -> &AlternativeAlleles {
        &self.alternative_alleles
    }

    /// Returns a mutable reference to a map of symbolic alternate alleles (`ALT`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::value::{map::AlternativeAllele, Map},
    /// };
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let alt = Map::<AlternativeAllele>::new("Deletion");
    /// header.alternative_alleles_mut().insert(String::from("DEL"), alt.clone());
    ///
    /// let alternative_alleles = header.alternative_alleles();
    /// assert_eq!(alternative_alleles.len(), 1);
    /// assert_eq!(&alternative_alleles[0], &alt);
    /// ```
    pub fn alternative_alleles_mut(&mut self) -> &mut AlternativeAlleles {
        &mut self.alternative_alleles
    }

    /// Returns a map of contig records (`contig`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::record::value::{map::Contig, Map}};
    ///
    /// let contig = Map::<Contig>::new();
    ///
    /// let header = vcf::Header::builder()
    ///     .add_contig("sq0", contig.clone())
    ///     .build();
    ///
    /// let contigs = header.contigs();
    /// assert_eq!(contigs.len(), 1);
    /// assert_eq!(&contigs[0], &contig);
    /// ```
    pub fn contigs(&self) -> &Contigs {
        &self.contigs
    }

    /// Returns a mutable reference to a map of contig records (`contig`).
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::record::value::{map::Contig, Map}};
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let contig = Map::<Contig>::new();
    /// header.contigs_mut().insert(String::from("sq0"), contig.clone());
    ///
    /// let contigs = header.contigs();
    /// assert_eq!(contigs.len(), 1);
    /// assert_eq!(&contigs[0], &contig);
    /// ```
    pub fn contigs_mut(&mut self) -> &mut Contigs {
        &mut self.contigs
    }

    /// Returns a list of sample names that come after the FORMAT column in the header record.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexmap::IndexSet;
    /// use noodles_vcf as vcf;
    ///
    /// let header = vcf::Header::builder()
    ///     .add_sample_name("sample0")
    ///     .add_sample_name("sample1")
    ///     .build();
    ///
    /// let expected: IndexSet<_> = [String::from("sample0"), String::from("sample1")]
    ///     .into_iter()
    ///     .collect();
    ///
    /// assert_eq!(header.sample_names(), &expected);
    /// ```
    pub fn sample_names(&self) -> &SampleNames {
        &self.sample_names
    }

    /// Returns a mutable reference to a list of sample names that come after the FORMAT column in
    /// the header record.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexmap::IndexSet;
    /// use noodles_vcf as vcf;
    ///
    /// let mut header = vcf::Header::builder().add_sample_name("sample0").build();
    /// header.sample_names_mut().insert(String::from("sample1"));
    ///
    /// let expected: IndexSet<_> = [String::from("sample0"), String::from("sample1")]
    ///     .into_iter()
    ///     .collect();
    ///
    /// assert_eq!(header.sample_names(), &expected);
    /// ```
    pub fn sample_names_mut(&mut self) -> &mut SampleNames {
        &mut self.sample_names
    }

    /// Returns a map of records with nonstandard keys.
    ///
    /// This includes all records other than `fileformat`, `INFO`, `FILTER`, `FORMAT`, `ALT`, and
    /// `contig`.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{self as vcf, header::record::Value};
    ///
    /// let header = vcf::Header::builder()
    ///     .insert("fileDate".parse()?, Value::from("20200709"))?
    ///     .build();
    ///
    /// assert_eq!(header.other_records().len(), 1);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn other_records(&self) -> &OtherRecords {
        &self.other_records
    }

    /// Returns a mutable reference to a map of collections of records with nonstandard keys.
    ///
    /// This includes all records other than `fileformat`, `INFO`, `FILTER`, `FORMAT`, `ALT`, and
    /// `contig`.
    ///
    /// To simply add an nonstandard record, consider using [`Self::insert`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::{value::Collection, Value},
    /// };
    ///
    /// let mut header = vcf::Header::default();
    ///
    /// let collection = Collection::Unstructured(vec![String::from("20200709")]);
    /// header.other_records_mut().insert("fileDate".parse()?, collection.clone());
    ///
    /// assert_eq!(header.other_records().get("fileDate"), Some(&collection));
    /// # Ok::<_, vcf::header::record::key::other::ParseError>(())
    /// ```
    pub fn other_records_mut(&mut self) -> &mut OtherRecords {
        &mut self.other_records
    }

    /// Returns a collection of header values with the given key.
    ///
    /// This includes all records other than `fileformat`, `INFO`, `FILTER`, `FORMAT`, `ALT`, and
    /// `contig`.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::{value::Collection, Value},
    /// };
    ///
    /// let header = vcf::Header::builder()
    ///     .insert("fileDate".parse()?, Value::from("20200709"))?
    ///     .build();
    ///
    /// assert_eq!(
    ///     header.get("fileDate"),
    ///     Some(&Collection::Unstructured(vec![String::from("20200709")]))
    /// );
    ///
    /// assert!(header.get("reference").is_none());
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&record::value::Collection>
    where
        Q: ?Sized + Hash + indexmap::Equivalent<record::key::Other>,
    {
        self.other_records.get(key)
    }

    /// Inserts a key-value pair representing a nonstandard record into the header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{
    ///     self as vcf,
    ///     header::record::{value::Collection, Value},
    /// };
    ///
    /// let mut header = vcf::Header::default();
    /// assert!(header.get("fileDate").is_none());
    ///
    /// header.insert("fileDate".parse()?, Value::from("20200709"))?;
    /// assert_eq!(
    ///     header.get("fileDate"),
    ///     Some(&Collection::Unstructured(vec![String::from("20200709")]))
    /// );
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn insert(
        &mut self,
        key: record::key::Other,
        value: record::Value,
    ) -> Result<(), record::value::collection::AddError> {
        let collection = self
            .other_records
            .entry(key)
            .or_insert_with(|| match value {
                record::Value::String(_) => record::value::Collection::Unstructured(Vec::new()),
                record::Value::Map(..) => record::value::Collection::Structured(IndexMap::new()),
            });

        collection.add(value)
    }

    #[doc(hidden)]
    pub fn string_maps(&self) -> &StringMaps {
        &self.string_maps
    }

    #[doc(hidden)]
    pub fn string_maps_mut(&mut self) -> &mut StringMaps {
        &mut self.string_maps
    }
}

impl Default for Header {
    fn default() -> Self {
        Builder::default().build()
    }
}

impl FromStr for Header {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::default().parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let header = Header::default();
        assert_eq!(header.file_format(), FileFormat::default());
    }

    #[test]
    fn test_insert_with_duplicate_keys() -> Result<(), Box<dyn std::error::Error>> {
        let key: record::key::Other = "noodles".parse()?;
        let values = [record::Value::from("0"), record::Value::from("1")];

        let mut header = Header::default();

        for value in values {
            header.insert(key.clone(), value)?;
        }

        assert_eq!(
            header.get(&key),
            Some(&record::value::Collection::Unstructured(vec![
                String::from("0"),
                String::from("1")
            ]))
        );

        Ok(())
    }
}
