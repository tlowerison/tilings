use crate::{
    models::*,
    result::DbResult,
    schema::polygonlabel,
};
use diesel::prelude::*;

pub fn add_label_to_polygon(polygon_label_post: PolygonLabelPost, conn: &PgConnection) -> DbResult<usize> {
    diesel::insert_into(polygonlabel::table)
        .values(polygon_label_post)
        .on_conflict_do_nothing()
        .execute(conn)
}
