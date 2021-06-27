use crate::common::*;
use db_conn::DbConn;
use models::*;
use result::Result;
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[get("/v1/atlas/<id>")]
pub async fn get_atlas(id: i32, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| FullAtlas::find(id, conn)).await.map(Json)
}

#[get("/v1/atlases?<start_id>&<end_id>&<limit>")]
pub async fn get_atlases(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<Atlas>>> {
    db.run(move |conn|
        Atlas::find_all(start_id, end_id, clamp_optional(BATCH_LIMIT, limit), conn)
    ).await.map(Json)
}

#[post("/v1/atlas", data = "<full_atlas_post>")]
pub async fn create_atlas(full_atlas_post: FullAtlasPost, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| conn.build_transaction().run(|| full_atlas_post.insert(conn))).await.map(Json)
}

#[delete("/v1/atlas/<id>")]
pub async fn delete_atlas(id: i32, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(|| FullAtlas::delete(id, conn))).await.map(Json)
}
