use crate::{
    connection::{DbConn, Result},
    models::*,
    queries,
};
use rocket::serde::json::Json;

#[get("/match-labels?<query>")]
pub async fn match_labels(query: String, db: DbConn) -> Result<Json<Vec<Label>>> {
    queries::match_labels(query, db).await.map(Json)
}

#[post("/upsert-label", data = "<label>")]
pub async fn upsert_label(label: String, db: DbConn) -> Result<Json<Label>> {
    queries::upsert_label(label, db).await.map(Json)
}

#[delete("/label/<id>")]
pub async fn delete_label(id: i32, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(|| Label::delete(id, conn))).await.map(Json)
}
