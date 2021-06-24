use crate::{from_data, tables::*};
use serde::{Deserialize, Serialize};

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

from_data! {
    FullTiling,
    FullTilingPost,
    FullTilingPatch
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use super::*;
    use diesel::{self, prelude::*, result::Error as DieselError};
    use result::{Error, Result};
    use schema::*;
    use std::collections::HashMap;

    impl Full for FullTiling {
        fn find(id: i32, conn: &PgConnection) -> Result<Self> {
            let tiling = Tiling::find(id, conn)?;

            let labels = TilingLabel::belonging_to(&tiling)
                .inner_join(label::table)
                .select(label::all_columns)
                .load(conn)?;

            Ok(FullTiling { tiling, labels })
        }

        fn delete(id: i32, conn: &PgConnection) -> Result<usize> {
            Tiling::delete(id, conn)
        }

        fn find_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<Vec<Self>> {
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
                    .collect::<std::result::Result<Vec<Label>, DieselError>>()
                    .map_err(Error::from)
                )
                .collect::<Result<Vec<Vec<Label>>>>()?;

            Ok(
                izip!(tilings.into_iter(), labels.into_iter())
                    .map(|(tiling, labels)| FullTiling { tiling, labels, })
                    .collect()
            )
        }

        fn delete_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<usize> {
            diesel::delete(tilinglabel::table.filter(tilinglabel::tiling_id.eq_any(ids.clone())))
                .execute(conn)?;

            Tiling::delete_batch(ids, conn)
        }
    }

    impl FullInsertable for FullTilingPost {
        type Base = FullTiling;

        fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
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

        fn update(self, conn: &PgConnection) -> Result<Self::Base> {
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
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::internal::*;
