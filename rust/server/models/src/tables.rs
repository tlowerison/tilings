use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use diesel::{
    self,
    PgConnection,
    prelude::*,
    result::Error::QueryBuilderError,
};
#[cfg(not(target_arch = "wasm32"))] use result::{Error, Result};
#[cfg(not(target_arch = "wasm32"))] use schema::*;
#[cfg(not(target_arch = "wasm32"))] use std::hash::{Hash, Hasher};

#[cfg(target_arch = "wasm32")]
mod internal {
    #[macro_export]
    macro_rules! from_data {
        ($($name:ident),*) => {
            $(
                impl std::fmt::Display for $name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", serde_json::to_string_pretty(self).or(Err(std::fmt::Error))?)
                    }
                }
            )*
        }
    }

    #[macro_export]
    macro_rules! crud {
        ($(
            $table_name:expr,
            $table:ident,
            $($belongs_to:ident)*,
            $(#[$struct_meta:meta])*
            struct $name:ident {
                $(
                    $(#[$field_meta:meta])*
                    $({ $default_val:literal, $default_opt_val:literal })?
                    $field_name:ident: $field_type:ty,
                )*
            }
        ),*) => {
            paste! {
                $(
                    #[derive(Clone, Debug, Deserialize, Serialize)]
                    pub struct $name {
                        pub id: i32,
                        $(
                            $(#[$field_meta])*
                            $(#[serde(default = $default_val)])?
                            pub $field_name: $field_type,
                        )*
                    }

                    #[derive(Clone, Debug, Deserialize, Serialize)]
                    $(#[$struct_meta])*
                    pub struct [<$name Post>] {
                        $(
                            $(#[$field_meta])*
                            $(#[serde(default = $default_val)])?
                            pub $field_name: $field_type,
                        )*
                    }

                    #[derive(Clone, Debug, Deserialize, Serialize)]
                    $(#[$struct_meta])*
                    pub struct [<$name Patch>] {
                        pub id: i32,
                        $(
                            $(#[$field_meta])*
                            $(#[serde(default = $default_opt_val)])?
                            pub $field_name: Option<$field_type>,
                        )*
                    }
                )*
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    use diesel::{self, PgConnection};
    use result::Result;

    pub trait Full: Sized {
        fn find(id: i32, conn: &PgConnection) -> Result<Self>;
        fn delete(id: i32, conn: &PgConnection) -> Result<usize>;
        fn find_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<Vec<Self>>;
        fn delete_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<usize>;
    }

    pub trait FullInsertable {
        type Base;

        fn insert(self, conn: &PgConnection) -> Result<Self::Base>;

        fn insert_batch(insertables: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> where Self: Sized {
            insertables.into_iter().map(|insertable| insertable.insert(conn))
                .collect::<Result<Vec<Self::Base>>>()
        }
    }

    pub trait FullChangeset {
        type Base;

        fn update(self, conn: &PgConnection) -> Result<Self::Base>;

        fn update_batch(changesets: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> where Self: Sized {
            changesets.into_iter().map(|changeset| changeset.update(conn)).collect()
        }
    }

    #[macro_export]
    macro_rules! from_data {
        ($($name:ident),*) => {
            $(
                #[rocket::async_trait]
                impl<'r> rocket::data::FromData<'r> for $name {
                    type Error = ();

                    async fn from_data(_req: &'r rocket::request::Request<'_>, data: rocket::data::Data<'r>) -> rocket::data::Outcome<'r, Self> {
                        use rocket::data::ToByteUnit;

                        let string = match data.open(2_u8.mebibytes()).into_string().await {
                            Err(_) => return rocket::data::Outcome::Failure((rocket::http::Status::PayloadTooLarge, ())),
                            Ok(string) => {
                                if !string.is_complete() {
                                    return rocket::data::Outcome::Failure((rocket::http::Status::PayloadTooLarge, ()));
                                }
                                string.into_inner()
                            },
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

    #[macro_export]
    macro_rules! crud {
        ($(
            $table_name:expr,
            $table:ident,
            $($belongs_to:ident)*,
            $(#[$struct_meta:meta])*
            struct $name:ident {
                $(
                    $(#[$field_meta:meta])*
                    $({ $default_val:literal, $default_opt_val:literal })?
                    $field_name:ident: $field_type:ty,
                )*
            }
        ),*) => {
            use crate::from_data;
            paste! {
                $(
                    #[derive(Associations, Clone, Debug, Deserialize, Identifiable, Queryable, Serialize)]
                    #[table_name = $table_name]
                    $(#[belongs_to($belongs_to)])*
                    $(#[$struct_meta])*
                    pub struct $name {
                        pub id: i32,
                        $(
                            $(#[$field_meta])*
                            $(#[serde(default = $default_val)])?
                            pub $field_name: $field_type,
                        )*
                    }

                    from_data! { $name }

                    impl Full for $name {
                        fn find(id: i32, conn: &PgConnection) -> Result<$name> {
                            schema::$table::table.find(id)
                                .get_result(conn)
                                .map_err(Error::from)
                        }

                        fn delete(id: i32, conn: &PgConnection) -> Result<usize> {
                            diesel::delete(
                                schema::$table::table.filter(schema::$table::id.eq(id))
                            )
                                .execute(conn)
                                .map_err(Error::from)
                        }

                        fn find_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<Vec<$name>> {
                            schema::$table::table.filter(schema::$table::id.eq_any(ids))
                                .load(conn)
                                .map_err(Error::from)
                        }

                        fn delete_batch(ids: Vec<i32>, conn: &PgConnection) -> Result<usize> {
                            diesel::delete(
                                schema::$table::table.filter(schema::$table::id.eq_any(ids))
                            )
                                .execute(conn)
                                .map_err(Error::from)
                        }
                    }

                    impl PartialEq for $name {
                        fn eq(&self, other: &$name) -> bool {
                            self.id == other.id
                        }
                    }

                    impl Eq for $name {}

                    impl Hash for $name {
                        fn hash<H: Hasher>(&self, state: &mut H) {
                            self.id.hash(state);
                        }
                    }

                    impl $name {
                        pub fn find_all(start_id: Option<i32>, end_id: Option<i32>, limit: u32, conn: &PgConnection) -> Result<Vec<$name>> {
                            let query = schema::$table::table
                                .limit(limit as i64)
                                .filter(schema::$table::id.ge(
                                    match start_id { Some(start_id) => start_id, None => 0 },
                                ));

                            if let Some(end_id) = end_id {
                                return query
                                    .filter(schema::$table::id.lt(end_id))
                                    .get_results(conn)
                                    .map_err(Error::from);
                            }

                            query.get_results(conn)
                                .map_err(Error::from)
                        }
                    }

                    #[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
                    #[table_name = $table_name]
                    $(#[$struct_meta])*
                    pub struct [<$name Post>] {
                        $(
                            $(#[$field_meta])*
                            $(#[serde(default = $default_val)])?
                            pub $field_name: $field_type,
                        )*
                    }

                    from_data! { [<$name Post>] }

                    impl FullInsertable for [<$name Post>] {
                        type Base = $name;

                        fn insert(self, conn: &PgConnection) -> Result<Self::Base> {
                            diesel::insert_into(schema::$table::table)
                                .values(self)
                                .get_result(conn)
                                .map_err(Error::from)
                        }

                        fn insert_batch(insertables: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> {
                            diesel::insert_into(schema::$table::table)
                                .values(&insertables)
                                .get_results(conn)
                                .map_err(Error::from)
                        }
                    }

                    #[derive(AsChangeset, Clone, Debug, Deserialize, Identifiable, Queryable, Serialize)]
                    #[table_name = $table_name]
                    $(#[$struct_meta])*
                    pub struct [<$name Patch>] {
                        pub id: i32,
                        $(
                            $(#[$field_meta])*
                            $(#[serde(default = $default_opt_val)])?
                            pub $field_name: Option<$field_type>,
                        )*
                    }

                    from_data! { [<$name Patch>] }

                    impl FullChangeset for [<$name Patch>] {
                        type Base = $name;

                        fn update(self, conn: &PgConnection) -> Result<Self::Base> {
                            let id = self.id.clone();
                            let result = diesel::update(schema::$table::table.find(id))
                                .set(self)
                                .get_result(conn)
                                .optional();
                            match result {
                                Ok(row) => match row {
                                    Some(result) => Ok(result),
                                    None => Self::Base::find(id, conn),
                                },
                                Err(err) => {
                                    if let QueryBuilderError(_) = err {
                                        return Self::Base::find(id, conn)
                                    }
                                    return Err(Error::from(err))
                                }
                            }
                        }

                        fn update_batch(changesets: Vec<Self>, conn: &PgConnection) -> Result<Vec<Self::Base>> {
                            changesets.into_iter().map(|changeset| changeset.update(conn))
                                .collect::<Result<Vec<Self::Base>>>()
                        }
                    }
                )*
            }
        }
    }
}

pub use self::internal::*;
use crate::crud;

fn none_bool() -> Option<bool> { None }
fn none_i32() -> Option<i32> { None }
fn none_opt_string() -> Option<Option<String>> { None }

pub fn default_account_verified() -> bool { false }
pub fn default_account_verification_code() -> Option<String> { None }
pub fn default_atlas_tiling_type_id() -> i32 { 2 }

crud! {
    "account", account,,
    struct Account {
        email: String,
        password: String,
        display_name: String,
        #[serde(skip_deserializing)] { "default_account_verified", "none_bool" }
        verified: bool,
        #[serde(skip_deserializing)] { "default_account_verification_code", "none_opt_string" }
        verification_code: Option<String>,
    },

    "accountrole", accountrole, Account Role,
    struct AccountRole {
        account_id: i32,
        role_id: i32,
    },

    "apikey", apikey, Account,
    struct APIKey {
        account_id: i32,
        content: String,
    },

    "atlas", atlas, Tiling TilingType,
    struct Atlas {
        tiling_id: i32,
        #[serde(skip_deserializing)] { "default_atlas_tiling_type_id", "none_i32" }
        tiling_type_id: i32,
    },

    "atlasedge", atlasedge, Atlas PolygonPoint,
    struct AtlasEdge {
        atlas_id: i32,
        polygon_point_id: i32,
        source_id: i32,
        sink_id: i32,
        parity: bool,
        sequence: i32,
        neighbor_edge_id: Option<i32>,
    },

    "atlasvertex", atlasvertex, Atlas,
    struct AtlasVertex {
        atlas_id: i32,
    },

    "label", label,,
    struct Label {
        content: String,
    },

    "point", point,,
    struct Point {
        x: f64,
        y: f64,
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

    "polygonpoint", polygonpoint, Polygon Point,
    struct PolygonPoint {
        polygon_id: i32,
        point_id: i32,
        sequence: i32,
    },

    "role", role,,
    struct Role {
        title: String,
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

    "tilingtype", tilingtype,,
    struct TilingType {
        title: String,
    }
}
