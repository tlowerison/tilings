use crate::{
    connection::Result,
    data,
    models::{polygon::*, tables::*, tiling::*},
};
use diesel::{self, prelude::*};
use itertools::Itertools;
use rocket::response::Debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct FullAtlasEdge {
    pub id: i32,
    pub polygon_index: i32,
    pub point_index: i32,
    pub neighbor_index: i32,
}

#[derive(Deserialize, Serialize)]
pub struct FullAtlasEdgePost {
    pub polygon_index: i32,
    pub point_index: i32,
    pub neighbor_index: i32,
}

#[derive(Deserialize, Serialize)]
pub struct FullAtlasVertex {
    pub id: i32,
    pub title: Option<String>,
    pub edges: Vec<FullAtlasEdge>,
}

#[derive(Deserialize, Serialize)]
pub struct FullAtlasVertexPost {
    pub title: Option<String>,
    pub edges: Vec<FullAtlasEdgePost>,
}

#[derive(Deserialize, Serialize)]
pub struct FullAtlas {
    pub tiling: FullTiling,
    pub polygons: Vec<FullPolygon>,
    pub vertices: Vec<FullAtlasVertex>,
}

#[derive(Deserialize, Serialize)]
pub struct FullAtlasPost {
    pub tiling: FullTilingPost,
    pub polygons: Vec<FullPolygonPost>,
    pub vertices: Vec<FullAtlasVertexPost>,
}

#[derive(Deserialize, Serialize)]
pub struct FullAtlasPatch {
    pub tiling: FullTilingPatch,
    pub polygons: Option<Vec<FullPolygonPost>>,
    pub vertices: Option<Vec<FullPolygonPost>>,
}

data! {
    FullAtlasVertex,
    FullAtlasVertexPost,
    FullAtlasEdge,
    FullAtlasEdgePost,
    FullAtlas,
    FullAtlasPost,
    FullAtlasPatch
}

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
                        (polygon_index as i32, point_index as i32),
                    ))
            )
            .kmerge()
            .collect::<HashMap<i32, (i32, i32)>>();

        let all_full_atlas_edges = all_atlas_edges
            .into_iter()
            .map(|atlas_edge| {
                let (polygon_index, point_index) = polygon_and_point_indices_by_polygon_point_id
                    .get(&atlas_edge.polygon_point_id)
                    .ok_or(Debug(diesel::result::Error::NotFound))?;
                Ok((
                    atlas_edge.source_id,
                    FullAtlasEdge {
                        id: atlas_edge.id,
                        polygon_index: polygon_index.clone(),
                        point_index: point_index.clone(),
                        neighbor_index: atlas_index_by_atlas_id
                            .get(&atlas_edge.sink_id)
                            .ok_or(Debug(diesel::result::Error::NotFound))?
                            .clone(),
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
                title: atlas_vertex.title,
                edges: full_atlas_edges,
            })
            .collect::<Vec<FullAtlasVertex>>();

        Ok(FullAtlas {
            tiling: full_tiling,
            polygons: full_polygons,
            vertices: full_atlas_vertices,
        })
    }

    fn delete(_id: i32, _conn: &PgConnection) -> Result<usize> {
        todo!()
    }

    fn find_batch(_ids: Vec<i32>, _conn: &PgConnection) -> Result<Vec<Self>> {
        todo!()
    }

    fn delete_batch(_ids: Vec<i32>, _conn: &PgConnection) -> Result<usize> {
        todo!()
    }
}
