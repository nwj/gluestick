use crate::views::index::IndexTemplate;

pub async fn index() -> IndexTemplate<'static> {
    IndexTemplate { name: "world" }
}

