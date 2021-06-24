extern crate console_error_panic_hook;
extern crate serde_json;

mod coloring;
mod routes;

pub use self::routes::*;

use atlas::{Atlas, Patch, TileDiff};
use coloring::Coloring;
use geometry::*;
use models;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::{cell::RefCell, collections::HashMap, panic, sync::Mutex};
use tile::Tile;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

struct Global {
    coloring: Coloring,
    cur_tile_centroid: Point,
    patch: RefCell<Patch>,
    tiling_id: i32,
}

static mut GLOBAL: Option<Mutex<Global>> = None;

const CENTER: (f64, f64) = (0., 0.);
const SCALE: f64 = 30.;
const TO_CANVAS_AFFINE: Affine = Affine([[SCALE, 0.], [0., -SCALE]], [CENTER.0, CENTER.1]);
const FROM_CANVAS_AFFINE: Affine = Affine([[1./SCALE, 0.], [0., -1./SCALE]], [-CENTER.0 / SCALE, CENTER.1/SCALE]);

#[wasm_bindgen]
#[allow(non_snake_case)]
pub async fn setTiling(canvas: HtmlCanvasElement, atlas_id: i32) -> Result<JsValue, JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let atlas = get_atlas(atlas_id).await?;

    unsafe {
        match &mut GLOBAL {
            None => {
                let (mut patch, coloring, cur_tile_centroid) = patch_from_atlas(atlas)?;
                draw(canvas, &coloring, patch.drain_tile_diffs())?;
                GLOBAL = Some(Mutex::new(Global {
                    coloring,
                    cur_tile_centroid,
                    tiling_id: atlas_id,
                    patch: RefCell::new(patch),
                }));
            },
            Some(mutex) => {
                let mut global = mutex.lock().unwrap();
                if global.tiling_id != atlas_id {
                    let (patch, coloring, cur_tile_centroid) = patch_from_atlas(atlas)?;
                    global.coloring = coloring;
                    global.cur_tile_centroid = cur_tile_centroid;
                    global.patch = RefCell::new(patch);
                    global.tiling_id = atlas_id;
                }
            },
        }
        Ok(JsValue::TRUE)
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn handleEvent(canvas: HtmlCanvasElement, x: f64, y: f64) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    unsafe {
        match &mut GLOBAL {
            Some(global) => {
                let mut global = match global.try_lock() { Ok(result) => result, Err(_) => return JsValue::FALSE };
                let centroid = match global.patch.borrow_mut().insert_adjacent_tile_by_point(&global.cur_tile_centroid, from_canvas_point(x, y)) {
                    Ok(centroid) => centroid,
                    Err(e) => return JsValue::from_str(&String::from(format!("{}", e))),
                };
                global.cur_tile_centroid = centroid;
                let tile_diffs = global.patch.borrow_mut().drain_tile_diffs();
                match draw(canvas, &global.coloring, tile_diffs) {
                    Ok(_) => JsValue::TRUE,
                    Err(js_value) => js_value,
                }
            }
            None => JsValue::FALSE
        }
    }
}

fn patch_from_atlas(db_atlas: models::FullAtlas) -> Result<(Patch, Coloring, Point), JsValue> {
    let atlas = Atlas::new(db_atlas).map_err(|e| JsValue::from_str(&e))?;
    let (patch, cur_tile_centroid) = Patch::new(atlas).map_err(|e| JsValue::from_str(&e))?;
    let coloring = Coloring::new(&patch.atlas);
    Ok((patch, coloring, cur_tile_centroid))
}

fn draw(canvas: HtmlCanvasElement, coloring: &Coloring, tile_diffs: HashMap<Tile, TileDiff>) -> Result<(), JsValue> {
    let mut backend = CanvasBackend::with_canvas_object(canvas).unwrap();
    for (tile, tile_diff) in tile_diffs.into_iter() {
        let mut points = tile.points.iter().map(|point| to_canvas_point(point)).collect::<Vec<(i32,i32)>>();
        let result = match tile_diff {
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
        };
        match result {
            Ok(_) => {},
            Err(_) => return Err(JsValue::FALSE),
        }
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
