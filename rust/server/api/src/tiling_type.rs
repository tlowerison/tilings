use crate::common::*;
use db_conn::DbConn;
use models::*;
use result::Result;
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[get("/v1/tiling-type/<id>")]
pub async fn get_tiling_type(id: i32, db: DbConn) -> Result<Json<TilingType>> {
    db.run(move |conn| TilingType::find(id, conn)).await.map(Json)
}

#[get("/v1/tiling-types?<start_id>&<end_id>&<limit>")]
pub async fn get_tiling_types(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<TilingType>>> {
    db.run(move |conn|
        TilingType::find_all(start_id, end_id, clamp_optional(BATCH_LIMIT, limit), conn)
    ).await.map(Json)
}
