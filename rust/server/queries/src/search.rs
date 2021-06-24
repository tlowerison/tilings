use diesel::prelude::*;
use models::*;
use result::Result;

pub fn omni_search(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
    let tiling_matches = Tiling::text_search(query.clone(), conn)?;
    let polygon_matches = Polygon::text_search(query, conn)?;
    Ok(tiling_matches.into_iter().chain(polygon_matches.into_iter()).collect::<Vec<TextSearchItem>>())
}

pub fn tiling_search(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
    Tiling::text_search(query.clone(), conn)
}
