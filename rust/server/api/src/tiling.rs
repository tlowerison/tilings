use crate::common::*;
use auth::AuthAccount;
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

#[patch("/v1/tiling", data="<full_tiling_patch>")]
pub async fn update_tiling(full_tiling_patch: FullTilingPatch, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<FullTiling>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.can_edit(Owned::Tiling, full_tiling_patch.tiling.id, conn)?;
        full_tiling_patch.update(conn)
    })).await.map(Json)
}

#[delete("/v1/tiling/<id>")]
pub async fn delete_tiling(id: i32, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.can_edit(Owned::Tiling, id, conn)?;
        FullTiling::delete(id, conn)
    })).await.map(Json)
}

#[patch("/v1/lock-tiling/<id>")]
pub async fn lock_tiling(id: i32, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<()>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.allowed(&ALLOWED_ADMIN_ROLES, conn)?;
        Owned::Tiling.lock(id, conn)
    })).await.map(Json)
}
