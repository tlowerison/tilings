use crate::{
    connection::{DbConn, Result},
    models::*,
    queries,
};
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[post("/polygon/label", data="<polygon_label_post>")]
pub async fn add_label_to_polygon(polygon_label_post: PolygonLabelPost, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(||
        queries::add_label_to_polygon(polygon_label_post, conn)
    )).await.map(Json)
}

#[post("/polygon", data="<full_polygon_post>")]
pub async fn create_polygon(full_polygon_post: FullPolygonPost, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn| conn.build_transaction().run(||
        full_polygon_post.insert(conn)
    )).await.map(Json)
}

#[delete("/polygon/<id>")]
pub async fn delete_polygon(id: i32, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(||
        FullPolygon::delete(id, conn)
    )).await.map(Json)
}

#[get("/polygon/<id>")]
pub async fn get_polygon(id: i32, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn|
        FullPolygon::find(id, conn)
    ).await.map(Json)
}

#[get("/polygons?<start_id>&<end_id>&<limit>")]
pub async fn get_polygons(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<Polygon>>> {
    db.run(move |conn|
        Polygon::find_all(
            start_id,
            end_id,
            match limit {
                Some(limit) => if limit >= BATCH_LIMIT { BATCH_LIMIT } else { limit },
                None => BATCH_LIMIT,
            },
            conn,
        )
    ).await.map(Json)
}

#[patch("/polygon", data="<full_polygon_patch>")]
pub async fn update_polygon(full_polygon_patch: FullPolygonPatch, db: DbConn) -> Result<Json<FullPolygon>> {
    db.run(move |conn| conn.build_transaction().run(||
        full_polygon_patch.update(conn)
    )).await.map(Json)
}
