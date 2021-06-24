use crate::common::percent_decode;
use db_conn::DbConn;
use models::*;
use queries;
use result::Result;
use rocket::serde::json::Json;

#[get("/omni-search?<query>")]
pub async fn omni_search(query: String, db: DbConn) -> Result<Json<Vec<TextSearchItem>>> {
    db.run(move |conn| queries::omni_search(percent_decode(query)?, conn)).await.map(Json)
}

#[get("/tiling-search?<query>")]
pub async fn tiling_search(query: String, db: DbConn) -> Result<Json<Vec<TextSearchItem>>> {
    db.run(move |conn| queries::tiling_search(percent_decode(query)?, conn)).await.map(Json)
}
