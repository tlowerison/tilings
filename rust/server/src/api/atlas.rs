use crate::{
    connection::DbConn,
    models::*,
    response::Response,
};

const BATCH_LIMIT: u32 = 1000;

#[get("/atlas/<id>")]
pub async fn get_atlas(id: i32, db: DbConn) -> Response<FullAtlas> {
    Response::from(db.run(move |conn| FullAtlas::find(id, conn)).await)
}

#[get("/atlases?<start_id>&<end_id>&<limit>")]
pub async fn get_atlases(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Response<Vec<Atlas>> {
    Response::from(
        db.run(move |conn|
            Atlas::find_all(
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
