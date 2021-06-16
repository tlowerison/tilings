use crate::{
    connection::Result,
    data,
    models::tables::*,
    schema::{label, point, polygonlabel, polygonpoint},
};
use diesel::{self, BelongingToDsl, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FullPolygon {
    pub polygon: Polygon,
    pub labels: Vec<Label>,
    pub points: Vec<Point>,
}

impl FullPolygon {
    pub fn find(id: i32, conn: &PgConnection) -> Result<FullPolygon> {
        let polygon = Polygon::find(id, conn)?;

        let labels = PolygonLabel::belonging_to(&polygon)
            .inner_join(label::table)
            .select(label::all_columns)
            .load(conn)?;

        let points = PolygonPoint::belonging_to(&polygon)
            .order(polygonpoint::sequence.asc())
            .inner_join(point::table)
            .select(point::all_columns)
            .load(conn)?;

        Ok(FullPolygon { polygon, labels, points })
    }

    pub fn delete(id: i32, conn: &PgConnection) -> Result<usize> {
        diesel::delete(polygonlabel::table.filter(polygonlabel::polygon_id.eq(id))).execute(conn)?;

        let polygon_points = diesel::delete(polygonpoint::table.filter(polygonpoint::polygon_id.eq(id)))
            .get_results::<PolygonPoint>(conn)?;

        Point::batch_delete(polygon_points.into_iter().map(|polygon_point| polygon_point.id).collect(), conn)?;

        Polygon::delete(id, conn)
    }
}

#[derive(Deserialize, Serialize)]
pub struct FullPolygonPost {
    pub polygon: PolygonPost,
    pub label_ids: Vec<i32>,
    pub points: Vec<PointPost>,
}

impl NestedInsertable for FullPolygonPost {
    type Base = FullPolygon;

    fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
        let polygon = self.polygon.insert(conn)?;

        let labels = Label::batch_find(self.label_ids.clone(), conn)?;

        PolygonLabelPost::batch_insert(
            self.label_ids
                .into_iter()
                .map(|label_id| PolygonLabelPost { label_id, polygon_id: polygon.id })
                .collect(),
            conn,
        )?;

        let points = PointPost::batch_insert(self.points, conn)?;

        PolygonPointPost::batch_insert(
            points.iter().enumerate().map(|(i, point)| PolygonPointPost {
                sequence: i as i32,
                polygon_id: polygon.id,
                point_id: point.id,
            }).collect(),
            conn,
        )?;

        Ok(FullPolygon { polygon, labels, points })
    }
}

#[derive(Deserialize, Serialize)]
pub struct FullPolygonPatch {
    pub polygon: PolygonPatch,
    pub label_ids: Option<Vec<i32>>, // if present, replace
    pub points: Option<Vec<PointPost>>, // if present, replace
}

impl NestedChangeset for FullPolygonPatch {
    type Base = FullPolygon;

    fn update(self, conn: &PgConnection) -> Result<Self::Base> {
        let polygon = self.polygon.clone().update(conn)?;

        if let Some(label_ids) = self.label_ids {
            let existing_polygon_labels = polygonlabel::table.filter(polygonlabel::polygon_id.eq(self.polygon.id)).load::<PolygonLabel>(conn)?;

            let existing_polygon_label_ids = existing_polygon_labels.iter()
                .map(|polygon_label| polygon_label.id)
                .collect::<Vec<i32>>();
            PolygonLabel::batch_delete(existing_polygon_label_ids, conn)?;

            PolygonLabelPost::batch_insert(
                label_ids
                    .into_iter()
                    .map(|label_id| PolygonLabelPost { label_id, polygon_id: polygon.id })
                    .collect(),
                conn,
            )?;
        }

        let labels = PolygonLabel::belonging_to(&polygon)
            .inner_join(label::table)
            .select(label::all_columns)
            .load::<Label>(conn)?;

        let points = match self.points {
            None => PolygonPoint::belonging_to(&polygon)
                .inner_join(point::table)
                .select(point::all_columns)
                .load::<Point>(conn)?,
            Some(points) => {
                let existing_polygon_points = polygonpoint::table.filter(polygonpoint::polygon_id.eq(self.polygon.id)).load::<PolygonPoint>(conn)?;

                let existing_polygon_point_ids = existing_polygon_points.iter()
                    .map(|polygon_point| polygon_point.id)
                    .collect::<Vec<i32>>();
                PolygonPoint::batch_delete(existing_polygon_point_ids, conn)?;

                let existing_point_ids = existing_polygon_points.iter()
                    .map(|polygon_point| polygon_point.point_id)
                    .collect::<Vec<i32>>();
                Point::batch_delete(existing_point_ids, conn)?;

                let points = PointPost::batch_insert(points, conn)?;

                PolygonPointPost::batch_insert(
                    points.iter().enumerate().map(|(i, point)| PolygonPointPost {
                        sequence: i as i32,
                        polygon_id: polygon.id,
                        point_id: point.id,
                    }).collect(),
                    conn,
                )?;

                points
            }
        };

        Ok(FullPolygon { polygon, labels, points })
    }
}

data! {
    FullPolygon,
    FullPolygonPost,
    FullPolygonPatch
}
