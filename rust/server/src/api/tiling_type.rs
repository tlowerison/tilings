use crate::{
    connection::{DbConn, Result},
    models::*,
};
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[get("/tiling-type/<id>")]
pub async fn get_tiling_type(id: i32, db: DbConn) -> Result<Json<TilingType>> {
    db.run(move |conn| TilingType::find(id, conn)).await.map(Json)
}

#[get("/tiling-types?<start_id>&<end_id>&<limit>")]
pub async fn get_tiling_types(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<TilingType>>> {
    db.run(move |conn|
        TilingType::find_all(
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
