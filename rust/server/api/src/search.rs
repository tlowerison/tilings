use db_conn::DbConn;
use models::*;
use queries;
use result::Result;
use rocket::serde::json::Json;

#[get("/text-search?<query>")]
pub async fn text_search(query: String, db: DbConn) -> Result<Json<Vec<TextSearchItem>>> {
    db.run(move |conn| queries::text_search(query, conn)).await.map(Json)
}