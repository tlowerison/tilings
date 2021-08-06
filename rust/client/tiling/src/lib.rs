use atlas::Atlas;
use canvas::{Canvas, SCALE, TO_CANVAS_AFFINE};
use geometry::*;
use patch::{Patch, PatchTile, TileDiff};
use plotters::style::RGBColor;
use pmr_quad_tree::Config as TreeConfig;
use std::collections::HashMap;
use tile::Tile;
use web_sys::HtmlCanvasElement;
use wasm_bindgen::{JsCast, prelude::*};

pub struct Tiling<State> {
    pub id: i32,
    pub canvases: HashMap<Point, Canvas>,
    pub canvas_radius: f64,
    pub patch: Patch<State>,
    neighbor_centroid_diffs: Vec<Point>,
}

pub struct Config {
    pub id: i32,
    pub atlas: Atlas,
    pub canvas_radius: f64,
    pub tile_tree_config: TreeConfig,
    pub vertex_star_tree_config: TreeConfig,
}

impl<State> Tiling<State> {
    pub fn new(config: Config) -> Result<Tiling<State>, String> {
        let canvas_diameter = 2. * config.canvas_radius;
        let neighbor_centroid_diffs = vec![
            Point(canvas_diameter, 0.),
            Point(canvas_diameter, canvas_diameter),
            Point(0., canvas_diameter),
            Point(-canvas_diameter, canvas_diameter),
            Point(-canvas_diameter, 0.),
            Point(-canvas_diameter, -canvas_diameter),
            Point(0., -canvas_diameter),
            Point(canvas_diameter, -canvas_diameter),
        ];

        Ok(Tiling {
            neighbor_centroid_diffs,
            id: config.id,
            canvas_radius: config.canvas_radius,
            canvases: HashMap::default(),
            patch: Patch::new(config.atlas, config.tile_tree_config, config.vertex_star_tree_config)?,
        })
    }

    pub fn insert_tile_by_point(&mut self, point: Point, state: Option<State>, get_color: impl Fn(&PatchTile<State>) -> &RGBColor) -> Result<(), String> {
        let centroid = Point(self.center(point.0), self.center(point.1));

        let neighbor_centroids = self.neighbor_centroids(&centroid);

        self.patch.insert_tile_by_point(point, state)?;
        let tile_diffs = self.patch.drain_tile_diffs();

        if tile_diffs.len() > 0 {
            let all_bounds = std::iter::once(centroid)
                .chain(neighbor_centroids.into_iter())
                .map(|neighbor_centroid| Bounds { center: neighbor_centroid, radius: self.canvas_radius })
                .collect::<Vec<Bounds>>();

            for (patch_tile_rc_item, tile_diff) in tile_diffs.into_iter() {
                let patch_tile = &*patch_tile_rc_item.value();
                let color = get_color(patch_tile);

                match tile_diff {
                    TileDiff::Added => {
                        for bounds in all_bounds.iter() {
                            for edge in patch_tile.edges().into_iter() {
                                if edge.intersects(bounds) {
                                    self.fill_tile(&bounds.center, &patch_tile.tile, &color)?;
                                }
                            }
                        }
                    },
                    _ => {},
                }
            }
        }

        Ok(())
    }

    fn center(&self, distance: f64) -> f64 {
        (2. * (distance / (2. * self.canvas_radius)).ceil() - 1.) * self.canvas_radius
    }

    fn fill_tile(&mut self, canvas_centroid: &Point, tile: &Tile, color: &RGBColor) -> Result<(), String> {
        let existing_canvas = self.canvases.get_mut(canvas_centroid);
        if let Some(canvas) = existing_canvas {
            return canvas.fill_tile(tile, color);
        }

        let html_canvas_element = match get_new_canvas(&self.id, &self.canvas_radius, canvas_centroid) {
            Some(html_canvas_element) => html_canvas_element,
            None => return Err(String::from("could not get new HtmlCanvasElement")),
        };

        self.canvases
            .entry(canvas_centroid.clone())
            .or_insert(Canvas::new(html_canvas_element, canvas_centroid.clone()))
            .fill_tile(tile, color)
    }

    fn neighbor_centroids(&self, centroid: &Point) -> Vec<Point> {
        self.neighbor_centroid_diffs
            .iter()
            .map(|neighbor_centroid_diff| centroid + neighbor_centroid_diff)
            .collect()
    }
}

#[wasm_bindgen(module = "components/canvas/utils.js")]
extern "C" {
    fn getNewCanvas(global_id: i32, canvas_radius: f64, center_x: f64, center_y: f64) -> JsValue;
}

fn get_new_canvas(global_id: &i32, canvas_radius: &f64, center: &Point) -> Option<HtmlCanvasElement> {
    let point = center.transform(&TO_CANVAS_AFFINE);
    getNewCanvas(*global_id, SCALE * canvas_radius, point.0, point.1)
        .dyn_into::<HtmlCanvasElement>()
        .ok()
}
