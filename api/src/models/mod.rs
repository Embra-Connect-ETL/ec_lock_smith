use serde::{Deserialize, Serialize};

/*----------
 Responses
----------*/
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
}
