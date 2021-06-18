use crate::{
    connection::{DbConn, Result},
    models::*,
};
use rocket::serde::json::Json;

#[get("/atlas/<id>")]
pub async fn get_atlas(id: i32, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| FullAtlas::find(id, conn)).await.map(Json)
}
