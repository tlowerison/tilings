use crate::{
    api::common::*,
    auth::AuthAccount,
    connection::DbConn,
    models::*,
    queries,
    result::Result,
};
use rocket::serde::json::Json;

const BATCH_LIMIT: u32 = 1000;

#[delete("/label/<id>")]
pub async fn delete_label(id: i32, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<usize>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.allowed(&ALLOWED_ADMIN_ROLES, conn)?;
        auth_account.verified(conn)?;
        Label::delete(id, conn)
    })).await.map(Json)
}

#[get("/labels?<start_id>&<end_id>&<limit>")]
pub async fn get_labels(start_id: Option<i32>, end_id: Option<i32>, limit: Option<u32>, db: DbConn) -> Result<Json<Vec<Label>>> {
    db.run(move |conn|
        Label::find_all(start_id, end_id, clamp_optional(BATCH_LIMIT, limit), conn)
    ).await.map(Json)
}

#[get("/match-labels?<query>")]
pub async fn match_labels(query: String, db: DbConn) -> Result<Json<Vec<Label>>> {
    db.run(move |conn| queries::match_labels(query, conn)).await.map(Json)
}

#[post("/label", data = "<label>")]
pub async fn upsert_label(label: String, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<Label>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.allowed(&ALLOWED_ADMIN_ROLES, conn)?;
        auth_account.verified(conn)?;
        queries::upsert_label(label, conn)
    })).await.map(Json)
}
