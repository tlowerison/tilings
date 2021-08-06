extern crate console_error_panic_hook;
#[macro_use] extern crate lazy_static;
extern crate serde_json;

mod coloring;
mod routes;

pub use self::routes::*;

use atlas::Atlas;
use canvas::*;
use coloring::Coloring;
use patch::PatchTile;
use plotters::prelude::*;
use pmr_quad_tree::Config as TreeConfig;
use std::{collections::HashMap, panic};
use tiling::{Config as TilingConfig, Tiling};
use wasm_bindgen::prelude::*;

type State = ();

struct Global {
    coloring: Coloring,
    tiling: Tiling<State>,
}

static CANVAS_RADIUS: f64 = 4.;

static TILE_TREE_CONFIG: TreeConfig = TreeConfig {
    initial_radius: 1000.,
    max_depth: 50,
    splitting_threshold: 25,
};

static VERTEX_STAR_TREE_CONFIG: TreeConfig = TreeConfig {
    initial_radius: 1000.,
    max_depth: 70,
    splitting_threshold: 10,
};

static mut GLOBALS: Option<HashMap<i32, Global>> = None;

#[wasm_bindgen]
pub fn init() {
    unsafe {
        if let None = GLOBALS.as_ref() {
            GLOBALS = Some(HashMap::default());
        }
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub async fn setTiling(global_id: i32, tiling_id: i32) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let db_atlas = get_atlas_by_tiling_id(tiling_id).await?;
    let atlas = Atlas::new(&db_atlas).map_err(|e| JsValue::from_str(&e))?;
    let coloring = Coloring::new(&atlas);

    let tiling = Tiling::new(TilingConfig {
        atlas,
        canvas_radius: CANVAS_RADIUS,
        id: global_id,
        tile_tree_config: TILE_TREE_CONFIG.clone(),
        vertex_star_tree_config: VERTEX_STAR_TREE_CONFIG.clone(),
    }).map_err(|e| JsValue::from_str(&e))?;

    let new_global = Global { coloring, tiling };

    let globals = unsafe { GLOBALS.as_mut().unwrap() };
    if let None = globals.get(&global_id) {
        globals.insert(global_id, new_global);
    } else {
        let global = globals.get_mut(&global_id).unwrap();
        *global = new_global;
    }
    Ok(())
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn removeTiling(global_id: i32) {
    unsafe {
        GLOBALS.as_mut().unwrap().remove(&global_id);
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn insertTileByPoint(global_id: i32, x: f64, y: f64) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    unsafe {
        let global = GLOBALS
            .as_mut()
            .unwrap()
            .get_mut(&global_id)
            .ok_or_else(|| JsValue::from_str(&format!("no global found with id {}", global_id)))?;

        let coloring = &global.coloring;

        global
            .tiling
            .insert_tile_by_point(
                from_canvas_point(x, y),
                Some(()),
                |patch_tile: &PatchTile<State>| coloring.0.get(&patch_tile.size()).unwrap_or(&BLACK),
            )
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
    }

    Ok(())
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn getNeighbors(global_id: i32, x: f64, y: f64) -> Result<JsValue, JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    unsafe {
        GLOBALS
            .as_ref()
            .unwrap()
            .get(&global_id)
            .ok_or_else(|| JsValue::from_str(&format!("no canvas found with id {}", global_id)))?
            .tiling
            .patch
            .get_tile_neighbor_centroids(&from_canvas_point(x, y))
            .map(|centroids| JsValue::from_str(&format!("{:?}", centroids)))
            .ok_or_else(|| JsValue::FALSE)
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
