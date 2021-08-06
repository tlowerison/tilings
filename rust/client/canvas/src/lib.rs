use geometry::*;
use plotters::{
    prelude::*,
    style::RGBColor,
};
use plotters_canvas::CanvasBackend;
use tile::Tile;
use web_sys::HtmlCanvasElement;

pub const CENTER: (f64, f64) = (0., 0.);
pub const SCALE: f64 = 30.;
pub const TO_CANVAS_AFFINE: Affine = Affine([[SCALE, 0.], [0., -SCALE]], [CENTER.0, CENTER.1]);
pub const FROM_CANVAS_AFFINE: Affine = Affine([[1./SCALE, 0.], [0., -1./SCALE]], [-CENTER.0 / SCALE, CENTER.1/SCALE]);

pub struct Canvas {
    pub backend: CanvasBackend,
    pub bounds: Bounds,
}

impl Canvas {
    // Asserts canvas width and height are the same.
    pub fn new(canvas: HtmlCanvasElement, center: Point) -> Canvas {
        let radius = canvas.width();
        assert_eq!(radius, canvas.height());

        Canvas {
            backend: CanvasBackend::with_canvas_object(canvas).unwrap(),
            bounds: Bounds {
                center,
                radius: radius as f64,
            },
        }
    }

    pub fn fill_tile(&mut self, tile: &Tile, color: &RGBColor) -> Result<(), String> {
        let mut canvas_points: Vec<(i32, i32)> = tile.points
            .iter()
            .map(|point| to_canvas_point(point))
            .collect();

        self.backend.fill_polygon(canvas_points.clone(), color).or_else(|_| Err(String::from("could not fill polygon in canvas")))?;

        canvas_points.push(canvas_points.get(0).unwrap().clone());
        self.backend.draw_path(canvas_points, &BLACK).or_else(|_| Err(String::from("could not draw path in canvas")))?;

        Ok(())
    }
}

impl Spatial for Canvas {
    type Hashed = Point;
    fn distance(&self, point: &Point) -> f64 { self.bounds.distance(point) }
    fn intersects(&self, bounds: &Bounds) -> bool { self.bounds.intersects(bounds) }
    fn key(&self) -> Self::Hashed { self.bounds.key() }
}

pub fn from_canvas_point(x: f64, y: f64) -> Point {
    Point(x, y).transform(&FROM_CANVAS_AFFINE)
}

pub fn to_canvas_point(point: &Point) -> (i32, i32) {
    let transformed = point.transform(&TO_CANVAS_AFFINE);
    (transformed.0.round() as i32, transformed.1.round() as i32)
}
