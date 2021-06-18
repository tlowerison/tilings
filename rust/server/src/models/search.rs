use crate::{
    connection::Result,
    data,
    models::tables::*,
};
use diesel::{self, prelude::*};
use itertools::Itertools;
use rocket::response::Debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TextSearchItem {
    id: i32,
    table: String,
    title: String,
    labels: Vec<Label>,
}

pub trait TextSearchable {
    type GroupItem;

    fn process_groups(vec: Vec<Self::GroupItem>) -> Vec<TextSearchItem>;
    fn search_title(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>>;
    fn search_labels(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>>;

    fn text_search(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
        let title_matches = Self::search_title(query.clone(), conn)?;
        let label_matches = Self::search_labels(query, conn)?;
        Ok(title_matches.into_iter().chain(label_matches.into_iter()).collect())
    }
}

data! { TextSearchItem }

macro_rules! text_searchable {
    ($($table:ident $name:ident),*) => {
        mashup! {
            $(
                Label[$name "Label"] = $name Label;
                label[$table "label"] = $table label;
            )*
        }

        $(
            Label! { label! {
                impl TextSearchable for $name {
                    type GroupItem = ($name, ($name "Label", Label));

                    fn search_title(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
                        Ok($name::process_groups(
                            $crate::schema::$table::table.filter($crate::schema::$table::title.like(format!("%{}%", query)))
                                .inner_join($crate::schema::$table "label"::table.inner_join($crate::schema::label::table))
                                .load::<Self::GroupItem>(conn)
                                .map_err(Debug)?
                        ))
                    }

                    fn search_labels(query: String, conn: &PgConnection) -> Result<Vec<TextSearchItem>> {
                        let base_ids = $crate::schema::label::table.filter($crate::schema::label::content.like(format!("%{}%", query)))
                            .inner_join($crate::schema::$table "label"::table.inner_join($crate::schema::$table::table))
                            .select($crate::schema::$table::id)
                            .load::<i32>(conn)
                            .map_err(Debug)?;
                        Ok($name::process_groups(
                            $crate::schema::$table::table.filter($crate::schema::$table::id.eq_any(base_ids))
                                .inner_join($crate::schema::$table "label"::table.inner_join($crate::schema::label::table))
                                .load::<Self::GroupItem>(conn)
                                .map_err(Debug)?
                        ))
                    }

                    fn process_groups(group_items: Vec<Self::GroupItem>) -> Vec<TextSearchItem> {
                        group_items.into_iter()
                            .group_by(|(item, _)| item.id)
                            .into_iter()
                            .map(|(id, group)| {
                                let group = group.collect::<Vec<Self::GroupItem>>();

                                TextSearchItem {
                                    id: id,
                                    table: String::from(stringify!($name)),
                                    title: group.get(0).unwrap().0.title.clone(),
                                    labels: group.into_iter().map(|(_, (_, label))| label).collect(),
                                }
                            })
                            .collect()
                    }
                }
            } }
        )*
    }
}

text_searchable! {
    polygon Polygon,
    tiling Tiling
}