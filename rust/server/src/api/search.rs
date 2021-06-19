use crate::{
    connection::DbConn,
    models::*,
    queries,
    response::Response
};

#[get("/text-search?<query>")]
pub async fn text_search(query: String, db: DbConn) -> Response<Vec<TextSearchItem>> {
    Response::from(db.run(move |conn| queries::text_search(query, conn)).await)
}
