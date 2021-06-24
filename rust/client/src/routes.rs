extern crate console_error_panic_hook;
extern crate serde_json;

use models;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Request,
    RequestInit,
    RequestMode,
    Response,
};

const BASE_URL: &'static str = "http://localhost:8000";

#[derive(Debug, Deserialize, Serialize)]
enum Error {
    Default,
    Serde,
    Query,
    Url,
}

impl Error {
    fn js_value(&self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

fn url(path: &str) -> String {
    format!("{}{}", BASE_URL, path)
}

fn clean_query(url: String) -> Result<String, JsValue> {
    let split_url = url.split("?").collect::<Vec<&str>>();

    let mut new_query = String::from("");

    let query = match split_url.get(1) {
        Some(query) => query,
        None => return Ok(url),
    };

    let empty_param_blocks = query.split("=&").collect::<Vec<&str>>();

    for empty_param_block in empty_param_blocks.iter() {
        let mut params = empty_param_block.split("&").collect::<Vec<&str>>();

        let last = params.pop().unwrap();
        if &last[last.len()-1..] != "=" {
            params.push(last);
        }
        let cleaned_param_block = params.join("&");

        new_query = format!(
            "{}{}{}",
            new_query,
            (if new_query == "" || cleaned_param_block == "" { "" } else { "&" }),
            cleaned_param_block,
        );
    }

    let base_url = String::from(*split_url.get(0).ok_or(Error::Query.js_value())?);

    if new_query.len() == 0 {
        Ok(base_url)
    } else {
        Ok(String::from(vec![base_url, new_query].join("?")))
    }
}

macro_rules! get_delete {
    ($(
        $method:expr,
        $route:expr,
        $fn_name:ident,
        $exp_fn_name:ident,
        $return_type:ty,
        Params { $($param_name:ident: $param_type:ty,)* },
        Query { $($arg_name:ident: $arg_type:ty,)* },
    )*) => {
        paste! {
            $(
                #[wasm_bindgen]
                #[allow(non_snake_case)]
                pub async fn $exp_fn_name(
                    $($param_name: JsValue,)*
                    $($arg_name: JsValue,)*
                ) -> Result<JsValue, JsValue> {
                    let value = $fn_name(
                        $($param_name.into_serde().or(Err(JsValue::from_str(stringify!($param_name))))?,)*
                        $($arg_name.into_serde().or(Err(JsValue::from_str(stringify!($arg_name))))?,)*
                    ).await?;
                    Ok(JsValue::from_serde(&value).or(Err(Error::Serde.js_value()))?)
                }

                pub async fn $fn_name(
                    $($param_name: $param_type,)*
                    $($arg_name: Option<$arg_type>,)*
                ) -> Result<$return_type, JsValue> {
                    panic::set_hook(Box::new(console_error_panic_hook::hook));

                    let mut opts = RequestInit::new();

                    opts.method($method);

                    opts.mode(RequestMode::Cors);

                    let url = clean_query(url(&format!(
                        $route,
                        $($param_name,)*
                        $(match $arg_name { None => String::from(""), Some(val) => format!("{}", val) },)*
                    )))?;

                    let request = Request::new_with_str_and_init(&url, &opts)
                        .or(Err(Error::Url.js_value()))?;

                    request
                        .headers()
                        .set("Accept", "application/json")
                        .or(Err(Error::Url.js_value()))?;

                    let window = web_sys::window().unwrap();

                    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

                    let resp: Response = resp_value.dyn_into().unwrap();

                    // Convert this other `Promise` into a rust `Future`.
                    let json = JsFuture::from(resp.json()?).await?;

                    // Use serde to parse the JSON into a struct.
                    let value: $return_type = json.into_serde().unwrap();

                    Ok(value)
                }
            )*
        }
    }
}

macro_rules! post_patch {
    ($(
        $method:expr,
        $route:expr,
        $fn_name:ident,
        $exp_fn_name:ident,
        $return_type:ty,
        Params { $($param_name:ident: $param_type:ty,)* },
        Query { $($arg_name:ident: $arg_type:ty,)* },
        Data $($data_name:ident: $data_type:ty)?,
    )*) => {
        paste! {
            $(
                #[wasm_bindgen]
                #[allow(non_snake_case)]
                pub async fn $exp_fn_name(
                    $($data_name: JsValue,)?
                    $($param_name: JsValue,)*
                    $($arg_name: JsValue,)*
                ) -> Result<JsValue, JsValue> {
                    let value = $fn_name(
                        $($data_name.into_serde().or(Err(JsValue::from_str(stringify!($data_name))))?,)?
                        $($param_name.into_serde().or(Err(JsValue::from_str(stringify!($param_name))))?,)*
                        $($arg_name.into_serde().or(Err(JsValue::from_str(stringify!($arg_name))))?,)*
                    ).await?;
                    Ok(JsValue::from_serde(&value).or(Err(Error::Serde.js_value()))?)
                }

                pub async fn $fn_name(
                    $($data_name: $data_type,)?
                    $($param_name: $param_type,)*
                    $($arg_name: Option<$arg_type>,)*
                ) -> Result<$return_type, JsValue> {
                    panic::set_hook(Box::new(console_error_panic_hook::hook));

                    let mut opts = RequestInit::new();

                    opts.method($method);

                    opts.mode(RequestMode::Cors);

                    $(opts.body(Some(&JsValue::from_serde(&$data_name)
                        .or(Err(Error::Serde.js_value()))?));)?

                    let url = clean_query(url(&format!(
                        $route,
                        $($($param_name,)*)?
                        $($(match $arg_name { None => String::from(""), Some(val) => &format!("{}", val) },)*)?
                    )))?;

                    let request = Request::new_with_str_and_init(&url, &opts)
                        .or(Err(Error::Url.js_value()))?;

                    request
                        .headers()
                        .set("Accept", "application/json")
                        .or(Err(Error::Url.js_value()))?;

                    let window = web_sys::window().unwrap();

                    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

                    let resp: Response = resp_value.dyn_into().unwrap();

                    // Convert this other `Promise` into a rust `Future`.
                    let json = JsFuture::from(resp.json()?).await?;

                    // Use serde to parse the JSON into a struct.
                    let value: $return_type = json.into_serde().unwrap();

                    Ok(value)
                }
            )*
        }
    }
}

get_delete! {
    "GET", "/check-display-name/{}", check_display_name, checkDisplayName,
    bool,
    Params {
        display_name: String,
    },
    Query {},

    "GET", "/check-email/{}", check_email, checkEmail,
    bool,
    Params {
        email: String,
    },
    Query {},

    "DELETE", "/label/{}", delete_label, deleteLabel,
    usize,
    Params {
        id: i32,
    },
    Query {},

    "DELETE", "/polygon/{}", delete_polygon, deletePolygon,
    usize,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/atlas/{}", get_atlas, getAtlas,
    models::FullAtlas,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/atlases", get_atlases, getAtlases,
    Vec<models::FullAtlas>,
    Params {},
    Query {},

    "GET", "/labels?start_id={}&end_id={}&limit={}", get_labels, getLabels,
    Vec<models::Label>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },

    "GET", "/match-labels?query={}", match_labels, matchLabels,
    Vec<models::Label>,
    Params {},
    Query {
        query: String,
    },

    "GET", "/omni-search?query={}", omni_search, omniSearch,
    Vec<models::TextSearchItem>,
    Params {},
    Query {
        query: String,
    },

    "GET", "/polygon/{}", get_polygon, getPolygon,
    models::FullPolygon,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/polygons?start_id={}&end_id={}&limit={}", get_polygons, getPolygons,
    Vec<models::FullPolygon>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },

