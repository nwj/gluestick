use serde::{de::Error, Deserialize, Deserializer};

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
    pub fn new<T>(params: &OffsetPaginationParams, results: &[T]) -> Self {
        let prev_page = match params.page {
            0 => None,
            p => Some(p - 1),
        };

        let has_lookahead_result = results.len() > params.limit;
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
