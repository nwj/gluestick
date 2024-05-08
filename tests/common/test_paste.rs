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
    fn default() -> TestPaste {
        TestPaste {
            id: Some(Uuid::now_v7()),
            title: Uuid::now_v7().to_string(),
            description: Uuid::now_v7().to_string(),
            body: Uuid::now_v7().to_string(),
        }
    }
}

impl TestPaste {
    pub fn default_without_id() -> TestPaste {
        TestPaste {
            id: None,
            title: Uuid::now_v7().to_string(),
            description: Uuid::now_v7().to_string(),
            body: Uuid::now_v7().to_string(),
        }
    }

    pub fn compare_without_ids(&self, other: TestPaste) -> bool {
       self.title == other.title && self.description == other.description && self.body == other.body
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
