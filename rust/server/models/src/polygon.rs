use crate::{from_data, tables::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FullPolygonPoint {
    pub polygon_point: PolygonPoint,
    pub point: Point,
}

#[derive(Deserialize, Serialize)]
pub struct FullPolygon {
    pub polygon: Polygon,
    pub labels: Vec<Label>,
    pub points: Vec<FullPolygonPoint>,
}

#[derive(Deserialize, Serialize)]
pub struct FullPolygonPost {
    pub polygon: PolygonPost,
    pub label_ids: Option<Vec<i32>>,
    pub points: Vec<PointPost>,
}

#[derive(Deserialize, Serialize)]
pub struct FullPolygonPatch {
    pub polygon: PolygonPatch,
    pub label_ids: Option<Vec<i32>>, // if present, replace
    pub points: Option<Vec<PointPost>>, // if present, replace
}

from_data! {
    FullPolygon,
    FullPolygonPost,
    FullPolygonPatch
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use super::*;
    use diesel::{self, prelude::*, result::Error as DieselError};
    use result::{Error, Result};
    use schema::*;
    use std::collections::HashMap;

    fn to_full_polygon_points(polygon_points: Vec<PolygonPoint>, points: Vec<Point>) -> Result<Vec<FullPolygonPoint>> {
        if polygon_points.len() != points.len() {
            return Err(Error::from(DieselError::NotFound));
        }
        Ok(
            izip!(polygon_points.into_iter(), points.into_iter())
                .map(|(polygon_point, point)| FullPolygonPoint { polygon_point, point })
                .collect()
        )
    }

    impl Full for FullPolygon {
        fn find(id: i32, conn: &PgConnection) -> Result<Self> {
            let polygon = Polygon::find(id, conn)?;

            let labels = PolygonLabel::belonging_to(&polygon)
                .inner_join(label::table)
                .select(label::all_columns)
                .load(conn)?;

            let polygon_points = PolygonPoint::belonging_to(&polygon)
                .order(polygonpoint::sequence.asc())
                .load::<PolygonPoint>(conn)?;

            let points = Point::find_batch(
                polygon_points.iter().map(|polygon_point| polygon_point.point_id).collect(),
                conn,
            )?;

            Ok(FullPolygon {
                polygon,
                labels,
                points: to_full_polygon_points(polygon_points, points)?,
            })
        }

        fn delete(id: i32, conn: &PgConnection) -> Result<usize> {
            diesel::delete(polygonlabel::table.filter(polygonlabel::polygon_id.eq(id)))
                .execute(conn)?;

            let polygon_points = diesel::delete(polygonpoint::table.filter(polygonpoint::polygon_id.eq(id)))
                .get_results::<PolygonPoint>(conn)?;

            Point::delete_batch(polygon_points.into_iter().map(|polygon_point| polygon_point.id).collect(), conn)?;

            Polygon::delete(id, conn)
        }

        // TODO: remove cloning for labels and points
        fn find_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<Vec<Self>> {
            let polygons = Polygon::find_batch(ids, conn)?;

            let all_polygon_labels = PolygonLabel::belonging_to(&polygons)
                .load::<PolygonLabel>(conn)?;

            let all_labels = Label::find_batch(
                all_polygon_labels.iter().map(|polygon_label| polygon_label.label_id).collect(),
                conn,
            )?
                .into_iter()
                .map(|label| (label.id, label))
                .collect::<HashMap<i32, Label>>();

            let labels = all_polygon_labels
                .grouped_by(&polygons)
                .into_iter()
                .map(|polygon_labels| polygon_labels
                    .into_iter()
                    .map(|polygon_label| all_labels
                        .get(&polygon_label.label_id)
                        .map(|label| label.clone())
                        .ok_or(DieselError::NotFound)
                    )
                    .collect::<std::result::Result<Vec<Label>, DieselError>>()
                    .map_err(Error::from)
                )
                .collect::<Result<Vec<Vec<Label>>>>()?;

            let all_polygon_points = PolygonPoint::belonging_to(&polygons)
                .order(polygonpoint::sequence.asc())
                .load::<PolygonPoint>(conn)?;

            let all_points = Point::find_batch(
                all_polygon_points.iter().map(|polygon_point| polygon_point.point_id).collect(),
                conn,
            )?
                .into_iter()
                .map(|point| (point.id, point))
                .collect::<HashMap<i32, Point>>();

            let full_polygon_points = all_polygon_points
                .grouped_by(&polygons)
                .into_iter()
                .map(|polygon_points| polygon_points
                    .into_iter()
                    .map(|polygon_point| all_points
                        .get(&polygon_point.point_id)
                        .map(|point| FullPolygonPoint {
                            polygon_point: polygon_point,
                            point: point.clone(),
                        })
                        .ok_or(DieselError::NotFound)
                    )
                    .collect::<std::result::Result<Vec<FullPolygonPoint>, DieselError>>()
                    .map_err(Error::from)
                )
                .collect::<Result<Vec<Vec<FullPolygonPoint>>>>()?;

            Ok(
                izip!(polygons.into_iter(), labels.into_iter(), full_polygon_points.into_iter())
                    .map(|(polygon, labels, full_polygon_points)| FullPolygon {
                        polygon,
                        labels,
                        points: full_polygon_points,
                    })
                    .collect()
            )
        }

        fn delete_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<usize> {
            diesel::delete(polygonlabel::table.filter(polygonlabel::polygon_id.eq_any(ids.clone())))
                .execute(conn)?;

            let polygon_points = diesel::delete(
                polygonpoint::table.filter(
                    polygonpoint::polygon_id.eq_any(ids.clone())
                )
            )
                .get_results::<PolygonPoint>(conn)?;

            Point::delete_batch(
                polygon_points.into_iter().map(|polygon_point| polygon_point.id)
                    .collect(),
                conn,
            )?;

            Polygon::delete_batch(ids, conn)
        }
    }

    impl FullInsertable for FullPolygonPost {
        type Base = FullPolygon;

        fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
            let polygon = self.polygon.insert(conn)?;

            let labels = match self.label_ids {
                None => Vec::<Label>::with_capacity(0),
                Some(label_ids) => {
                    PolygonLabelPost::insert_batch(
                        label_ids
                            .clone()
                            .into_iter()
                            .map(|label_id| PolygonLabelPost { label_id, polygon_id: polygon.id })
                            .collect(),
                        conn,
                    )?;
                    Label::find_batch(label_ids, conn)?
                },
            };

            let points = PointPost::insert_batch(self.points, conn)?;

            let polygon_points = PolygonPointPost::insert_batch(
                points.iter().enumerate().map(|(i, point)| PolygonPointPost {
                    sequence: i as i32,
                    polygon_id: polygon.id,
                    point_id: point.id,
                    is_locked: false,
                }).collect(),
                conn,
            )?;

            Ok(FullPolygon {
                polygon,
                labels,
                points: to_full_polygon_points(polygon_points, points)?,
            })
        }
    }

    impl FullChangeset for FullPolygonPatch {
        type Base = FullPolygon;

        fn update(self, conn: &PgConnection) -> Result<Self::Base> {
            let polygon = self.polygon.clone().update(conn)?;

            if let Some(label_ids) = self.label_ids {
                let existing_polygon_labels = polygonlabel::table.filter(polygonlabel::polygon_id.eq(self.polygon.id)).load::<PolygonLabel>(conn)?;

                let existing_polygon_label_ids = existing_polygon_labels.iter()
                    .map(|polygon_label| polygon_label.id)
                    .collect::<Vec<i32>>();
                PolygonLabel::delete_batch(existing_polygon_label_ids, conn)?;

                PolygonLabelPost::insert_batch(
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

            let full_polygon_points: Vec<FullPolygonPoint> = match self.points {
                None => PolygonPoint::belonging_to(&polygon)
                    .inner_join(point::table)
                    .load::<(PolygonPoint, Point)>(conn)?
                    .into_iter()
                    .map(|(polygon_point, point)| FullPolygonPoint { polygon_point, point })
                    .collect(),
                Some(points) => {
                    let existing_polygon_points = polygonpoint::table.filter(polygonpoint::polygon_id.eq(self.polygon.id)).load::<PolygonPoint>(conn)?;

                    let existing_polygon_point_ids = existing_polygon_points.iter()
                        .map(|polygon_point| polygon_point.id)
                        .collect::<Vec<i32>>();
                    PolygonPoint::delete_batch(existing_polygon_point_ids, conn)?;

                    let existing_point_ids = existing_polygon_points.iter()
                        .map(|polygon_point| polygon_point.point_id)
                        .collect::<Vec<i32>>();
                    Point::delete_batch(existing_point_ids, conn)?;

                    let points = PointPost::insert_batch(points, conn)?;

                    let polygon_points = PolygonPointPost::insert_batch(
                        points.iter().enumerate().map(|(i, point)| PolygonPointPost {
                            sequence: i as i32,
                            polygon_id: polygon.id,
                            point_id: point.id,
                            is_locked: false,
                        }).collect(),
                        conn,
                    )?;

                    to_full_polygon_points(polygon_points, points)?
                }
            };

            Ok(FullPolygon { polygon, labels, points: full_polygon_points })
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::internal::*;
