use crate::{
    connection::Result,
    data,
    models::tables::*,
    schema::{label, point, polygon, polygonlabel, polygonpoint},
};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use rocket::response::Debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FullPolygon {
    pub polygon: Polygon,
    pub labels: Vec<Label>,
    pub points: Vec<Point>,
}

impl FullPolygon {
    pub fn find(conn: &PgConnection, id: i32) -> Result<FullPolygon> {
        let result = polygon::table.find(id)
            .inner_join(polygonlabel::table.inner_join(label::table))
            .inner_join(polygonpoint::table.inner_join(point::table))
            .get_result(conn)
            .map_err(Debug)?;

        // Ok(FullPolygon { polygon, labels, points })
    }
}

#[derive(Deserialize, Serialize)]
pub struct FullPolygonPost {
    pub polygon: PolygonPost,
    pub labelids: Vec<i32>,
    pub points: Vec<PointPost>,
}

impl NestedInsertable for FullPolygonPost {
    type Base = FullPolygon;

    fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
        let polygon = self.polygon.insert(conn)?;

        let labels = Label::batch_find(conn, self.labelids.clone())?;

        let polygonlabels = PolygonLabelPost::batch_insert(
            conn,
            self.labelids
                .into_iter()
                .map(|labelid| PolygonLabelPost { labelid, polygonid: polygon.id })
                .collect(),
        )?;

        let points = PointPost::batch_insert(conn, self.points)?;

        let polygonpoints = PolygonPointPost::batch_insert(
            conn,
            points.iter().enumerate().map(|(i, point)| PolygonPointPost {
                sequence: i as i32,
                polygonid: polygon.id,
                pointid: point.id,
            }).collect(),
        )?;

        Ok(FullPolygon { polygon, labels, points })
    }
}

data! {
    FullPolygon,
    FullPolygonPost
}
