use crate::{connection::Result, schema::*};
use diesel::{
    self,
    ExpressionMethods,
    Insertable,
    OptionalExtension,
    PgConnection,
    QueryDsl,
    result::Error::QueryBuilderError,
    RunQueryDsl,
};
use rocket::response::Debug;
use serde::{Deserialize, Serialize};
use serde_json;

pub trait NestedInsertable {
    type Base;

    fn insert(self, conn: &PgConnection) -> Result<Self::Base>;

    fn batch_insert(insertables: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> where Self: Sized {
        insertables.into_iter().map(|insertable| insertable.insert(conn)).collect()
    }
}

pub trait NestedChangeset {
    type Base;

    fn update(self, conn: &PgConnection) -> Result<Self::Base>;

    fn batch_update(changesets: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> where Self: Sized {
        changesets.into_iter().map(|changeset| changeset.update(conn)).collect()
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
        $($belongs_to:ident)*,
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
            #[derive(Associations, Clone, Debug, Deserialize, Identifiable, Queryable, Serialize)]
            $(#[belongs_to($belongs_to)])*
            #[table_name = $table_name]
            pub struct $name {
                pub id: i32,
                $(pub $field_name: $field_type,)*
            }

            data! { $name }

            impl $name {
                pub fn find(id: i32, conn: &PgConnection) -> Result<$name> {
                    $crate::schema::$table::table.find(id)
                        .get_result(conn)
                        .map_err(Debug)
                }

                pub fn batch_find(ids: Vec<i32>, conn: &PgConnection) -> Result<Vec<$name>> {
                    $crate::schema::$table::table.filter($crate::schema::$table::id.eq_any(ids))
                        .load(conn)
                        .map_err(Debug)
                }

                pub fn delete(id: i32, conn: &PgConnection) -> Result<usize> {
                    diesel::delete(
                        $crate::schema::$table::table.filter($crate::schema::$table::id.eq(id))
                    )
                        .execute(conn)
                        .map_err(Debug)
                }

                pub fn batch_delete(ids: Vec<i32>, conn: &PgConnection) -> Result<usize> {
                    diesel::delete(
                        $crate::schema::$table::table.filter($crate::schema::$table::id.eq_any(ids))
                    )
                        .execute(conn)
                        .map_err(Debug)
                }
            }

            Post! {
                #[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
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

                    fn batch_insert(insertables: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> {
                        diesel::insert_into($crate::schema::$table::table)
                            .values(&insertables)
                            .get_results(conn)
                            .map_err(Debug)
                    }
                }
            }

            Patch! {
                #[derive(AsChangeset, Clone, Debug, Deserialize, Identifiable, Queryable, Serialize)]
                #[table_name = $table_name]
                pub struct $name "Patch" {
                    pub id: i32,
                    $(pub $field_name: Option<$field_type>,)*
                }

                data! { $name "Patch" }

                impl NestedChangeset for $name "Patch" {
                    type Base = $name;

                    fn update(self, conn: &PgConnection) -> Result<Self::Base> {
                        let id = self.id.clone();
                        let result = diesel::update($crate::schema::$table::table.find(id))
                            .set(self)
                            .get_result(conn)
                            .optional();
                        match result {
                            Ok(row) => match row {
                                Some(result) => Ok(result),
                                None => Self::Base::find(id, conn),
                            },
                            Err(e) => {
                                if let QueryBuilderError(_) = e {
                                    return Self::Base::find(id, conn)
                                }
                                return Err(Debug(e))
                            }
                        }
                    }

                    fn batch_update(changesets: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> {
                        changesets.into_iter().map(|changeset| changeset.update(conn)).collect()
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

    "tiling", tiling, TilingType,
    struct Tiling {
        title: String,
        tiling_type_id: i32,
    },

    "tilinglabel", tilinglabel, Tiling Label,
    struct TilingLabel {
        tiling_id: i32,
        label_id: i32,
    },

    "polygon", polygon,,
    struct Polygon {
        title: String,
    },

    "polygonlabel", polygonlabel, Polygon Label,
    struct PolygonLabel {
        polygon_id: i32,
        label_id: i32,
    },

    "point", point,,
    struct Point {
        x: f64,
        y: f64,
    },

    "polygonpoint", polygonpoint, Polygon Point,
    struct PolygonPoint {
        polygon_id: i32,
        point_id: i32,
        sequence: i32,
    },

    "atlas", atlas,,
    struct Atlas {
        tiling_id: i32,
        tiling_type_id: i32,
    },

    "atlasvertex", atlasvertex,,
    struct AtlasVertex {
        atlas_id: i32,
        title: Option<String>,
    },

    "atlasvertexprototile", atlasvertexprototile,,
    struct AtlasVertexProtoTile {
        atlas_vertex_id: i32,
        polygon_point_id: i32,
    },

    "atlasedge", atlasedge,,
    struct AtlasEdge {
        source_id: i32,
        sink_id: i32,
    }
}
