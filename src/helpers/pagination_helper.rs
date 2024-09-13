use derive_more::Into;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CursorPaginationParams {
    #[serde(default)]
    pub per_page: PerPage,
    pub prev_page: Option<Uuid>,
    pub next_page: Option<Uuid>,
}

impl CursorPaginationParams {
    pub fn limit(&self) -> usize {
        self.per_page.into()
    }

    pub fn limit_with_lookahead(&self) -> usize {
        // We add 1 here, so that we can "look ahead" and see if there are results beyond the
        // current page (which can then inform things like `prev_page` and `next_page`).
        self.limit() + 1
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

        if results.len() > params.per_page.into() {
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

#[derive(Clone, Copy, Debug, Deserialize, Into, PartialEq, PartialOrd)]
#[serde(from = "usize")]
pub struct PerPage(usize);

impl Default for PerPage {
    fn default() -> Self {
        Self(10)
    }
}

impl From<usize> for PerPage {
    fn from(value: usize) -> Self {
        if (1..=100).contains(&value) {
            Self(value)
        } else {
            Self::default()
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
