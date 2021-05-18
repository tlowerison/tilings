extern crate console_error_panic_hook;

use crate::coloring::Coloring;
use geometry::*;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::{collections::HashMap, panic};
use tiling::{self, Patch, Tile, TileDiff};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Copy,Clone,PartialEq)]
pub enum TilingType {
    _3_3_3_3_3_3,
    _4_4_4_4,
    _6_6_6,
    _3_12_12,
    _4_6_12,
}

impl TilingType {
    pub fn new_tiling(&self) -> tiling::Tiling {
        match self {
            TilingType::_3_3_3_3_3_3 => tiling::_3_3_3_3_3_3(),
            TilingType::_4_4_4_4 => tiling::_4_4_4_4(),
            TilingType::_6_6_6 => tiling::_6_6_6(),
            TilingType::_3_12_12 => tiling::_3_12_12(),
            TilingType::_4_6_12 => tiling::_4_6_12(),
        }
    }
}

#[wasm_bindgen]
pub struct Config {
    tiling_type: TilingType,
}

static mut CONFIG: Config = Config {
    tiling_type: TilingType::_4_4_4_4,
};

struct State {
    coloring: Coloring,
    cur_tile_centroid: Point,
    patch: Patch,
}

static mut STATE: Option<State> = None;

const CENTER: (f64, f64) = (300., 200.);
const SCALE: f64 = 30.;
const TO_CANVAS_AFFINE: Affine = Affine([[SCALE, 0.], [0., -SCALE]], [CENTER.0, CENTER.1]);
const FROM_CANVAS_AFFINE: Affine = Affine([[1./SCALE, 0.], [0., -1./SCALE]], [-CENTER.0 / SCALE, CENTER.1/SCALE]);

fn from_canvas(x: f64, y: f64) -> Point {
    Point(x, y).transform(&FROM_CANVAS_AFFINE)
}

fn to_canvas(point: &Point) -> (i32, i32) {
    let transformed = point.transform(&TO_CANVAS_AFFINE);
    (transformed.0.round() as i32, transformed.1.round() as i32)
}

#[wasm_bindgen]
pub fn init (canvas: HtmlCanvasElement) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    unsafe {
        let js_value = set_tiling(canvas.clone(), CONFIG.tiling_type);
        if js_value != JsValue::TRUE {
            return js_value
        }
        match &mut STATE {
            None => JsValue::FALSE,
            Some(state) => {
                match draw(canvas, state.patch.drain_tile_diffs()) {
                    None => JsValue::TRUE,
                    Some(js_value) => js_value,
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_tiling(canvas: HtmlCanvasElement, tiling_type: TilingType) -> JsValue {
    unsafe {
        let initialized = match STATE { Some(_) => true, None => false };
        if !initialized || CONFIG.tiling_type != tiling_type {
            let tiling = tiling_type.new_tiling();
            CONFIG.tiling_type = tiling_type;
            match &mut STATE {
                None => {},
                Some(state) => {
                    draw(canvas, state.patch.drain_tiles());
                },
            };

            let (patch, cur_tile_centroid) = match Patch::new(tiling) { Ok(pair) => pair, Err(s) => return JsValue::from_str(&s) };
            STATE = Some(State {
                coloring: Coloring::new(&patch.tiling),
                cur_tile_centroid,
                patch,
            });
        }
    }
    JsValue::TRUE
}

#[wasm_bindgen]
pub fn click(canvas: HtmlCanvasElement, x: f64, y: f64) ->  JsValue {
    unsafe {
        match &mut STATE {
            Some(state) => {
                let point = from_canvas(x, y);
                match state.patch.insert_adjacent_tile_by_point(&state.cur_tile_centroid, point) {
                    Ok(centroid) => {
                        state.cur_tile_centroid = centroid;
                        match draw(canvas, state.patch.drain_tile_diffs()) {
                            Some(js_value) => js_value,
                            None => JsValue::TRUE,
                        }
                    },
                    Err(e) => JsValue::from_str(&String::from(format!("{}", e))),
                }
            }
            None => JsValue::FALSE
        }
    }
}

fn draw(canvas: HtmlCanvasElement, tile_diffs: HashMap<Tile, TileDiff>) -> Option<JsValue> {
    unsafe {
        match &STATE {
            None => Some(JsValue::FALSE),
            Some(state) => {
                let mut backend = CanvasBackend::with_canvas_object(canvas).unwrap();
                for (tile, tile_diff) in tile_diffs.into_iter() {
                    let mut points = tile.points.iter().map(|point| to_canvas(point)).collect::<Vec<(i32,i32)>>();
                    let result = match tile_diff {
                        TileDiff::Added => {
                            match backend.fill_polygon(points.clone(), state.coloring.0.get(&tile.size()).unwrap_or(&BLACK)) {
                                Ok(_) => {},
                                Err(e) => return Some(JsValue::from_str(&format!("{}", e))),
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
                        Err(_) => return Some(JsValue::FALSE),
                    }
                }
                None
            },
        }
    }
}
