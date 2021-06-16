use crate::{connection::Result, schema::*};
use diesel::{
    self,
    Insertable,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
};
use rocket::response::Debug;
use serde::{Deserialize, Serialize};
use serde_json;

pub trait NestedInsertable {
    type Base;

    fn insert(self, conn: &PgConnection) -> Result<Self::Base>;

    fn batch_insert(conn: &PgConnection, insertables: Vec<Self>) -> Result<Vec<Self::Base>> where Self: Sized {
        let mut results = Vec::<Self::Base>::with_capacity(insertables.len());
        for (i, insertable) in insertables.into_iter().enumerate() {
            let result = insertable.insert(conn)?;
            *(&mut results[i]) = result;
        }
        Ok(results)
    }
}

pub trait NestedChangeset {
    type Base;

    fn update(self, conn: &PgConnection) -> Result<Self::Base>;

    fn batch_update(conn: &PgConnection, changesets: Vec<Self>) -> Result<Vec<Self::Base>> where Self: Sized {
        let mut results = Vec::<Self::Base>::with_capacity(changesets.len());
        for (i, changeset) in changesets.into_iter().enumerate() {
            let result = changeset.update(conn)?;
            *(&mut results[i]) = result;
        }
        Ok(results)
    }
}

#[macro_export]
macro_rules! data {
    ($($name:ident),*) => {
        $(
            #[rocket::async_trait]
            impl<'r> rocket::data::FromData<'r> for $name {
                type Error = ();

                async fn from_data(_req: &'r rocket::request::Request<'_>, data: rocket::data::Data<'r>) -> rocket::data::Outcome<'r, Self> {
                    use rocket::data::ToByteUnit;

                    let string = match data.open(2_u8.mebibytes()).into_string().await {
                        Ok(string) => {
                            if !string.is_complete() {
                                return rocket::data::Outcome::Failure((rocket::http::Status::PayloadTooLarge, ()));
                            }
                            string
                        },
                        Err(_) => return rocket::data::Outcome::Failure((rocket::http::Status::PayloadTooLarge, ())),
                    };
                    match serde_json::from_str(&string) {
                        Ok(entity) => rocket::data::Outcome::Success(entity),
                        Err(_) => rocket::data::Outcome::Failure((rocket::http::Status::UnprocessableEntity, ())),
                    }
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", serde_json::to_string_pretty(self).or(Err(std::fmt::Error))?)
                }
            }
        )*
    }
}

macro_rules! crud {
    ($(
        $table_name:expr,
        $table:ident,
        $($belongs_to:ident $foreign_key:expr)*,
        struct $name:ident {
            $($field_name:ident: $field_type:ty,)*
        }
    ),*) => {
        mashup! {
            $(
                Post[$name "Post"] = $name Post;
                Patch[$name "Patch"] = $name Patch;
            )*
        }

        $(
            #[derive(Associations, Debug, Deserialize, Identifiable, Queryable, Serialize)]
            $(#[belongs_to($belongs_to, foreign_key = $foreign_key)])*
            #[table_name = $table_name]
            pub struct $name {
                pub id: i32,
                $(pub $field_name: $field_type,)*
            }

            data! { $name }

            impl $name {
                pub fn find(conn: &PgConnection, id: i32) -> Result<$name> {
                    $crate::schema::$table::table.find(id)
                        .get_result(conn)
                        .map_err(Debug)
                }

                pub fn batch_find(conn: &PgConnection, ids: Vec<i32>) -> Result<Vec<$name>> {
                    let mut results = Vec::<$name>::with_capacity(ids.len());
                    for (i, id) in ids.into_iter().enumerate() {
                        let result = $crate::schema::$table::table.find(id)
                            .get_result(conn)
                            .map_err(Debug)?;

                        *(&mut results[i]) = result;
                    }
                    Ok(results)
                }
            }

            Post! {
                #[derive(Debug, Deserialize, Insertable, Serialize)]
                #[table_name = $table_name]
                pub struct $name "Post" {
                    $(pub $field_name: $field_type,)*
                }

                data! { $name "Post" }

                impl NestedInsertable for $name "Post" {
                    type Base = $name;
                    fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
                        diesel::insert_into($crate::schema::$table::table)
                            .values(self)
                            .get_result(conn)
                            .map_err(Debug)
                    }

                    fn batch_insert(conn: &PgConnection, insertables: Vec<Self>) -> Result<Vec<Self::Base>> {
                        diesel::insert_into($crate::schema::$table::table)
                            .values(&insertables)
                            .get_results(conn)
                            .map_err(Debug)
                    }
                }
            }

            Patch! {
                #[derive(AsChangeset, Debug, Deserialize, Identifiable, Queryable, Serialize)]
                #[table_name = $table_name]
                pub struct $name "Patch" {
                    pub id: i32,
                    $(pub $field_name: Option<$field_type>,)*
                }

                data! { $name "Patch" }

                impl NestedChangeset for $name "Patch" {
                    type Base = $name;
                    fn update(self, conn: &PgConnection) -> Result<Self::Base> {
                        diesel::update($crate::schema::$table::table.find(self.id))
                            .set(self)
                            .get_result(conn)
                            .map_err(Debug)
                    }

                    fn batch_update(conn: &PgConnection, changesets: Vec<Self>) -> Result<Vec<Self::Base>> {
                        let mut results = Vec::<Self::Base>::with_capacity(changesets.len());
                        for (i, changeset) in changesets.into_iter().enumerate() {
                            let result = diesel::update($crate::schema::$table::table)
                                .set(changeset)
                                .get_result(conn)
                                .map_err(Debug)?;

                            *(&mut results[i]) = result;
                        }
                        Ok(results)
                    }
                }
            }
        )*
    }
}

crud! {
    "tilingtype", tilingtype,,
    struct TilingType {
        title: String,
    },

    "label", label,,
    struct Label {
        content: String,
    },

    "tiling", tiling, TilingType "tilingtypeid",
    struct Tiling {
        title: String,
        tilingtypeid: i32,
    },

    "tilinglabel", tilinglabel, Tiling "tilingid" Label "labelid",
    struct TilingLabel {
        tilingid: i32,
        labelid: i32,
    },

    "polygon", polygon,,
    struct Polygon {
        title: String,
    },

    "polygonlabel", polygonlabel, Polygon "polygonid" Label "labelid",
    struct PolygonLabel {
        polygonid: i32,
        labelid: i32,
    },

    "point", point,,
    struct Point {
        x: f64,
        y: f64,
    },

    "polygonpoint", polygonpoint, Polygon "polygonid" Point "pointid",
    struct PolygonPoint {
        polygonid: i32,
        pointid: i32,
        sequence: i32,
    },

    "atlas", atlas,,
    struct Atlas {
        tilingid: i32,
        tilingtypeid: i32,
    },

    "atlasvertex", atlasvertex,,
    struct AtlasVertex {
        atlasid: i32,
        title: Option<String>,
    },

    "atlasvertexprototile", atlasvertexprototile,,
    struct AtlasVertexProtoTile {
        atlasvertexid: i32,
        polygonpointid: i32,
    },

    "atlasedge", atlasedge,,
    struct AtlasEdge {
        sourceid: i32,
        sinkid: i32,
    }
}
