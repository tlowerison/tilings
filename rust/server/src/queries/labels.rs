use crate::{
    models::*,
    result::DbResult,
    schema::label,
};
use diesel::prelude::*;

pub fn match_labels(query: String, conn: &PgConnection) -> DbResult<Vec<Label>> {
    label::table.filter(label::content.like(format!("%{}%", query)))
        .load::<Label>(conn)
}

pub fn upsert_label(content: String, conn: &PgConnection) -> DbResult<Label> {
    let existing_label = label::table.filter(label::content.eq(&content))
        .get_result(conn)
        .optional()?;
    if let Some(existing_label) = existing_label {
        return Ok(existing_label)
    }
    LabelPost { content }.insert(conn)
}
