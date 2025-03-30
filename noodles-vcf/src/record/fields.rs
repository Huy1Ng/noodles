mod bounds;

use std::io;

use noodles_core::Position;

pub(crate) use self::bounds::Bounds;
use super::{AlternateBases, Filters, Ids, Info, Samples};

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Fields {
    pub(crate) buf: String,
    pub(crate) bounds: Bounds,
}

const MISSING: &str = ".";

impl Fields {
    pub(super) fn reference_sequence_name(&self) -> &str {
        &self.buf[self.bounds.reference_sequence_name_range()]
    }

    pub(super) fn variant_start(&self) -> Option<io::Result<Position>> {
        const TELOMERE_START: &str = "0";

        match &self.buf[self.bounds.variant_start_range()] {
            TELOMERE_START => None,
            src => Some(
                src.parse::<usize>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    .and_then(|n| {
                        Position::try_from(n)
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    }),
            ),
        }
    }

    pub(super) fn ids(&self) -> Ids<'_> {
        let src = match &self.buf[self.bounds.ids_range()] {
            MISSING => "",
            buf => buf,
        };

        Ids::new(src)
    }

    pub(super) fn reference_bases(&self) -> &str {
        &self.buf[self.bounds.reference_bases_range()]
    }

    pub(super) fn alternate_bases(&self) -> AlternateBases {
        let src = match &self.buf[self.bounds.alternate_bases_range()] {
            MISSING => "",
            buf => buf,
        };

        AlternateBases::new(src)
    }

    pub(super) fn quality_score(&self) -> Option<io::Result<f32>> {
        match &self.buf[self.bounds.quality_score_range()] {
            MISSING => None,
            src => Some(
                src.parse()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            ),
        }
    }

    pub(super) fn filters(&self) -> Filters<'_> {
        let src = match &self.buf[self.bounds.filters_range()] {
            MISSING => "",
            buf => buf,
        };

        Filters::new(src)
    }

    pub(super) fn info(&self) -> Info<'_> {
        let src = match &self.buf[self.bounds.info_range()] {
            MISSING => "",
            buf => buf,
        };

        Info::new(src)
    }

    pub(super) fn samples(&self) -> Samples<'_> {
        const DELIMITER: char = '\t';

        let src = &self.buf[self.bounds.samples_range()];

        let is_missing = || {
            src.split(DELIMITER)
                .next()
                .map(|s| s == MISSING)
                .unwrap_or_default()
        };

        if src.is_empty() || is_missing() {
            Samples::new("")
        } else {
            Samples::new(src)
        }
    }
}

impl Default for Fields {
    fn default() -> Self {
        Self {
            buf: String::from("sq01.A...."),
            bounds: Bounds::default(),
        }
    }
}
