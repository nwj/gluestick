use serde::{de::Error, Deserialize, Deserializer, Serialize};
use uuid::Uuid;

const PER_PAGE_DEFAULT: usize = 10;
const PER_PAGE_MAX: usize = 100;
const PAGE_MAX: usize = 50;

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct OffsetPaginationParams {
    #[serde(deserialize_with = "OffsetPaginationParams::deserialize_page")]
    pub page: usize,
    #[serde(deserialize_with = "OffsetPaginationParams::deserialize_per_page")]
    pub per_page: usize,
}

impl OffsetPaginationParams {
    pub fn limit_with_lookahead(&self) -> usize {
        // We add 1 here, so that we can "look ahead" and see if there are results beyond the
        // current page (which can then inform things like `next_page`).
        self.per_page + 1
    }

    pub fn offset(&self) -> usize {
        self.page * self.per_page
    }

    fn deserialize_page<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value > PAGE_MAX {
            Err(D::Error::custom(format!(
                "page value must be less than or equal to {PAGE_MAX}, got {value}"
            )))
        } else {
            Ok(value)
        }
    }

    fn deserialize_per_page<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value > PER_PAGE_MAX {
            Err(D::Error::custom(format!(
                "per_page value must be less than or equal to {PER_PAGE_MAX}, got {value}"
            )))
        } else {
            Ok(value)
        }
    }
}

impl Default for OffsetPaginationParams {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: PER_PAGE_DEFAULT,
        }
    }
}

pub struct OffsetPaginationResponse {
    pub prev_page: Option<usize>,
    pub next_page: Option<usize>,
}

impl OffsetPaginationResponse {
    // This assumes that results were produced using limit_with_lookahead
    pub fn new<T>(params: &OffsetPaginationParams, results: &mut Vec<T>) -> Self {
        let prev_page = match params.page {
            0 => None,
            p => Some(p - 1),
        };

        let has_lookahead_result = results.len() > params.per_page;
        if has_lookahead_result {
            results.pop();
        }

        let next_page = match (params.page, has_lookahead_result) {
            (p, _) if p > PAGE_MAX => None,
            (_, false) => None,
            (p, true) => Some(p + 1),
        };

        Self {
            prev_page,
            next_page,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct CursorPaginationParams {
    pub prev_page: Option<Uuid>,
    pub next_page: Option<Uuid>,
    #[serde(deserialize_with = "CursorPaginationParams::deserialize_per_page")]
    pub per_page: usize,
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

    fn deserialize_per_page<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value > PER_PAGE_MAX {
            Err(D::Error::custom(format!(
                "per_page value must be less than or equal to {PER_PAGE_MAX}, got {value}"
            )))
        } else {
            Ok(value)
        }
    }
}

impl Default for CursorPaginationParams {
    fn default() -> Self {
        Self {
            prev_page: None,
            next_page: None,
            per_page: PER_PAGE_DEFAULT,
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
