use crate::{from_data, tables::*};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::hash_set::HashSet;

#[derive(Deserialize, Serialize)]
pub struct TextSearchItem {
    id: i32,
    table: String,
    title: String,
    labels: Vec<Label>,
}

from_data! { TextSearchItem }

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use super::*;
    use diesel::{self, prelude::*};
    use itertools::Itertools;
    use result::Result;

    pub trait TextSearchable {
        type LeftDBTuple;
        type InnerDBTuple;
        type GroupItem;

        fn process_groups(vec: Vec<Self::GroupItem>) -> Vec<TextSearchItem>;
        fn search_title(query: String, conn: &PgConnection) -> Result<Vec<Self::GroupItem>>;
        fn search_labels(query: String, conn: &PgConnection) -> Result<Vec<Self::GroupItem>>;

        fn text_search(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
            let title_matches = Self::search_title(query.clone(), conn)?;
            let label_matches = Self::search_labels(query, conn)?;
            Ok(Self::process_groups(
                title_matches.into_iter()
                    .chain(label_matches.into_iter())
                    .collect()
            ))
        }
    }

    macro_rules! text_searchable {
        ($($table:ident $name:ident),*) => {
            paste! {
                $(
                    impl TextSearchable for $name {
                        type LeftDBTuple = ($name, Option<([<$name Label>] , Label)>);
                        type InnerDBTuple = ($name, ([<$name Label>] , Label));
                        type GroupItem = (i32, Self::LeftDBTuple);

                        fn search_title(query: String, conn: &PgConnection) -> Result<Vec<Self::GroupItem>> {
                            Ok(
                                schema::$table::table.filter(schema::$table::title.like(format!("%{}%", query)))
                                    .left_join(schema::[<$table label>]::table.inner_join(schema::label::table))
                                    .load::<Self::LeftDBTuple>(conn)?
                                    .into_iter()
                                    .map(|db_tuple| (db_tuple.0.id, db_tuple))
                                    .collect()
                            )
                        }

                        fn search_labels(query: String, conn: &PgConnection) -> Result<Vec<Self::GroupItem>> {
                            let base_ids = schema::label::table.filter(schema::label::content.like(format!("%{}%", query)))
                                .inner_join(schema::[<$table label>]::table.inner_join(schema::$table::table))
                                .select(schema::$table::id)
                                .load::<i32>(conn)?;
                            Ok(
                                schema::$table::table.filter(schema::$table::id.eq_any(base_ids))
                                    .inner_join(schema::[<$table label>]::table.inner_join(schema::label::table))
                                    .load::<Self::InnerDBTuple>(conn)?
                                    .into_iter()
                                    .map(|(entity, label_tuple)| (label_tuple.0.[<$table _id>], (entity, Some(label_tuple))))
                                    .collect()
                            )
                        }

                        fn process_groups(group_items: Vec<Self::GroupItem>) -> Vec<TextSearchItem> {
                            group_items.into_iter()
                                .into_group_map()
                                .into_iter()
                                .map(|(entity_id, group_items)| {
                                    let title = group_items.get(0).unwrap().0.title.clone();

                                    let mut labels = group_items.into_iter()
                                        .filter_map(|(_, db_tuple)| db_tuple.map(|(_, label)| label))
                                        .collect::<HashSet<Label>>()
                                        .into_iter()
                                        .collect::<Vec<Label>>();

                                    labels.sort_by(|a, b| a.content.partial_cmp(&b.content).unwrap());

                                    TextSearchItem {
                                        id: entity_id.clone(),
                                        table: String::from(stringify!($name)),
                                        title,
                                        labels,
                                    }
                                })
                                .collect()
                        }
                    }
                )*
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    text_searchable! {
        polygon Polygon,
        tiling Tiling
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::internal::*;
