use crate::{
    connection::DbConn,
    models::*,
    response::Response,
};

const BATCH_LIMIT: u32 = 1000;

#[get("/tiling/<id>")]
pub async fn get_tiling(id: i32, db: DbConn) -> Response<FullTiling> {
    Response::from(db.run(move |conn| FullTiling::find(id, conn)).await)
}

#[get("/tilings?<start_id>&<end_id>&<limit>")]
pub async fn get_tilings(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Response<Vec<Tiling>> {
    Response::from(
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
        ).await
    )
}
