use crate::params::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct FilenameParam(String);

impl FilenameParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for FilenameParam {
    fn from(value: String) -> Self {
        FilenameParam(value)
    }
}

impl From<FilenameParam> for String {
    fn from(value: FilenameParam) -> Self {
        value.0
    }
}

impl Validate for FilenameParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.chars().count() < 1 {
            report.add("filename", "Filename is a required field");
        }
        if self.0.chars().count() > 256 {
            report.add("filename", "Filename may not be longer than 256 characters");
        }
        if self
            .0
            .contains(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..])
        {
            report.add(
                "filename",
                "Filename may not contain the following characters: '<', '>', ':', '\"', '/', '\\', '|', '?', or '*'",
            );
        }
        if self.0.ends_with('.') {
            report.add("filename", "Filename may not end with a '.' character");
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct DescriptionParam(String);

impl DescriptionParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for DescriptionParam {
    fn from(value: String) -> Self {
        DescriptionParam(value)
    }
}

impl From<DescriptionParam> for String {
    fn from(value: DescriptionParam) -> Self {
        value.0
    }
}

impl Validate for DescriptionParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.chars().count() > 256 {
            report.add(
                "description",
                "Description may not be longer than 256 characters",
            );
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct BodyParam(String);

impl BodyParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for BodyParam {
    fn from(value: String) -> Self {
        BodyParam(value)
    }
}

impl From<BodyParam> for String {
    fn from(value: BodyParam) -> Self {
        value.0
    }
}

impl Validate for BodyParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.chars().count() < 1 {
            report.add("body", "Body is a required field");
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
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

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct UpdatePasteParams {
    pub filename: FilenameParam,
    pub description: DescriptionParam,
    pub body: BodyParam,
}

impl Validate for UpdatePasteParams {
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

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}
