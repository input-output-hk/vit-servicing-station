pub mod rest_client;
pub mod server;
pub mod startup;

use crate::{db::models::api_tokens::APITokenData, v0::api_token::APIToken};

use chrono::offset::Utc;

pub fn get_testing_token() -> (APITokenData, String) {
    let data = b"ffffffffffffffffffffffffffffffff".to_vec();
    let token_data = APITokenData {
        token: APIToken::new(data.clone()),
        creation_time: Utc::now().timestamp(),
        expire_time: Utc::now().timestamp(),
    };
    (
        token_data,
        base64::encode_config(data, base64::URL_SAFE_NO_PAD),
    )
}
