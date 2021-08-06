use crate::from_data;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ResetPasswordPost {
    #[serde(rename = "passwordResetCode")]
    pub password_reset_code: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignInPost {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignUpPost {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub email: String,
    pub password: String,
}

from_data! {
    ResetPasswordPost,
    SignInPost,
    SignUpPost
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use crate::tables::*;
    use diesel::{self, prelude::*};
    use result::{Error, Result};
    use schema::*;

    impl APIKey {
        pub fn find_by_email(email: String, conn: &PgConnection) -> Result<APIKey> {
            account::table.filter(account::email.eq(email))
                .inner_join(apikey::table)
                .select(apikey::all_columns)
                .get_result(conn)
                .map_err(Error::from)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::internal::*;
