use crate::common::*;
use auth::AuthAccount;
use db_conn::DbConn;
use models::*;
use result::Result;
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[get("/v1/atlas/<id>")]
pub async fn get_atlas(id: i32, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| FullAtlas::find(id, conn)).await.map(Json)
}

#[get("/v1/atlas-by-tiling-id/<tiling_id>")]
pub async fn get_atlas_by_tiling_id(tiling_id: i32, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| FullAtlas::find_by_tiling_id(tiling_id, conn)).await.map(Json)
}

#[get("/v1/atlases?<start_id>&<end_id>&<limit>")]
pub async fn get_atlases(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<Atlas>>> {
    db.run(move |conn|
        Atlas::find_all(start_id, end_id, clamp_optional(BATCH_LIMIT, limit), conn)
    ).await.map(Json)
}

#[post("/v1/atlas", data = "<full_atlas_post>")]
pub async fn create_atlas(mut full_atlas_post: FullAtlasPost, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.allowed(&ALLOWED_EDITOR_ROLES, conn)?;
        full_atlas_post.owner_id = Some(auth_account.id);
        full_atlas_post.insert(conn)
    })).await.map(Json)
}

#[patch("/v1/atlas", data = "<full_atlas_patch>")]
pub async fn update_atlas(full_atlas_patch: FullAtlasPatch, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<FullAtlas>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.can_edit(Owned::Atlas, full_atlas_patch.id, conn)?;
        full_atlas_patch.update(conn)

    })).await.map(Json)
}

#[delete("/v1/atlas/<id>")]
pub async fn delete_atlas(id: i32, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.can_edit(Owned::Atlas, id, conn)?;
        FullAtlas::delete(id, conn)

    })).await.map(Json)
}

#[patch("/v1/lock-atlas/<id>")]
pub async fn lock_atlas(id: i32, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<()>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.allowed(&ALLOWED_ADMIN_ROLES, conn)?;
        Owned::Atlas.lock(id, conn)
    })).await.map(Json)
}
