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

        report.merge_result(self.filename.validate())?;
        report.merge_result(self.description.validate())?;
        report.merge_result(self.body.validate())?;

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
            report.merge_result(filename.validate())?;
        }
        if let Some(description) = &self.description {
            report.merge_result(description.validate())?;
        }
        if let Some(body) = &self.body {
            report.merge_result(body.validate())?;
        }

        report.to_result()
    }
}
