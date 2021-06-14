use super::schema::*;
use rocket::{
    data::{Data, FromData, Outcome, ToByteUnit},
    http::Status,
    request::Request,
};
use serde::{Deserialize, Serialize};
use serde_json;

macro_rules! crud {
    ($(
        $table_name:expr,
        struct $name:ident {
            $($field_name:ident: $field_type:ty,)*
        }
    ),*) => {
        mashup! {
            $(
                Post[$name] = $name Post;
                Patch[$name] = $name Patch;
            )*
        }

        $(
            #[derive(Debug, Deserialize, Queryable, Serialize)]
            pub struct $name {
                pub id: i32,
                $(pub $field_name: $field_type,)*
            }

            #[rocket::async_trait]
            impl<'r> FromData<'r> for $name {
                type Error = ();

                async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
                    let string = match data.open(2_u8.mebibytes()).into_string().await {
                        Ok(string) => {
                            if !string.is_complete() {
                                return Outcome::Failure((Status::PayloadTooLarge, ()));
                            }
                            string
                        },
                        Err(_) => return Outcome::Failure((Status::PayloadTooLarge, ())),
                    };
                    match serde_json::from_str(&string) {
                        Ok(entity) => Outcome::Success(entity),
                        Err(_) => Outcome::Failure((Status::UnprocessableEntity, ())),
                    }
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", serde_json::to_string_pretty(self).or(Err(std::fmt::Error))?)
                }
            }

            Post! {
                #[derive(Debug, Deserialize, Insertable, Serialize)]
                #[table_name = $table_name]
                pub struct $name {
                    $(pub $field_name: $field_type,)*
                }

                #[rocket::async_trait]
                impl<'r> FromData<'r> for $name {
                    type Error = ();

                    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
                        let string = match data.open(2_u8.mebibytes()).into_string().await {
                            Ok(string) => {
                                if !string.is_complete() {
                                    return Outcome::Failure((Status::PayloadTooLarge, ()));
                                }
                                string
                            },
                            Err(_) => return Outcome::Failure((Status::PayloadTooLarge, ())),
                        };
                        match serde_json::from_str(&string) {
                            Ok(entity) => Outcome::Success(entity),
                            Err(_) => Outcome::Failure((Status::UnprocessableEntity, ())),
                        }
                    }
                }

                impl std::fmt::Display for $name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", serde_json::to_string_pretty(self).or(Err(std::fmt::Error))?)
                    }
                }
            }

            Patch! {
                #[derive(AsChangeset, Debug, Deserialize, Queryable, Serialize)]
                #[table_name = $table_name]
                pub struct $name {
                    $(pub $field_name: Option<$field_type>,)*
                }

                #[rocket::async_trait]
                impl<'r> FromData<'r> for $name {
                    type Error = ();

                    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
                        let string = match data.open(2_u8.mebibytes()).into_string().await {
                            Ok(string) => {
                                if !string.is_complete() {
                                    return Outcome::Failure((Status::PayloadTooLarge, ()));
                                }
                                string
                            },
                            Err(_) => return Outcome::Failure((Status::PayloadTooLarge, ())),
                        };
                        match serde_json::from_str(&string) {
                            Ok(entity) => Outcome::Success(entity),
                            Err(_) => Outcome::Failure((Status::UnprocessableEntity, ())),
                        }
                    }
                }

                impl std::fmt::Display for $name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", serde_json::to_string_pretty(self).or(Err(std::fmt::Error))?)
                    }
                }
            }
        )*
    }
}

crud! {
    "label",
    struct Label {
        content: String,
    },

    "tiling",
    struct Tiling {
        title: String,
    },

    "tilinglabel",
    struct TilingLabel {
        tilingid: i32,
        labelid: i32,
    },

    "polygon",
    struct Polygon {
        title: String,
    },

    "polygonlabel",
    struct PolygonLabel {
        polygonid: i32,
        labelid: i32,
    },

    "point",
    struct Point {
        x: f64,
        y: f64,
    },

    "polygonpoint",
    struct PolygonPoint {
        polygonid: i32,
        pointid: i32,
        sequence: i32,
    },

    "atlas",
    struct Atlas {
        tilingid: i32,
    },

    "atlasvertex",
    struct AtlasVertex {
        atlasid: i32,
        title: Option<String>,
    },

    "atlasvertexprototile",
    struct AtlasVertexProtoTile {
        atlasvertexid: i32,
        polygonpointid: i32,
    },

    "atlasedge",
    struct AtlasEdge {
        sourceid: i32,
        sinkid: i32,
    }
}
