use crate::{
    connection::{DbConn, Result},
    models::*,
    schema::{
        tiling::dsl::tiling,
        tilinglabel::dsl::tilinglabel,
    },
};
use diesel::{QueryDsl, RunQueryDsl};
use rocket::response::Debug;

pub async fn get_tiling(db: DbConn, id: i32) -> Result<Vec<(Tiling, TilingLabel)>> {
    db.run(move |conn|
        tiling.find(id)
            .inner_join(tilinglabel)
            // .inner_join(tilinglabel.inner_join(label))
            .get_results(conn)
    ).await.map_err(Debug)
}

pub async fn create_tiling(db: DbConn, tiling_post: TilingPost) -> Result<Tiling> {
    db.run(move |conn| tiling_post.insert(conn)).await
}

pub async fn update_tiling(db: DbConn, id: i32, tiling_patch: TilingPatch) -> Result<Tiling> {
    db.run(move |conn|
        diesel::update(tiling.find(id))
            .set(tiling_patch)
            .get_result(conn)
    ).await.map_err(Debug)
}

pub async fn delete_tiling(db: DbConn, id: i32) -> Result<Tiling> {
    db.run(move |conn| diesel::delete(tiling.find(id)).get_result(conn)).await.map_err(Debug)
}
