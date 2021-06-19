use crate::{
    data,
    connection::DbConn,
    models::*,
    queries,
    response::Response,
};
use serde::{Deserialize, Serialize};

#[get("/check-email/<email>")]
pub async fn check_email(email: String, db: DbConn) -> Response<bool> {
    Response::from(
        db.run(move |conn|
            queries::check_email(email, conn)
        ).await
    )
}

#[get("/check-display-name/<display_name>")]
pub async fn check_display_name(display_name: String, db: DbConn) -> Response<bool> {
    Response::from(
        db.run(move |conn|
            queries::check_display_name(display_name, conn)
        ).await
    )
}

#[post("/sign-up", data = "<account_post>")]
pub async fn sign_up(account_post: AccountPost, db: DbConn) -> Response<bool> {
    Response::from(
        db.run(move |conn| conn.build_transaction().run(||
            queries::sign_up(account_post, conn)
        )).await
    )
}

#[derive(Deserialize, Serialize)]
pub struct SignInPost {
    email: String,
    password: String,
}

data! { SignInPost }

#[post("/sign-in", data = "<sign_in_post>")]
pub async fn sign_in(sign_in_post: SignInPost, db: DbConn) -> Response<bool> {
    Response::from(
        db.run(move |conn| conn.build_transaction().run(||
            queries::sign_in(sign_in_post.email, sign_in_post.password, conn)
        )).await
    )
}

#[post("/sign-out")]
pub async fn sign_out(db: DbConn) -> Response<bool> {
    Response::from(
        db.run(move |conn| conn.build_transaction().run(||
            queries::sign_out(conn)
        )).await
    )
}
