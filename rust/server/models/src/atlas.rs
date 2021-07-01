use crate::{
    from_data,
    polygon::*,
    tiling::*,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::tables::*;
#[cfg(not(target_arch = "wasm32"))]
use schema::atlasedge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasEdge {
    pub id: i32,
    pub polygon_index: usize,
    pub point_index: usize,
    pub neighbor_index: usize,
    pub neighbor_edge_index: usize,
    pub parity: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasEdgePost {
    pub polygon_index: usize,
    pub point_index: usize,
    pub neighbor_index: usize,
    pub neighbor_edge_index: usize,
    pub parity: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasVertex {
    pub id: i32,
    pub edges: Vec<FullAtlasEdge>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasVertexPost {
    pub edges: Vec<FullAtlasEdgePost>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlas {
    pub id: i32,
    pub tiling: FullTiling,
    pub polygons: Vec<FullPolygon>,
    pub vertices: Vec<FullAtlasVertex>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasPost {
    pub tiling: FullSubTilingPost,
    pub polygon_ids: Vec<i32>,
    pub vertices: Vec<FullAtlasVertexPost>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasPatch {
    pub id: i32,
    pub tiling: FullTilingPatch,
    pub polygon_ids: Option<Vec<i32>>,
    pub vertices: Option<Vec<FullAtlasVertexPost>>,
}

from_data! {
    FullAtlasVertex,
    FullAtlasVertexPost,
    FullAtlasEdge,
    FullAtlasEdgePost,
    FullAtlas,
    FullAtlasPost,
    FullAtlasPatch
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use super::*;
    use crate::tables::default_atlas_tiling_type_id;
    use diesel::{self, prelude::*};
    use itertools::Itertools;
    use result::{Error, Result};
    use rocket::http::Status;
    use schema::*;
    use std::collections::HashMap;

    fn insert_atlas_vertices(
        atlas_id: i32,
        polygon_ids: Vec<i32>,
        vertices: Vec<FullAtlasVertexPost>,
        conn: &PgConnection,
    ) -> Result<()> {
        let full_polygons = FullPolygon::find_batch(polygon_ids, conn)?;

        let atlas_vertices = vertices
            .iter()
            .map(|_| AtlasVertexPost { atlas_id }.insert(conn))
            .collect::<Result<Vec<AtlasVertex>>>()?;

        let all_atlas_edges = izip!(atlas_vertices.iter(), vertices.iter())
            .map(|(atlas_vertex, full_atlas_vertex_post)|
                full_atlas_vertex_post.edges
                    .iter()
                    .enumerate()
                    .map(|(sequence, edge_post)| AtlasEdgePost {
                        atlas_id,
                        polygon_point_id: full_polygons
                            .get(edge_post.polygon_index)
                            .ok_or(Error::Status(Status::BadRequest))?
                            .points
                            .get(edge_post.point_index)
                            .ok_or(Error::Status(Status::BadRequest))?
                            .polygon_point.id,
                        source_id: atlas_vertex.id,
                        sink_id: atlas_vertices
                            .get(edge_post.neighbor_index)
                            .ok_or(Error::Status(Status::BadRequest))?
                            .id,
                        parity: edge_post.parity,
                        sequence: sequence as i32,
                        neighbor_edge_id: None,
                    }.insert(conn))
                    .collect::<Result<Vec<AtlasEdge>>>()
            )
            .collect::<Result<Vec<Vec<AtlasEdge>>>>()?;

        for (full_atlas_vertex_post, atlas_edges) in izip!(vertices.iter(), all_atlas_edges.iter()) {
            for (edge_post, atlas_edge) in izip!(full_atlas_vertex_post.edges.iter(), atlas_edges.iter()) {
                AtlasEdgePatch {
                    id: atlas_edge.id,
                    neighbor_edge_id: Some(Some(
                        all_atlas_edges
                            .get(edge_post.neighbor_index)
                            .ok_or(Error::Status(Status::BadRequest))?
                            .get(edge_post.neighbor_edge_index)
                            .ok_or(Error::Status(Status::BadRequest))?
                            .id
                    )),
                    atlas_id: None,
                    parity: None,
                    polygon_point_id: None,
                    sequence: None,
                    sink_id: None,
                    source_id: None,
                }.update(conn)?;
            }
        }

        Ok(())
    }

    impl Full for FullAtlas {
        fn find(id: i32, conn: &PgConnection) -> Result<Self> {
            let atlas = Atlas::find(id, conn)?;

            let full_tiling = FullTiling::find(atlas.tiling_id, conn)?;

            let atlas_vertices = AtlasVertex::belonging_to(&atlas).load::<AtlasVertex>(conn)?;

            let vertex_index_by_vertex_id = atlas_vertices
                .iter()
                .enumerate()
                .map(|(atlas_index, atlas_vertex)| (atlas_vertex.id, atlas_index as i32))
                .collect::<HashMap<i32, i32>>();

            let all_atlas_edges = AtlasEdge::belonging_to(&atlas)
                .order_by(atlasedge::sequence.asc())
                .load::<AtlasEdge>(conn)?;

            let atlas_edge_indices_by_ids = all_atlas_edges
                .iter()
                .map(|atlas_edge| (atlas_edge.id, atlas_edge.sequence))
                .collect::<HashMap<i32, i32>>();

            let all_polygon_points = PolygonPoint::find_batch(
                all_atlas_edges
                    .iter()
                    .map(|atlas_edge| atlas_edge.polygon_point_id)
                    .collect(),
                conn,
            )?;

            let full_polygons = FullPolygon::find_batch(
                all_polygon_points
                    .iter()
                    .map(|polygon_point| polygon_point.polygon_id)
                    .unique()
                    .collect(),
                conn,
            )?;

            let polygon_and_point_indices_by_polygon_point_id = full_polygons
                .iter()
                .enumerate()
                .map(|(polygon_index, full_polygon)|
                    std::iter::repeat(polygon_index)
                        .take(full_polygon.points.len())
                        .enumerate()
                        .zip(full_polygon.points.iter())
                        .map(|((point_index, polygon_index), full_polygon_point)| (
                            full_polygon_point.polygon_point.id,
                            (polygon_index, point_index),
                        ))
                )
                .kmerge()
                .collect::<HashMap<i32, (usize, usize)>>();

            let all_full_atlas_edges = all_atlas_edges
                .into_iter()
                .map(|atlas_edge| {
                    let neighbor_edge_id = atlas_edge.neighbor_edge_id.ok_or(diesel::result::Error::NotFound)?;
                    let (polygon_index, point_index) = polygon_and_point_indices_by_polygon_point_id
                        .get(&atlas_edge.polygon_point_id)
                        .ok_or(diesel::result::Error::NotFound)?;
                    Ok((
                        atlas_edge.source_id,
                        FullAtlasEdge {
                            id: atlas_edge.id,
                            parity: atlas_edge.parity,
                            polygon_index: polygon_index.clone(),
                            point_index: point_index.clone(),
                            neighbor_index: vertex_index_by_vertex_id
                                .get(&atlas_edge.sink_id)
                                .ok_or(diesel::result::Error::NotFound)?
                                .clone() as usize,
                            neighbor_edge_index: atlas_edge_indices_by_ids
                                .get(&neighbor_edge_id)
                                .ok_or(diesel::result::Error::NotFound)?
                                .clone() as usize,
                        },
                    ))
                })
                .collect::<Result<Vec<(i32, FullAtlasEdge)>>>()?
                .into_iter()
                .sorted_by_key(|x| x.0)
                .group_by(|(atlas_vertex_id, _)| *atlas_vertex_id)
                .into_iter()
                .map(|(_, group)| group
                    .into_iter()
                    .map(|(_, full_atlas_edge)| full_atlas_edge)
                    .collect()
                )
                .collect::<Vec<Vec<FullAtlasEdge>>>();

            let full_atlas_vertices = izip!(
                atlas_vertices.into_iter(),
                all_full_atlas_edges.into_iter(),
            )
                .map(|(atlas_vertex, full_atlas_edges)| FullAtlasVertex {
                    id: atlas_vertex.id,
                    edges: full_atlas_edges,
                })
                .collect::<Vec<FullAtlasVertex>>();

            Ok(FullAtlas {
                id,
                tiling: full_tiling,
                polygons: full_polygons,
                vertices: full_atlas_vertices,
            })
        }

        fn delete(id: i32, conn: &PgConnection) -> Result<usize> {
            diesel::delete(atlasedge::table.filter(atlasedge::atlas_id.eq(id)))
                .execute(conn)?;
            diesel::delete(atlasvertex::table.filter(atlasvertex::atlas_id.eq(id)))
                .execute(conn)?;
            let atlas = Atlas::find(id, conn)?;
            Atlas::delete(id, conn)?;
            FullTiling::delete(atlas.tiling_id, conn)?;
            Ok(1)
        }

        fn find_batch(_ids: Vec<i32>, _conn: &PgConnection) -> Result<Vec<Self>> {
            todo!()
        }

        fn delete_batch(_ids: Vec<i32>, _conn: &PgConnection) -> Result<usize> {
            todo!()
        }
    }

    impl FullInsertable for FullAtlasPost {
        type Base = FullAtlas;

        fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
            let full_tiling = self.tiling
                .as_full_tiling_post(default_atlas_tiling_type_id())
                .insert(conn)?;

            let atlas = AtlasPost {
                tiling_id: full_tiling.tiling.id,
                tiling_type_id: default_atlas_tiling_type_id(),
            }.insert(conn)?;

            insert_atlas_vertices(atlas.id, self.polygon_ids, self.vertices, conn)?;

            FullAtlas::find(atlas.id, conn)
        }
    }

    impl FullChangeset for FullAtlasPatch {
        type Base = FullAtlas;

        fn update(self, conn: &PgConnection) -> Result<Self::Base> {
            self.tiling.update(conn)?;

            if let Some(vertices) = self.vertices {
                diesel::delete(atlasvertex::table.filter(atlasvertex::atlas_id.eq(self.id)))
                    .execute(conn)?;
                diesel::delete(atlasedge::table.filter(atlasedge::atlas_id.eq(self.id)))
                    .execute(conn)?;

                // if updating vertices, must provide polygon ids as well
                let polygon_ids = self.polygon_ids.ok_or(Error::Status(Status::BadRequest))?;

                insert_atlas_vertices(self.id, polygon_ids, vertices, conn)?;
            }
            FullAtlas::find(self.id, conn)
        }
    }

    impl FullAtlas {
        pub fn find_by_tiling_id(tiling_id: i32, conn: &PgConnection) -> Result<Self> {
            let atlas: Atlas = atlas::table.filter(atlas::tiling_id.eq(tiling_id))
                .get_result(conn)?;
            FullAtlas::find(atlas.id, conn)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::internal::*;
