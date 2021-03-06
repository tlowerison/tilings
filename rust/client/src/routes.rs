use models;
use paste::paste;
use percent_encoding::{self, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Request,
    RequestCredentials,
    RequestInit,
    RequestMode,
    Response,
};

#[derive(Deserialize, Serialize)]
pub struct Account {
    pub id: i32,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseDetails {
    ok: bool,
    status: u16,
    #[serde(rename = "statusText")]
    status_text: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
enum Error {
    Default,
    Serde,
    Query,
    Network(ResponseDetails),
    Url,
}

impl From<&Response> for Error {
    fn from(res: &Response) -> Error {
        Error::Network(ResponseDetails {
            ok: res.ok(),
            status: res.status(),
            status_text: res.status_text(),
            url: res.url(),
        })
    }
}

pub fn percent_encode(query: String) -> String {
    percent_encoding::percent_encode(query.as_bytes(), NON_ALPHANUMERIC).to_string()
}

impl Error {
    fn js_value(&self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

fn url(path: &str) -> String {
    lazy_static! {
        static ref BASE_URL: &'static str = env!("SERVER_HOST");
    }
    format!("{}{}", *BASE_URL, path)
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
                #[wasm_bindgen(catch)]
                #[allow(non_snake_case)]
                pub async fn $exp_fn_name(
                    $($param_name: JsValue,)*
                    $($arg_name: JsValue,)*
                ) -> Result<JsValue, JsValue> {
                    let value = $fn_name(
                        $($param_name.into_serde().or_else(|_| Err(JsValue::from_str(stringify!($param_name))))?,)*
                        $($arg_name.into_serde().or_else(|_| Err(JsValue::from_str(stringify!($arg_name))))?,)*
                    ).await?;
                    Ok(JsValue::from_serde(&value).or_else(|_| Err(Error::Serde.js_value()))?)
                }

                pub async fn $fn_name(
                    $($param_name: $param_type,)*
                    $($arg_name: Option<$arg_type>,)*
                ) -> Result<$return_type, JsValue> {
                    panic::set_hook(Box::new(console_error_panic_hook::hook));

                    let window = web_sys::window().unwrap();

                    let url = clean_query(url(&format!(
                        $route,
                        $($param_name,)*
                        $(match $arg_name { None => String::from(""), Some(val) => percent_encode(format!("{}", val)) },)*
                    )))?;

                    let request = Request::new_with_str(&url)
                        .or_else(|_| Err(Error::Url.js_value()))?;

                    request
                        .headers()
                        .set("Accept", "application/json")
                        .or_else(|_| Err(Error::Url.js_value()))?;

                    let mut opts = RequestInit::new();

                    opts.method($method);
                    opts.mode(RequestMode::Cors);
                    opts.credentials(RequestCredentials::Include);

                    let response: Response = JsFuture::from(window.fetch_with_request_and_init(&request, &opts))
                        .await?
                        .dyn_into()
                        .unwrap();

                    if !response.ok() {
                        return Err(Error::from(&response).js_value());
                    }

                    // Convert this other `Promise` into a rust `Future`.
                    let json = JsFuture::from(response.json()?).await?;

                    // Use serde to parse the JSON into a struct.
                    let value: $return_type = json.into_serde().or_else(|_| Err(Error::from(&response).js_value()))?;

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
                #[wasm_bindgen(catch)]
                #[allow(non_snake_case)]
                pub async fn $exp_fn_name(
                    $($data_name: JsValue,)?
                    $($param_name: JsValue,)*
                    $($arg_name: JsValue,)*
                ) -> Result<JsValue, JsValue> {
                    let value = $fn_name(
                        $($data_name.into_serde().or_else(|_| Err(JsValue::from_str(stringify!($data_name))))?,)?
                        $($param_name.into_serde().or_else(|_| Err(JsValue::from_str(stringify!($param_name))))?,)*
                        $($arg_name.into_serde().or_else(|_| Err(JsValue::from_str(stringify!($arg_name))))?,)*
                    ).await?;
                    Ok(JsValue::from_serde(&value).or_else(|_| Err(Error::Serde.js_value()))?)
                }

                pub async fn $fn_name(
                    $($data_name: $data_type,)?
                    $($param_name: $param_type,)*
                    $($arg_name: Option<$arg_type>,)*
                ) -> Result<$return_type, JsValue> {
                    panic::set_hook(Box::new(console_error_panic_hook::hook));

                    let window = web_sys::window().unwrap();

                    let url = clean_query(url(&format!(
                        $route,
                        $($param_name,)*
                        $(match $arg_name { None => String::from(""), Some(val) => percent_encode(format!("{}", val)) },)*
                    )))?;

                    let request = Request::new_with_str(&url)
                        .or_else(|_| Err(Error::Url.js_value()))?;

                    request
                        .headers()
                        .set("Accept", "application/json")
                        .or_else(|_| Err(Error::Url.js_value()))?;

                    let mut opts = RequestInit::new();

                    opts.method($method);
                    opts.mode(RequestMode::Cors);
                    opts.credentials(RequestCredentials::Include);

                    $(
                        opts.body(Some(&JsValue::from_str(
                            &serde_json::to_string(&$data_name)
                                .or_else(|_| Err(Error::Serde.js_value()))?
                        )));
                    )?

                    let response: Response = JsFuture::from(window.fetch_with_request_and_init(&request, &opts))
                        .await?
                        .dyn_into()
                        .unwrap();

                    if !response.ok() {
                        return Err(Error::from(&response).js_value());
                    }

                    // Convert this other `Promise` into a rust `Future`.
                    let json = JsFuture::from(response.json()?).await?;

                    // Use serde to parse the JSON into a struct.
                    let value: $return_type = json.into_serde().or_else(|_| Err(Error::from(&response).js_value()))?;

                    Ok(value)
                }
            )*
        }
    }
}

get_delete! {
    "GET", "/api/tilings/v1/account", get_account, getAccount,
    Account,
    Params {},
    Query {},

    "GET", "/api/tilings/v1/account-tilings", get_account_tilings, getAccountTilings,
    Vec<models::FullTiling>,
    Params {},
    Query {},

    "GET", "/api/tilings/v1/atlas/{}", get_atlas, getAtlas,
    models::FullAtlas,
    Params {
        id: i32,
    },
    Query {},

    "DELETE", "/api/tilings/v1/atlas/{}", delete_atlas, deleteAtlas,
    usize,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/api/tilings/v1/atlases", get_atlases, getAtlases,
    Vec<models::Atlas>,
    Params {},
    Query {},

    "GET", "/api/tilings/v1/atlas-by-tiling-id/{}", get_atlas_by_tiling_id, getAtlasByTilingId,
    models::FullAtlas,
    Params {
        tiling_id: i32,
    },
    Query {},

    "GET", "/api/tilings/v1/check-display-name/{}", check_display_name, checkDisplayName,
    bool,
    Params {
        display_name: String,
    },
    Query {},

    "GET", "/api/tilings/v1/check-email/{}", check_email, checkEmail,
    bool,
    Params {
        email: String,
    },
    Query {},

    "GET", "/api/tilings/v1/check-password-reset-code/{}", check_password_reset_code, checkPasswordResetCode,
    String,
    Params {
        password_reset_code: String,
    },
    Query {},

    "DELETE", "/api/tilings/v1/label/{}", delete_label, deleteLabel,
    usize,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/api/tilings/v1/labels?start_id={}&end_id={}&limit={}", get_labels, getLabels,
    Vec<models::Label>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },

    "GET", "/api/tilings/v1/match-labels?query={}", match_labels, matchLabels,
    Vec<models::Label>,
    Params {},
    Query {
        query: String,
    },

    "GET", "/api/tilings/v1/omni-search?query={}", omni_search, omniSearch,
    Vec<models::TextSearchItem>,
    Params {},
    Query {
        query: String,
    },

    "GET", "/api/tilings/v1/polygon/{}", get_polygon, getPolygon,
    models::FullPolygon,
    Params {
        id: i32,
    },
    Query {},

    "DELETE", "/api/tilings/v1/polygon/{}", delete_polygon, deletePolygon,
    usize,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/api/tilings/v1/polygons?start_id={}&end_id={}&limit={}", get_polygons, getPolygons,
    Vec<models::Polygon>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },

