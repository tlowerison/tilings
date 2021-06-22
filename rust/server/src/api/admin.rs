use crate::{
    api::common::*,
    auth::AuthAccount,
    connection::DbConn,
    models::*,
    queries,
    result::Result,
};
use rocket::serde::json::Json;

#[post("/lock/polygon/<polygon_id>")]
pub async fn lock_polygon(polygon_id: i32, mut auth_account: AuthAccount, db: DbConn) -> Result<Json<()>> {
    db.run(move |conn| conn.build_transaction().run(|| {
        auth_account.allowed(&ALLOWED_ADMIN_ROLES, conn)?;
        auth_account.verified(conn)?;
    })).await.map(Json)
}
