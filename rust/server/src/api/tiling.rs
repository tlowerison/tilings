use super::super::{
    connection::{DbConn, Result},
    models::*,
    queries,
};
use rocket::serde::json::Json;

#[get("/tiling/<id>")]
pub async fn get_tiling(id: i32, db: DbConn) -> Result<Json<Vec<(Tiling, TilingLabel)>>> {
    queries::get_tiling(db, id).await.map(Json)
}

#[post("/tiling", data="<tiling_post>")]
pub async fn create_tiling(tiling_post: TilingPost, db: DbConn) -> Result<Json<Tiling>> {
    queries::create_tiling(db, tiling_post).await.map(Json)
}

#[patch("/tiling/<id>", data="<tiling_patch>")]
pub async fn update_tiling(id: i32, tiling_patch: TilingPatch, db: DbConn) -> Result<Json<Tiling>> {
    queries::update_tiling(db, id, tiling_patch).await.map(Json)
}

#[delete("/tiling/<id>")]
pub async fn delete_tiling(id: i32, db: DbConn) -> Result<Json<Tiling>> {
    queries::delete_tiling(db, id).await.map(Json)
}