    "GET", "/reset-api-key", reset_api_key, resetApiKey,
    String,
    Params {},
    Query {},

    "GET", "/tiling/{}", get_tiling, getTiling,
    models::FullTiling,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/tilings?start_id={}&end_id={}&limit={}", get_tilings, getTilings,
    Vec<models::Tiling>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },

    "GET", "/tiling-search?query={}", tiling_search, tilingSearch,
    Vec<models::TextSearchItem>,
    Params {},
    Query {
        query: String,
    },

    "GET", "/tiling-type/{}", get_tiling_type, getTilingType,
    models::TilingType,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/tiling-types?start_id={}&end_id={}&limit={}", get_tiling_types, getTilingTypes,
    Vec<models::TilingType>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },
}

post_patch! {
    "POST", "/add-label-to-polygon", add_label_to_polygon, addLabelToPolygon,
    (),
    Params {},
    Query {},
    Data polygon_label: models::PolygonLabel,

    "POST", "/create-polygon", create_polygon, createPolygon,
    models::FullPolygon,
    Params {},
    Query {},
    Data polygon_post: models::FullPolygonPost,

    "POST", "/sign-in", sign_in, signIn,
    (),
    Params {},
    Query {},
    Data sign_in_post: models::SignInPost,

    "POST", "/sign-out", sign_out, signOut,
    (),
    Params {},
    Query {},
    Data,

    "POST", "/sign-up", sign_up, signUp,
    (),
    Params {},
    Query {},
    Data sign_in_post: models::AccountPost,

    "POST", "/update-polygon", update_polygon, updatePolygon,
    (),
    Params {},
    Query {},
    Data polygon_patch: models::FullPolygonPatch,

    "POST", "/upsert-label", upsert_label, upsertLabel,
    (),
    Params {},
    Query {},
    Data label: String,
}
