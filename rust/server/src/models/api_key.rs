use crate::{
    models::tables::*,
    result::{Error, Result},
    schema::*,
};
use diesel::{self, prelude::*};

impl APIKey {
    pub fn find_by_email(email: String, conn: &PgConnection) -> Result<APIKey> {
        account::table.filter(account::email.eq(email))
            .inner_join(apikey::table)
            .select(apikey::all_columns)
            .get_result(conn)
            .map_err(Error::from)
    }
}