    "GET", "/api/tilings/v1/reset-api-key", reset_api_key, resetApiKey,
    String,
    Params {},
    Query {},

    "GET", "/api/tilings/v1/tiling/{}", get_tiling, getTiling,
    models::FullTiling,
    Params {
        id: i32,
    },
    Query {},

    "DELETE", "/api/tilings/v1/tiling/{}", delete_tiling, deleteTiling,
    usize,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/api/tilings/v1/tilings?start_id={}&end_id={}&limit={}", get_tilings, getTilings,
    Vec<models::Tiling>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },

    "GET", "/api/tilings/v1/tiling-search?query={}", tiling_search, tilingSearch,
    Vec<models::TextSearchItem>,
    Params {},
    Query {
        query: String,
    },

    "GET", "/api/tilings/v1/tiling-type/{}", get_tiling_type, getTilingType,
    models::TilingType,
    Params {
        id: i32,
    },
    Query {},

    "GET", "/api/tilings/v1/tiling-types?start_id={}&end_id={}&limit={}", get_tiling_types, getTilingTypes,
    Vec<models::TilingType>,
    Params {},
    Query {
        start_id: i32,
        end_id: i32,
        limit: u32,
    },
}

post_patch! {
    "POST", "/api/tilings/v1/add-label-to-polygon", add_label_to_polygon, addLabelToPolygon,
    (),
    Params {},
    Query {},
    Data polygon_label: models::PolygonLabel,

    "POST", "/api/tilings/v1/atlas", create_atlas, createAtlas,
    models::FullAtlas,
    Params {},
    Query {},
    Data atlas_post: models::FullAtlasPost,

    "PATCH", "/api/tilings/v1/atlas", update_atlas, updateAtlas,
    models::FullAtlas,
    Params {},
    Query {},
    Data atlas_post: models::FullAtlasPatch,

    "POST", "/api/tilings/v1/create-polygon", create_polygon, createPolygon,
    models::FullPolygon,
    Params {},
    Query {},
    Data full_polygon_post: models::FullPolygonPost,

    "PATCH", "/api/tilings/v1/lock-atlas/{}", lock_atlas, lockAtlas,
    (),
    Params {
        id: i32,
    },
    Query {},
    Data,

    "PATCH", "/api/tilings/v1/lock-polygon/{}", lock_polygon, lockPolygon,
    (),
    Params {
        id: i32,
    },
    Query {},
    Data,

    "PATCH", "/api/tilings/v1/lock-tiling/{}", lock_tiling, lockTiling,
    (),
    Params {
        id: i32,
    },
    Query {},
    Data,

    "POST", "/api/tilings/v1/resend-verification-code-email", resend_verification_code_email, resendVerificationCodeEmail,
    (),
    Params {},
    Query {},
    Data,

    "POST", "/api/tilings/v1/send-password-reset-link/{}", send_password_reset_link, sendResetPasswordLink,
    (),
    Params {
        email: String,
    },
    Query {},
    Data,

    "POST", "/api/tilings/v1/reset-password", reset_password, resetPassword,
    (),
    Params {},
    Query {},
    Data reset_password_post: models::ResetPasswordPost,

    "POST", "/api/tilings/v1/sign-in", sign_in, signIn,
    Account,
    Params {},
    Query {},
    Data sign_in_post: models::SignInPost,

    "POST", "/api/tilings/v1/sign-out", sign_out, signOut,
    (),
    Params {},
    Query {},
    Data,

    "POST", "/api/tilings/v1/sign-up", sign_up, signUp,
    Account,
    Params {},
    Query {},
    Data sign_in_post: models::SignUpPost,

    "PATCH", "/api/tilings/v1/tiling", update_tiling, updateTiling,
    models::FullTiling,
    Params {},
    Query {},
    Data atlas_post: models::FullTilingPatch,

    "POST", "/api/tilings/v1/update-polygon", update_polygon, updatePolygon,
    (),
    Params {},
    Query {},
    Data full_polygon_patch: models::FullPolygonPatch,

    "POST", "/api/tilings/v1/upsert-label", upsert_label, upsertLabel,
    (),
    Params {},
    Query {},
    Data label: String,

    "POST", "/api/tilings/v1/verify/{}", _verify, verify,
    bool,
    Params {
        verification_code: String,
    },
    Query {},
    Data,
}
