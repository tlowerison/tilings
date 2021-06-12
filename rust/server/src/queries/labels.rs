use super::super::{
    connection::{DbConn, Result},
    models::{
        Label,
        TilingLabel,
        TilingLabelPost,
    },
    schema::{
        label::{dsl::label, content},
        tilinglabel::table as tilinglabel_table,
    },
};
use diesel::{QueryDsl, RunQueryDsl, TextExpressionMethods};
use rocket::response::Debug;

pub async fn match_labels(db: DbConn, query: String) -> Result<Vec<Label>> {
    db.run(move |conn| label.filter(content.like(format!("%{}%", query))).get_results::<Label>(conn)).await.map_err(Debug)
}

pub async fn add_label_to_tiling(db: DbConn, tiling_label_post: TilingLabelPost) -> Result<()> {
    db.run(move |conn| {
        let result = diesel::insert_into(tilinglabel_table)
            .values(tiling_label_post)
            .on_conflict_do_nothing()
            .get_result::<TilingLabel>(conn);
        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                if let diesel::result::Error::NotFound = e {
                    return Ok(());
                }
                return Err(e);
            }
        }
    }).await.map_err(Debug)
}
