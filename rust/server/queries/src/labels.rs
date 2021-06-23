use diesel::prelude::*;
use models::*;
use result::{Error, Result};
use schema::*;

pub fn match_labels(query: String, conn: &PgConnection) -> Result<Vec<Label>> {
    label::table.filter(label::content.like(format!("%{}%", query)))
        .load::<Label>(conn)
        .map_err(Error::from)
}

pub fn upsert_label(content: String, conn: &PgConnection) -> Result<Label> {
    let existing_label = label::table.filter(label::content.eq(&content))
        .get_result(conn)
        .optional()?;
    if let Some(existing_label) = existing_label {
        return Ok(existing_label)
    }
    LabelPost { content }.insert(conn)
}
