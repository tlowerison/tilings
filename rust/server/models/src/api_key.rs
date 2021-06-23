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
