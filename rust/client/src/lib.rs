extern crate console_error_panic_hook;
#[macro_use] extern crate lazy_static;
extern crate serde_json;

mod coloring;
mod routes;

pub use self::routes::*;

use atlas::{Atlas, Patch, TileDiff};
use coloring::Coloring;
use geometry::*;
use lazy_static::lazy_static;
use models;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use pmr_quad_tree::Config as TreeConfig;
use std::{cell::RefCell, collections::HashMap, panic, sync::Mutex};
use tile::Tile;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

struct Global {
    coloring: Coloring,
    patch: RefCell<Patch<()>>,
    tiling_id: i32,
}

static mut GLOBAL: Option<Mutex<Global>> = None;

lazy_static! {
    static ref TILE_TREE_CONFIG: TreeConfig = TreeConfig {
        initial_radius: 1000.,
        max_depth: 50,
        splitting_threshold: 25,
    };
    static ref VERTEX_STAR_TREE_CONFIG: TreeConfig = TreeConfig {
        initial_radius: 1000.,
        max_depth: 70,
        splitting_threshold: 10,
    };
}

const CENTER: (f64, f64) = (0., 0.);
const SCALE: f64 = 30.;
const TO_CANVAS_AFFINE: Affine = Affine([[SCALE, 0.], [0., -SCALE]], [CENTER.0, CENTER.1]);
const FROM_CANVAS_AFFINE: Affine = Affine([[1./SCALE, 0.], [0., -1./SCALE]], [-CENTER.0 / SCALE, CENTER.1/SCALE]);

#[wasm_bindgen]
#[allow(non_snake_case)]
pub async fn setTiling(canvas: HtmlCanvasElement, tiling_id: i32) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let atlas = get_atlas_by_tiling_id(tiling_id).await?;

    unsafe {
        match &mut GLOBAL {
            None => {
                let (mut patch, coloring) = patch_from_atlas(atlas)?;
                draw(canvas, &coloring, patch.drain_tile_diffs())?;
                GLOBAL = Some(Mutex::new(Global {
                    coloring,
                    tiling_id,
                    patch: RefCell::new(patch),
                }));
            },
            Some(mutex) => {
                let mut global = mutex.lock().unwrap();
                if global.tiling_id != tiling_id {
                    let (patch, coloring) = patch_from_atlas(atlas)?;
                    global.coloring = coloring;
                    global.patch = RefCell::new(patch);
                    global.tiling_id = tiling_id;
                }
            },
        }
        Ok(())
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn resetTiling() {
    unsafe {
        match &mut GLOBAL {
            Some(_) => {
                GLOBAL = None;
            },
            _ => {},
        }
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn insertTileByPoint(canvas: HtmlCanvasElement, x: f64, y: f64) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    unsafe {
        match &mut GLOBAL {
            Some(global) => {
                let global = global.try_lock().or(Err(JsValue::FALSE))?;
                global.patch
                    .borrow_mut()
                    .insert_tile_by_point(from_canvas_point(x, y), Some(()))
                    .map_err(|e| JsValue::from_str(&String::from(format!("{}", e))))?;

                let tile_diffs = global.patch.borrow_mut().drain_tile_diffs();
                draw(canvas, &global.coloring, tile_diffs)?;
                Ok(())
            }
            None => Err(JsValue::FALSE)
        }
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn getNeighbors(x: f64, y: f64) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    unsafe {
        match &mut GLOBAL {
            Some(global) => {
                let global = match global.try_lock() { Ok(global) => global, _ => return JsValue::FALSE };
                let x = global.patch
                    .borrow()
                    .get_tile_neighbor_centroids(&from_canvas_point(x, y))
                    .map(|centroids| JsValue::from_str(&format!("{:?}", centroids)))
                    .unwrap_or_else(|| JsValue::FALSE); x
            }
            None => JsValue::FALSE
        }
    }
}

fn patch_from_atlas(db_atlas: models::FullAtlas) -> Result<(Patch<()>, Coloring), JsValue> {
    let patch = Patch::new(
        Atlas::new(&db_atlas).map_err(|e| JsValue::from_str(&e))?,
        (*TILE_TREE_CONFIG).clone(),
        (*VERTEX_STAR_TREE_CONFIG).clone(),
    ).map_err(|e| JsValue::from_str(&e))?;
    let coloring = Coloring::new(&patch.atlas);
    Ok((patch, coloring))
}

fn draw(canvas: HtmlCanvasElement, coloring: &Coloring, tile_diffs: HashMap<Tile, TileDiff>) -> Result<(), JsValue> {
    let mut backend = CanvasBackend::with_canvas_object(canvas).unwrap();
    for (tile, tile_diff) in tile_diffs.into_iter() {
        let mut points = tile.points.iter().map(|point| to_canvas_point(point)).collect::<Vec<(i32,i32)>>();
        match tile_diff {
            TileDiff::Added => {
                match backend.fill_polygon(points.clone(), coloring.0.get(&tile.size()).unwrap_or(&BLACK)) {
                    Ok(_) => {},
                    Err(e) => return Err(JsValue::from_str(&format!("{}", e))),
                }
                points.push(points.get(0).unwrap().clone());
                backend.draw_path(points, &BLACK)
            },
            TileDiff::Removed => {
                backend.fill_polygon(points, &WHITE)
            },
        }.or(Err(JsValue::FALSE))?;
    }
    Ok(())
}

fn from_canvas_point(x: f64, y: f64) -> Point {
    Point(x, y).transform(&FROM_CANVAS_AFFINE)
}

fn to_canvas_point(point: &Point) -> (i32, i32) {
    let transformed = point.transform(&TO_CANVAS_AFFINE);
    (transformed.0.round() as i32, transformed.1.round() as i32)
}
