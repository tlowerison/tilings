use crate::{connection::{DbConn, Result}, models::*};
use rocket::serde::json::Json;

#[get("/polygon/<id>")]
pub async fn get_polygon(id: i32, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn| FullPolygon::find(id, conn)).await.map(Json)
}

#[post("/polygon", data="<full_polygon_post>")]
pub async fn create_polygon(full_polygon_post: FullPolygonPost, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn| conn.build_transaction().run(|| full_polygon_post.insert(conn))).await.map(Json)
}

#[patch("/polygon", data="<full_polygon_patch>")]
pub async fn update_polygon(full_polygon_patch: FullPolygonPatch, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn| conn.build_transaction().run(|| full_polygon_patch.update(conn))).await.map(Json)
}

#[delete("/polygon/<id>")]
pub async fn delete_polygon(id: i32, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(|| FullPolygon::delete(id, conn))).await.map(Json)
}
