use crate::params::prelude::*;
use derive_more::{From, Into};
use serde::Deserialize;

const MAX_FILENAME_LENGTH: usize = 256;
const MAX_DESCRIPTION_LENGTH: usize = 256;
const INVALID_FILENAME_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

pub const FILENAME_REPORT_KEY: &str = "filename";
pub const DESCRIPTION_REPORT_KEY: &str = "description";
pub const BODY_REPORT_KEY: &str = "body";

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct FilenameParam(String);

impl FilenameParam {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.is_empty() {
            report.add(FILENAME_REPORT_KEY, "Filename is a required field");
        }
        if self.0.chars().count() > MAX_FILENAME_LENGTH {
            report.add(
                FILENAME_REPORT_KEY,
                format!("Filename may not be longer than {MAX_FILENAME_LENGTH} characters"),
            );
        }
        if self.0.chars().any(|c| INVALID_FILENAME_CHARS.contains(&c)) {
            report.add(
                FILENAME_REPORT_KEY,
                format!(
                    "Filename may not contain the following characters: {}",
                    // once it gets stabilized, we should be able to replace this with .intersperse
                    INVALID_FILENAME_CHARS
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            );
        }
        if self.0.ends_with('.') {
            report.add(
                FILENAME_REPORT_KEY,
                "Filename may not end with a '.' character",
            );
        }

        report.to_result()
    }
}

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct DescriptionParam(String);

impl DescriptionParam {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.chars().count() > MAX_DESCRIPTION_LENGTH {
            report.add(
                DESCRIPTION_REPORT_KEY,
                format!("Description may not be longer than {MAX_DESCRIPTION_LENGTH} characters"),
            );
        }

        report.to_result()
    }
}

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct BodyParam(String);

impl BodyParam {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.is_empty() {
            report.add(BODY_REPORT_KEY, "Body is a required field");
        }

        report.to_result()
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum VisibilityParam {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "secret")]
    Secret,
}

impl From<VisibilityParam> for String {
    fn from(value: VisibilityParam) -> Self {
        match value {
            VisibilityParam::Public => "public".into(),
            VisibilityParam::Secret => "secret".into(),
        }
    }
}

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
    pub filename: FilenameParam,
    pub description: DescriptionParam,
    pub body: BodyParam,
}

impl UpdatePasteParams {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        report.merge_result(self.filename.validate())?;
        report.merge_result(self.description.validate())?;
        report.merge_result(self.body.validate())?;

        report.to_result()
    }
}
