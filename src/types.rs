use serde::Deserialize;
use validator::Validate;

use crate::error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNote {
    #[validate(length(min = 1, max = 120))]
    pub title: String,
    #[validate(length(min = 1, max = 10_000))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNote {
    #[validate(length(min = 1, max = 120))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 10_000))]
    pub body: Option<String>,
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(e: validator::ValidationErrors) -> Self {
        ApiError::BadRequest(e.to_string())
    }
}
