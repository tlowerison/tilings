use crate::common::*;
use db_conn::DbConn;
use models::*;
use result::Result;
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[get("/v1/tiling/<id>")]
pub async fn get_tiling(id: i32, db: DbConn) -> Result<Json<FullTiling>> {
    db.run(move |conn| FullTiling::find(id, conn)).await.map(Json)
}

#[get("/v1/tilings?<start_id>&<end_id>&<limit>")]
pub async fn get_tilings(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<Tiling>>> {
    db.run(move |conn|
        Tiling::find_all(start_id, end_id, clamp_optional(BATCH_LIMIT, limit), conn)
    ).await.map(Json)
}
