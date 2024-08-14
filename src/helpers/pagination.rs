use crate::params::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PER_PAGE_DEFAULT: usize = 10;
const PER_PAGE_MAX: usize = 100;

#[derive(Clone, Deserialize, Debug)]
pub struct CursorPaginationParams {
    #[serde(default = "CursorPaginationParams::per_page_default")]
    pub per_page: usize,
    pub prev_page: Option<Uuid>,
    pub next_page: Option<Uuid>,
}

impl CursorPaginationParams {
    pub fn limit(&self) -> usize {
        self.per_page
    }

    pub fn limit_with_lookahead(&self) -> usize {
        // We add 1 here, so that we can "look ahead" and see if there are results beyond the
        // current page (which can then inform things like `prev_page` and `next_page`).
        self.per_page + 1
    }

    pub fn cursor(&self) -> Option<Uuid> {
        // if both next_page and prev_page are (incorrectly) in the params, then this is next_page
        self.next_page.or(self.prev_page)
    }

    pub fn direction(&self) -> Direction {
        match (self.prev_page, self.next_page) {
            (Some(_), None) => Direction::Ascending,
            // if both next_page and prev_page are (incorrectly) in the params, then this is Descending
            _ => Direction::Descending,
        }
    }

    fn per_page_default() -> usize {
        PER_PAGE_DEFAULT
    }

    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.per_page < 1 {
            report.add("per_page", "'per_page' must be greater than 0");
        }

        if self.per_page > PER_PAGE_MAX {
            report.add(
                "per_page",
                format!("'per_page' may not be greater than {PER_PAGE_MAX}"),
            );
        }

        report.to_result()
    }
}

impl Default for CursorPaginationParams {
    fn default() -> Self {
        Self {
            per_page: PER_PAGE_DEFAULT,
            prev_page: None,
            next_page: None,
        }
    }
}

#[derive(Serialize)]
pub struct CursorPaginationResponse {
    pub prev_page: Option<Uuid>,
    pub next_page: Option<Uuid>,
}

impl CursorPaginationResponse {
    pub fn new(params: &CursorPaginationParams, results: &mut [impl HasOrderedId]) -> Self {
        let (mut prev_page, mut next_page) = (
            results.first().map(HasOrderedId::ordered_id),
            results.last().map(HasOrderedId::ordered_id),
        );

        if params.direction() == Direction::Ascending {
            results.reverse();
            std::mem::swap(&mut next_page, &mut prev_page);
        }

        Self {
            prev_page,
            next_page,
        }
    }

    // This assumes that results were produced using limit_with_lookahead.
    pub fn new_with_lookahead(
        params: &CursorPaginationParams,
        results: &mut Vec<impl HasOrderedId>,
    ) -> Self {
        let (mut prev_page, mut next_page) = match params.cursor() {
            Some(_) => (results.first().map(HasOrderedId::ordered_id), None),
            None => (None, None),
        };

        if results.len() > params.per_page {
            results.pop();
            next_page = results.last().map(HasOrderedId::ordered_id);
        }

        if params.direction() == Direction::Ascending {
            results.reverse();
            std::mem::swap(&mut next_page, &mut prev_page);
        }

        Self {
            prev_page,
            next_page,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Ascending,
    Descending,
}

impl Direction {
    pub fn to_raw_sql(&self) -> &str {
        match self {
            Direction::Ascending => "ASC",
            Direction::Descending => "DESC",
        }
    }
}

pub trait HasOrderedId {
    fn ordered_id(&self) -> Uuid;
}
