use serde::{de::Error, Deserialize, Deserializer};
use uuid::Uuid;

const LIMIT_DEFAULT: usize = 10;
const LIMIT_MAX: usize = 100;
const PAGE_MAX: usize = 100;

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct OffsetPaginationParams {
    #[serde(deserialize_with = "OffsetPaginationParams::deserialize_page")]
    pub page: usize,
    #[serde(deserialize_with = "OffsetPaginationParams::deserialize_limit")]
    pub limit: usize,
}

impl OffsetPaginationParams {
    pub fn limit_with_lookahead(&self) -> usize {
        // We add 1 here, so that we can "look ahead" and see if there are results beyond the
        // current page (which can then inform things like `next_page`).
        self.limit + 1
    }

    pub fn offset(&self) -> usize {
        self.page * self.limit
    }

    fn deserialize_page<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value > PAGE_MAX {
            Err(D::Error::custom(format!(
                "page value must be less than or equal to {}, got {}",
                PAGE_MAX, value
            )))
        } else {
            Ok(value)
        }
    }

    fn deserialize_limit<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value > LIMIT_MAX {
            Err(D::Error::custom(format!(
                "limit value must be less than or equal to {}, got {}",
                LIMIT_MAX, value
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
            limit: LIMIT_DEFAULT,
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

        let has_lookahead_result = results.len() > params.limit;
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
    pub cursor: Option<Uuid>,
    pub dir: Direction,
    #[serde(deserialize_with = "CursorPaginationParams::deserialize_limit")]
    pub limit: usize,
}

impl CursorPaginationParams {
    pub fn limit_with_lookahead(&self) -> usize {
        // We add 1 here, so that we can "look ahead" and see if there are results beyond the
        // current page (which can then inform things like `prev_page` and `next_page`).
        self.limit + 1
    }

    fn deserialize_limit<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        if value > LIMIT_MAX {
            Err(D::Error::custom(format!(
                "limit value must be less than or equal to {}, got {}",
                LIMIT_MAX, value
            )))
        } else {
            Ok(value)
        }
    }
}

impl Default for CursorPaginationParams {
    fn default() -> Self {
        Self {
            cursor: None,
            dir: Direction::Descending,
            limit: LIMIT_DEFAULT,
        }
    }
}

pub struct CursorPaginationResponse {
    pub prev_page: Option<Uuid>,
    pub next_page: Option<Uuid>,
}

impl CursorPaginationResponse {
    // This assumes that results were produced using limit_with_lookahead. And it will mutate
    // the results so that elements are reversed if the params direction was Ascending and so that
    // the lookahead element is removed
    pub fn new(params: &CursorPaginationParams, results: &mut Vec<impl HasOrderedId>) -> Self {
        let (mut prev_page, mut next_page) = match params.cursor {
            Some(_) => (results.first().map(|el| el.ordered_id()), None),
            None => (None, None),
        };

        if results.len() > params.limit {
            results.pop();
            next_page = results.last().map(|el| el.ordered_id());
        }

        if params.dir == Direction::Ascending {
            results.reverse();
            std::mem::swap(&mut next_page, &mut prev_page);
        }

        Self {
            prev_page,
            next_page,
        }
    }
}

#[derive(Clone, Copy, Deserialize, Debug, PartialEq)]
pub enum Direction {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
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
