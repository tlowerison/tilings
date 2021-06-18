use crate::{
    connection::{DbConn, Result},
    models::*,
};
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[get("/tiling/<id>")]
pub async fn get_tiling(id: i32, db: DbConn) -> Result<Json<FullTiling>> {
    db.run(move |conn| FullTiling::find(id, conn)).await.map(Json)
}

#[get("/tilings?<start_id>&<end_id>&<limit>")]
pub async fn get_tilings(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<Tiling>>> {
    db.run(move |conn|
        Tiling::find_all(
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
