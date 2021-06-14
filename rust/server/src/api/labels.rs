use super::super::{
    connection::{DbConn, Result},
    models::{Label, TilingLabelPost},
    queries,
};
use rocket::serde::json::Json;

#[get("/match-labels?<query>")]
pub async fn match_labels(query: String, db: DbConn) -> Result<Json<Vec<Label>>> {
    queries::match_labels(db, query).await.map(Json)
}

#[post("/add-label-to-tiling", data = "<tiling_label>")]
pub async fn add_label_to_tiling(tiling_label: TilingLabelPost, db: DbConn) -> Result<()> {
    queries::add_label_to_tiling(db, tiling_label).await
}
