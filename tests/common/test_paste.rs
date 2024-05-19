use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TestPaste {
    #[serde(serialize_with = "serialize_uuid")]
    id: Option<Uuid>,
    title: String,
    description: String,
    body: String,
}

impl Default for TestPaste {
    fn default() -> Self {
        Self {
            id: Some(Uuid::now_v7()),
            title: Uuid::now_v7().to_string(),
            description: Uuid::now_v7().to_string(),
            body: Uuid::now_v7().to_string(),
        }
    }
}

impl TestPaste {
    pub fn default_without_id() -> Self {
        Self::default().without_id()
    }

    pub fn without_id(&self) -> Self {
        Self {
            id: None,
            title: self.title.clone(),
            description: self.description.clone(),
            body: self.body.clone(),
        }
    }
}

fn serialize_uuid<S>(uuid: &Option<Uuid>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(uuid) = uuid {
        serializer.serialize_bytes(uuid.as_bytes())
    } else {
        serializer.serialize_bytes(&[])
    }
}
