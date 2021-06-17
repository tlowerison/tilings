use crate::{
    connection::Result,
    models::*,
    schema::polygonlabel,
};
use diesel::prelude::*;
use rocket::response::Debug;

pub fn add_label_to_polygon(polygon_label_post: PolygonLabelPost, conn: &PgConnection) -> Result<usize> {
    diesel::insert_into(polygonlabel::table)
        .values(polygon_label_post)
        .on_conflict_do_nothing()
        .execute(conn)
        .map_err(Debug)
}
