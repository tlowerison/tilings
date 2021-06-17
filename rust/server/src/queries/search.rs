use crate::{connection::Result, models::*};
use diesel::prelude::*;

pub fn text_search(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
    let tiling_matches = Tiling::text_search(query.clone(), conn)?;
    let polygon_matches = Polygon::text_search(query, conn)?;
    Ok(tiling_matches.into_iter().chain(polygon_matches.into_iter()).collect::<Vec<TextSearchItem>>())
}
