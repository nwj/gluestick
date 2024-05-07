use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TestPaste {
    #[serde(serialize_with = "serialize_uuid")]
    id: Uuid,
    title: String,
    description: String,
    body: String,
}

impl Default for TestPaste {
    fn default() -> TestPaste {
        TestPaste {
            id: Uuid::now_v7(),
            title: Uuid::now_v7().to_string(),
            description: Uuid::now_v7().to_string(),
            body: Uuid::now_v7().to_string(),
        }
    }
}

fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(uuid.as_bytes())
}
