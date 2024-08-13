use crate::params::pastes::{BodyParam, DescriptionParam, FilenameParam, VisibilityParam};
use crate::params::prelude::*;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct CreatePasteParams {
    pub filename: FilenameParam,
    pub description: DescriptionParam,
    pub body: BodyParam,
    pub visibility: VisibilityParam,
}

impl Validate for CreatePasteParams {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        match self.filename.validate() {
            Err(Error::Report(filename_report)) => report.merge(filename_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };
        match self.description.validate() {
            Err(Error::Report(description_report)) => report.merge(description_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };
        match self.body.validate() {
            Err(Error::Report(body_report)) => report.merge(body_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };

        report.to_result()
    }
}

#[derive(Clone, Deserialize)]
pub struct UpdatePasteParams {
    pub filename: Option<FilenameParam>,
    pub description: Option<DescriptionParam>,
    pub body: Option<BodyParam>,
}

impl Validate for UpdatePasteParams {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if let Some(filename) = &self.filename {
            match filename.validate() {
                Err(Error::Report(filename_report)) => report.merge(filename_report),
                Err(Error::Other(e)) => return Err(Error::Other(e)),
                _ => {}
            };
        }
        if let Some(description) = &self.description {
            match description.validate() {
                Err(Error::Report(description_report)) => report.merge(description_report),
                Err(Error::Other(e)) => return Err(Error::Other(e)),
                _ => {}
            };
        }
        if let Some(body) = &self.body {
            match body.validate() {
                Err(Error::Report(body_report)) => report.merge(body_report),
                Err(Error::Other(e)) => return Err(Error::Other(e)),
                _ => {}
            };
        }

        report.to_result()
    }
}
