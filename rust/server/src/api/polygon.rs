use crate::{connection::{DbConn, Result}, models::*};
use rocket::serde::json::Json;

#[post("/polygon", data="<full_polygon_post>")]
pub async fn create_polygon(full_polygon_post: FullPolygonPost, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn| full_polygon_post.insert(conn)).await.map(Json)
}

// #[patch("/polygon", data="<full_polygon_patch>")]
// pub async fn update_polygon(full_polygon_patch: FullPolygonPatch, db: DbConn) -> Result<Json<FullPolygon>> {
//     db.run(move |conn| full_polygon_patch.update(conn)).await.map(Json)
// }
