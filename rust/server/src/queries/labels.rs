use crate::{
    connection::Result,
    models::*,
    schema::label,
};
use diesel::prelude::*;
use rocket::response::Debug;

pub fn match_labels(query: String, conn: &PgConnection) -> Result<Vec<Label>> {
    label::table.filter(label::content.like(format!("%{}%", query)))
        .load::<Label>(conn)
        .map_err(Debug)
}

pub fn upsert_label(content: String, conn: &PgConnection) -> Result<Label> {
    let existing_label = label::table.filter(label::content.eq(&content))
        .get_result(conn)
        .optional()
        .map_err(Debug)?;
    if let Some(existing_label) = existing_label {
        return Ok(existing_label)
    }
    LabelPost { content }.insert(conn)
}
