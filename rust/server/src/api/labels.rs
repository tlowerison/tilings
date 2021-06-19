use crate::{
    connection::DbConn,
    models::*,
    queries,
    response::Response,
};

#[delete("/label/<id>")]
pub async fn delete_label(id: i32, db: DbConn) -> Response<usize> {
    Response::from(
        db.run(move |conn| conn.build_transaction().run(||
            Label::delete(id, conn)
        )).await
    )
}

#[get("/match-labels?<query>")]
pub async fn match_labels(query: String, db: DbConn) -> Response<Vec<Label>> {
    Response::from(
        db.run(move |conn|
            queries::match_labels(query, conn)
        ).await
    )
}

#[post("/upsert-label", data = "<label>")]
pub async fn upsert_label(label: String, db: DbConn) -> Response<Label> {
    Response::from(
        db.run(move |conn| conn.build_transaction().run(||
            queries::upsert_label(label, conn)
        )).await
    )
}
