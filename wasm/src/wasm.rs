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
    _3_3_3_3_3,
    _4_4_4_4,
    _6_6_6,
    _3_12_12,
}

impl TilingType {
    pub fn new_tiling(&self) -> tiling::Tiling {
        match self {
            TilingType::_3_3_3_3_3 => tiling::_3_3_3_3_3(),
            TilingType::_4_4_4_4 => tiling::_4_4_4_4(),
            TilingType::_6_6_6 => tiling::_6_6_6(),
            TilingType::_3_12_12 => tiling::_3_12_12(),
        }
    }
}

#[wasm_bindgen]
pub struct Config {
    tiling_type: TilingType,
}

static mut CONFIG: Config = Config {
    tiling_type: TilingType::_6_6_6,
};

struct State {
    component_index: usize,
    vertex_star_point: Point,
    heap_state: Option<HeapState>,
}

struct HeapState {
    coloring: Coloring,
    patch: Patch,
}

static mut STATE: State = State {
    component_index: 0,
    vertex_star_point: Point(0.,0.),
    heap_state: None,
};

const CENTER: (i32, i32) = (300, 200);
const SCALE: f64 = 30.;

#[wasm_bindgen]
pub fn init (canvas: HtmlCanvasElement) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    unsafe {
        set_tiling(canvas.clone(), CONFIG.tiling_type);
        match &STATE.heap_state {
            None => JsValue::FALSE,
            Some(heap_state) => {
                // JsValue::from_str(&format!("{}", patch))
                let mut tile_diffs: HashMap<Tile, TileDiff> = HashMap::default();
                let vertex_star = heap_state.patch.vertex_stars.get(&STATE.vertex_star_point).unwrap();
                let proto_tile = vertex_star.get_proto_tile(&heap_state.patch.tiling, STATE.component_index).unwrap();
                tile_diffs.insert(Tile::new(proto_tile), TileDiff::Added);
                match draw(canvas, tile_diffs) {
                    None => JsValue::TRUE,
                    Some(js_value) => js_value,
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_tiling(canvas: HtmlCanvasElement, tiling_type: TilingType) {
    unsafe {

        let initialized = match STATE.heap_state { Some(_) => true, None => false };
        if !initialized || CONFIG.tiling_type != tiling_type {
            let tiling = tiling_type.new_tiling();
            CONFIG.tiling_type = tiling_type;
            match &mut STATE.heap_state {
                None => {},
                Some(heap_state) => { draw(canvas, heap_state.patch.drain_tiles()); },
            };

            let mut patch = Patch::new(tiling);

            patch.add_path(tiling::Path {
                vertex_star_point: STATE.vertex_star_point,
                component_index: STATE.component_index,
                edge_indices: vec![],
            });
            STATE.heap_state = Some(HeapState {
                coloring: Coloring::new(&patch.tiling),
                patch,
            });
        }
    }
}

#[wasm_bindgen]
pub fn step(canvas: HtmlCanvasElement, edge_index: usize) -> JsValue {
    unsafe {
        match &mut STATE.heap_state {
            Some(heap_state) => {
                match heap_state.patch.add_path(tiling::Path {
                    vertex_star_point: STATE.vertex_star_point,
                    component_index: STATE.component_index,
                    edge_indices: vec![edge_index],
                }) {
                    Ok((vertex_star_point, component_index)) => {
                        STATE.vertex_star_point = vertex_star_point;
                        STATE.component_index = component_index;
                        match draw(canvas, heap_state.patch.drain_tile_diffs()) {
                            Some(js_value) => js_value,
                            None => {
                                let vertex_star = heap_state.patch.vertex_stars.get(&vertex_star_point).unwrap();
                                let link = match vertex_star.get_component_edges(&heap_state.patch.tiling, component_index) { None => return JsValue::TRUE, Some(link) => link };
                                JsValue::from_str(&format!("{{\"vertex_star_point\":{},\"edges\":[{}]}}", vertex_star_point, link.iter().map(|(p0,p1)| format!("[{},{}]", p0, p1)).collect::<Vec<String>>().join(",")))
                            },
                        }
                    },
                    Err(_) => JsValue::FALSE,
                }
            }
            None => JsValue::FALSE
        }
    }
}

fn draw(canvas: HtmlCanvasElement, tile_diffs: HashMap<Tile, TileDiff>) -> Option<JsValue> {
    unsafe {
        match &STATE.heap_state {
            None => Some(JsValue::FALSE),
            Some(heap_state) => {
                let mut backend = CanvasBackend::with_canvas_object(canvas).unwrap();
                for (tile, tile_diff) in tile_diffs.into_iter() {
                    let mut points = tile.proto_tile.points.iter().map(|point| (CENTER.0 + (SCALE * point.0).round() as i32, CENTER.1 - (SCALE * point.1).round() as i32)).collect::<Vec<(i32,i32)>>();
                    let result = match tile_diff {
                        TileDiff::Added => {
                            backend.fill_polygon(points.clone(), heap_state.coloring.0.get(&tile.proto_tile).unwrap_or(&BLACK));
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
