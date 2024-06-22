use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PaginationParams {
    pub per_page: Option<usize>,
    pub prev_page: Option<String>,
    pub next_page: Option<String>,
}

#[derive(Default)]
pub struct PaginationParamsBuilder {
    per_page: Option<usize>,
    prev_page: Option<String>,
    next_page: Option<String>,
}

impl PaginationParams {
    pub fn builder() -> PaginationParamsBuilder {
        PaginationParamsBuilder::new()
    }
}

impl PaginationParamsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn per_page(mut self, per_page: usize) -> Self {
        let _ = self.per_page.insert(per_page);
        self
    }

    pub fn prev_page(mut self, prev_page: impl Into<String>) -> Self {
        let _ = self.prev_page.insert(prev_page.into());
        self
    }

    pub fn next_page(mut self, next_page: impl Into<String>) -> Self {
        let _ = self.next_page.insert(next_page.into());
        self
    }

    pub fn build(self) -> PaginationParams {
        let per_page = self.per_page;
        let prev_page = self.prev_page;
        let next_page = self.next_page;

        PaginationParams {
            per_page,
            prev_page,
            next_page,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationResponse {
    pub prev_page: Option<String>,
    pub next_page: Option<String>,
}
