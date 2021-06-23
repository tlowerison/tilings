use diesel::prelude::*;
use models::*;
use result::{Error, Result};
use schema::*;

pub fn add_label_to_polygon(polygon_label_post: PolygonLabelPost, conn: &PgConnection) -> Result<usize> {
    diesel::insert_into(polygonlabel::table)
        .values(polygon_label_post)
        .on_conflict_do_nothing()
        .execute(conn)
        .map_err(Error::from)
}
