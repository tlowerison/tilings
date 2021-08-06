use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use diesel::{
    self,
    PgConnection,
    prelude::*,
    result::Error::QueryBuilderError,
};

#[cfg(not(target_arch = "wasm32"))]
use result::{Error, Result};

#[cfg(not(target_arch = "wasm32"))]
use schema::*;

#[cfg(not(target_arch = "wasm32"))]
use std::hash::{Hash, Hasher};

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

pub fn none_bool() -> Option<bool> { None }
pub fn none_i32() -> Option<i32> { None }
pub fn none_opt_datetime() -> Option<Option<NaiveDateTime>> { None }
pub fn none_opt_i32() -> Option<Option<i32>> { None }
pub fn none_opt_string() -> Option<Option<String>> { None }

pub fn default_account_password_reset_code() -> Option<String> { None }
pub fn default_account_password_reset_code_timestamp() -> Option<NaiveDateTime> { None }
pub fn default_account_verification_code() -> Option<String> { None }
pub fn default_atlas_tiling_type_id() -> i32 { 2 }
pub fn default_false() -> bool { false }

crud! {
    "account", account,,
    struct Account {
        email: String,
        #[serde(skip_serializing)]
        password: String,
        #[serde(rename = "displayName")]
        display_name: String,
        #[serde(skip_deserializing)] { "default_false", "none_bool" }
        verified: bool,
        #[serde(skip_deserializing, skip_serializing)] { "default_account_verification_code", "none_opt_string" }
        verification_code: Option<String>,
        #[serde(skip_deserializing, skip_serializing)] { "default_account_password_reset_code", "none_opt_string" }
        password_reset_code: Option<String>,
        #[serde(skip_deserializing, skip_serializing)] { "default_account_password_reset_code_timestamp", "none_opt_datetime" }
        password_reset_code_timestamp: Option<NaiveDateTime>,
    },

    "accountrole", accountrole, Account Role,
    struct AccountRole {
        #[serde(rename = "accountId")]
        account_id: i32,
        #[serde(rename = "roleId")]
        role_id: i32,
    },

    "apikey", apikey, Account,
    struct APIKey {
        #[serde(rename = "accountId")]
        account_id: i32,
        content: String,
    },

    "atlas", atlas, Tiling TilingType,
    struct Atlas {
        #[serde(rename = "tilingId")]
        tiling_id: i32,
        #[serde(rename = "tilingTypeId", skip_deserializing)] { "default_atlas_tiling_type_id", "none_i32" }
        tiling_type_id: i32,
    },

    "atlasedge", atlasedge, Atlas PolygonPoint,
    struct AtlasEdge {
        #[serde(rename = "atlasId")]
        atlas_id: i32,
        #[serde(rename = "polygonPointId")]
        polygon_point_id: i32,
        #[serde(rename = "sourceId")]
        source_id: i32,
        #[serde(rename = "sinkId")]
        sink_id: i32,
        parity: bool,
        sequence: i32,
        #[serde(rename = "neighborEdgeId")]
        neighbor_edge_id: Option<i32>,
    },

    "atlasvertex", atlasvertex, Atlas,
    struct AtlasVertex {
        #[serde(rename = "atlasId")]
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
        #[serde(rename = "ownerId", skip_deserializing, skip_serializing)] { "none_i32", "none_opt_i32" }
        owner_id: Option<i32>,
    },

    "polygonlabel", polygonlabel, Polygon Label,
    struct PolygonLabel {
        #[serde(rename = "polygonId")]
        polygon_id: i32,
        #[serde(rename = "labelId")]
        label_id: i32,
    },

    "polygonpoint", polygonpoint, Polygon Point,
    struct PolygonPoint {
        #[serde(rename = "polygonId")]
        polygon_id: i32,
        #[serde(rename = "pointId")]
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
        #[serde(rename = "tilingTypeId")]
        tiling_type_id: i32,
        #[serde(rename = "ownerId", skip_deserializing, skip_serializing)] { "none_i32", "none_opt_i32" }
        owner_id: Option<i32>,
    },

    "tilinglabel", tilinglabel, Tiling Label,
    struct TilingLabel {
        #[serde(rename = "tilingId")]
        tiling_id: i32,
        #[serde(rename = "labelId")]
        label_id: i32,
    },

    "tilingtype", tilingtype,,
    struct TilingType {
        title: String,
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impls {
    use super::*;
    use lazy_static::lazy_static;
    use std::collections::hash_set::HashSet;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub enum RoleEnum {
        Admin,
        Editor,
        ReadOnly,
    }

    impl RoleEnum {
        pub fn as_role_id(self) -> i32 {
            match self {
                RoleEnum::Admin => 3,
                RoleEnum::Editor => 2,
                RoleEnum::ReadOnly => 1,
            }
        }
    }

    impl std::fmt::Display for RoleEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                RoleEnum::Admin => write!(f, "Admin"),
                RoleEnum::Editor => write!(f, "Editor"),
                RoleEnum::ReadOnly => write!(f, "ReadOnly"),
            }
        }
    }

    impl AccountRole {
        pub fn as_role_enum(account_role: AccountRole) -> Option<RoleEnum> {
            match account_role.role_id {
                1 => Some(RoleEnum::ReadOnly),
                2 => Some(RoleEnum::Editor),
                3 => Some(RoleEnum::Admin),
                _ => None,
            }
        }
    }

    lazy_static! {
        pub static ref ALLOWED_EDITOR_ROLES: HashSet<RoleEnum> = [RoleEnum::Editor, RoleEnum::Admin].iter().cloned().collect();
        pub static ref ALLOWED_ADMIN_ROLES: HashSet<RoleEnum> = [RoleEnum::Admin].iter().cloned().collect();
    }

    pub enum Owned {
        Atlas,
        Polygon,
        Tiling,
    }

    impl Owned {
        pub fn get_owner_id(&self, id: i32, conn: &PgConnection) -> Result<Option<i32>> {
            match self {
                Owned::Atlas => atlas::table.filter(atlas::id.eq(id))
                    .inner_join(tiling::table)
                    .select(tiling::owner_id)
                    .get_result(conn)
                    .map_err(Error::from),

                Owned::Polygon => polygon::table.filter(polygon::id.eq(id))
                    .select(polygon::owner_id)
                    .get_result(conn)
                    .map_err(Error::from),

                Owned::Tiling => tiling::table.filter(tiling::id.eq(id))
                    .select(tiling::owner_id)
                    .get_result(conn)
                    .map_err(Error::from),
            }
        }

        pub fn lock(&self, id: i32, conn: &PgConnection) -> Result<()> {
            match self {
                Owned::Atlas => {
                    let tiling_id = atlas::table.filter(atlas::id.eq(id))
                        .select(atlas::tiling_id)
                        .get_result(conn)?;
                    Owned::Tiling.lock(tiling_id, conn)
                },

                Owned::Polygon => PolygonPatch {
                    id,
                    owner_id: Some(None),
                    title: None,
                }.update(conn).and(Ok(())),

                Owned::Tiling => TilingPatch {
                    id,
                    owner_id: Some(None),
                    title: None,
                    tiling_type_id: None,
                }.update(conn).and(Ok(())),
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::impls::*;
