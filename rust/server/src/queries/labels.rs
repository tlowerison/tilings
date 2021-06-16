use crate::{
    connection::{DbConn, Result},
    models::*,
    schema::label,
};
use diesel::prelude::*;
use rocket::response::Debug;

pub async fn match_labels(query: String, db: DbConn) -> Result<Vec<Label>> {
    db.run(move |conn|
        label::table.filter(label::content.like(format!("%{}%", query)))
            .load::<Label>(conn)
    ).await.map_err(Debug)
}

pub async fn upsert_label(content: String, db: DbConn) -> Result<Label> {
    db.run(move |conn| conn.build_transaction().run(|| {
        let existing_label = label::table.filter(label::content.eq(&content))
            .get_result(conn)
            .optional()
            .map_err(Debug)?;
        if let Some(existing_label) = existing_label {
            return Ok(existing_label)
        }
        LabelPost { content }.insert(conn)
    })).await
}
