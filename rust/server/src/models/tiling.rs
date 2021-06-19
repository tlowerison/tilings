use crate::{
    data,
    models::tables::*,
    result::DbResult,
    schema::*,
};
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct FullTiling {
    pub tiling: Tiling,
    pub labels: Vec<Label>,
}

#[derive(Deserialize, Serialize)]
pub struct FullTilingPost {
    pub tiling: TilingPost,
    pub label_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Serialize)]
pub struct FullTilingPatch {
    pub tiling: TilingPatch,
    pub label_ids: Option<Vec<i32>>,
}

data! {
    FullTiling,
    FullTilingPost,
    FullTilingPatch
}

impl Full for FullTiling {
    fn find(id: i32, conn: &PgConnection) -> DbResult<Self> {
        let tiling = Tiling::find(id, conn)?;

        let labels = TilingLabel::belonging_to(&tiling)
            .inner_join(label::table)
            .select(label::all_columns)
            .load(conn)?;

        Ok(FullTiling { tiling, labels })
    }

    fn delete(id: i32, conn: &PgConnection) -> DbResult<usize> {
        Tiling::delete(id, conn)
    }

    fn find_batch(ids: Vec<i32>, conn: &PgConnection) -> DbResult<Vec<Self>> {
        let tilings = Tiling::find_batch(ids, conn)?;

        let all_tiling_labels = TilingLabel::belonging_to(&tilings)
            .load::<TilingLabel>(conn)?;

        let all_labels = Label::find_batch(
            all_tiling_labels.iter().map(|tl| tl.label_id).collect(),
            conn,
        )?
            .into_iter()
            .map(|label| (label.id, label))
            .collect::<HashMap<i32, Label>>();

        let labels = all_tiling_labels
            .grouped_by(&tilings)
            .into_iter()
            .map(|tls| tls
                .into_iter()
                .map(|tl| all_labels
                    .get(&tl.label_id)
                    .map(|label| label.clone())
                    .ok_or(diesel::result::Error::NotFound)
                )
                .collect::<DbResult<Vec<Label>>>()
            )
            .collect::<DbResult<Vec<Vec<Label>>>>()?;

        Ok(
            izip!(tilings.into_iter(), labels.into_iter())
                .map(|(tiling, labels)| FullTiling { tiling, labels, })
                .collect()
        )
    }

    fn delete_batch(ids: Vec<i32>, conn: &PgConnection) -> DbResult<usize> {
        diesel::delete(tilinglabel::table.filter(tilinglabel::tiling_id.eq_any(ids.clone())))
            .execute(conn)?;

        Tiling::delete_batch(ids, conn)
    }
}

impl FullInsertable for FullTilingPost {
    type Base = FullTiling;

    fn insert(self, conn: &PgConnection) -> DbResult<Self::Base> {
        let tiling = self.tiling.insert(conn)?;

        let labels = match self.label_ids {
            None => Vec::<Label>::with_capacity(0),
            Some(label_ids) => {
                TilingLabelPost::insert_batch(
                    label_ids
                        .clone()
                        .into_iter()
                        .map(|label_id| TilingLabelPost { label_id, tiling_id: tiling.id })
                        .collect(),
                    conn,
                )?;
                Label::find_batch(label_ids, conn)?
            },
        };

        Ok(FullTiling { tiling, labels })
    }
}

impl FullChangeset for FullTilingPatch {
    type Base = FullTiling;

    fn update(self, conn: &PgConnection) -> DbResult<Self::Base> {
        let tiling = self.tiling.clone().update(conn)?;

        if let Some(label_ids) = self.label_ids {
            let existing_tiling_labels = tilinglabel::table.filter(tilinglabel::tiling_id.eq(self.tiling.id)).load::<TilingLabel>(conn)?;

            let existing_tiling_label_ids = existing_tiling_labels.iter()
                .map(|tiling_label| tiling_label.id)
                .collect::<Vec<i32>>();
            TilingLabel::delete_batch(existing_tiling_label_ids, conn)?;

            TilingLabelPost::insert_batch(
                label_ids
                    .into_iter()
                    .map(|label_id| TilingLabelPost { label_id, tiling_id: tiling.id })
                    .collect(),
                conn,
            )?;
        }

        let labels = TilingLabel::belonging_to(&tiling)
            .inner_join(label::table)
            .select(label::all_columns)
            .load::<Label>(conn)?;

        Ok(FullTiling { tiling, labels })
    }
}
