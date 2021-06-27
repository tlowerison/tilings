use crate::{
    from_data,
    polygon::*,
    tiling::*,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::tables::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasEdge {
    pub id: i32,
    pub polygon_index: usize,
    pub point_index: usize,
    pub neighbor_index: usize,
    pub parity: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullAtlasEdgePost {
    pub polygon_index: usize,
    pub point_index: usize,
    pub neighbor_index: usize,
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

    impl Full for FullAtlas {
        fn find(id: i32, conn: &PgConnection) -> Result<Self> {
            let atlas = Atlas::find(id, conn)?;

            let full_tiling = FullTiling::find(atlas.tiling_id, conn)?;

            let atlas_vertices = AtlasVertex::belonging_to(&atlas).load::<AtlasVertex>(conn)?;

            let atlas_index_by_atlas_id = atlas_vertices
                .iter()
                .enumerate()
                .map(|(atlas_index, atlas_vertex)| (atlas_vertex.id, atlas_index as i32))
                .collect::<HashMap<i32, i32>>();

            let all_atlas_edges = AtlasEdge::belonging_to(&atlas).load::<AtlasEdge>(conn)?;

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
                        .map(|((polygon_index, point_index), full_polygon_point)| (
                            full_polygon_point.polygon_point.id,
                            (polygon_index, point_index),
                        ))
                )
                .kmerge()
                .collect::<HashMap<i32, (usize, usize)>>();

            let all_full_atlas_edges = all_atlas_edges
                .into_iter()
                .map(|atlas_edge| {
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
                            neighbor_index: atlas_index_by_atlas_id
                                .get(&atlas_edge.sink_id)
                                .ok_or(diesel::result::Error::NotFound)?
                                .clone() as usize,
                        },
                    ))
                })
                .collect::<Result<Vec<(i32, FullAtlasEdge)>>>()?
                .into_iter()
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
            println!("start delete");
            diesel::delete(atlasedge::table.filter(atlasedge::atlas_id.eq(id)))
                .execute(conn)?;
            println!("deleted edges");
            diesel::delete(atlasvertex::table.filter(atlasvertex::atlas_id.eq(id)))
                .execute(conn)?;
            println!("deleted vertices");
            let atlas = Atlas::find(id, conn)?;
            println!("found atlas");
            Atlas::delete(id, conn)?;
            println!("deleted atlas");
            FullTiling::delete(atlas.tiling_id, conn)?;
            println!("deleted full tiling");
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
            let full_tiling = self.tiling.as_full_tiling_post(default_atlas_tiling_type_id()).insert(conn)?;

            let atlas = AtlasPost {
                tiling_id: full_tiling.tiling.id,
                tiling_type_id: default_atlas_tiling_type_id(),
            }.insert(conn)?;

            let full_polygons = FullPolygon::find_batch(self.polygon_ids, conn)?;

            let atlas_vertices = self.vertices
                .iter()
                .map(|_|
                    AtlasVertexPost {
                        atlas_id: atlas.id,
                    }.insert(conn)
                )
                .collect::<Result<Vec<AtlasVertex>>>()?;

            let full_atlas_vertices = izip!(atlas_vertices.iter(), self.vertices.iter())
                .map(|(atlas_vertex, full_atlas_vertex_post)| {
                    let atlas_edges = full_atlas_vertex_post.edges
                        .iter()
                        .map(|edge_post| AtlasEdgePost {
                            atlas_id: atlas.id,
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
                        }.insert(conn))
                        .collect::<Result<Vec<AtlasEdge>>>()?;
                    Ok(FullAtlasVertex {
                        id: atlas_vertex.id,
                        edges: izip!(atlas_edges.iter(), full_atlas_vertex_post.edges.iter())
                            .map(|(atlas_edge, full_atlas_edge_post)| FullAtlasEdge {
                                id: atlas_edge.id,
                                polygon_index: full_atlas_edge_post.polygon_index,
                                point_index: full_atlas_edge_post.point_index,
                                neighbor_index: full_atlas_edge_post.neighbor_index,
                                parity: full_atlas_edge_post.parity,
                            })
                            .collect()
                    })
                })
                .collect::<Result<Vec<FullAtlasVertex>>>()?;

            Ok(FullAtlas {
                id: atlas.id,
                tiling: full_tiling,
                polygons: full_polygons,
                vertices: full_atlas_vertices,
            })
        }
    }

    impl FullChangeset for FullAtlasPatch {
        type Base = FullAtlas;

        fn update(self, conn: &PgConnection) -> Result<Self::Base> {
            let atlas_id = self.id.clone();
            let full_tiling = self.tiling.update(conn)?;

            if let Some(vertices) = self.vertices {
                let polygon_ids = self.polygon_ids.ok_or(Error::Status(Status::BadRequest))?;

                diesel::delete(atlasvertex::table.filter(atlasvertex::atlas_id.eq(atlas_id)))
                    .execute(conn)?;
                diesel::delete(atlasedge::table.filter(atlasedge::atlas_id.eq(atlas_id)))
                    .execute(conn)?;

                let full_polygons = FullPolygon::find_batch(polygon_ids, conn)?;

                let atlas_vertices = vertices
                    .iter()
                    .map(|_| AtlasVertexPost { atlas_id }.insert(conn))
                    .collect::<Result<Vec<AtlasVertex>>>()?;

                let full_atlas_vertices = izip!(atlas_vertices.iter(), vertices.iter())
                    .map(|(atlas_vertex, full_atlas_vertex_post)| {
                        let atlas_edges = full_atlas_vertex_post.edges
                            .iter()
                            .map(|edge_post| AtlasEdgePost {
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
                            }.insert(conn))
                            .collect::<Result<Vec<AtlasEdge>>>()?;
                        Ok(FullAtlasVertex {
                            id: atlas_vertex.id,
                            edges: izip!(atlas_edges.iter(), full_atlas_vertex_post.edges.iter())
                                .map(|(atlas_edge, full_atlas_edge_post)| FullAtlasEdge {
                                    id: atlas_edge.id,
                                    polygon_index: full_atlas_edge_post.polygon_index,
                                    point_index: full_atlas_edge_post.point_index,
                                    neighbor_index: full_atlas_edge_post.neighbor_index,
                                    parity: full_atlas_edge_post.parity,
                                })
                                .collect()
                        })
                    })
                    .collect::<Result<Vec<FullAtlasVertex>>>()?;

                Ok(FullAtlas {
                    id: atlas_id,
                    tiling: full_tiling,
                    polygons: full_polygons,
                    vertices: full_atlas_vertices,
                })
            } else {
                FullAtlas::find(atlas_id, conn)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::internal::*;
