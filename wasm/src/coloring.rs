use colourado::{ColorPalette, PaletteType};
use itertools::*;
use plotters::style::RGBColor;
use std::collections::HashMap;
use tiling::*;

pub struct Coloring(HashMap<ProtoTile, RGBColor>);

impl Coloring {
    pub fn new(tiling: &Tiling) -> Coloring {
        Coloring(
            izip!(
                tiling.tiles.iter(),
                ColorPalette::new(tiling.tiles.len() as u32, PaletteType::Random, false).colors.iter()
            )
                .map(|(tile, color)| {
                    let rgb = color.to_array().iter().map(|e| (e * 256.) as u8).collect::<Vec<u8>>();
                    (tile.proto_tile.clone(), RGBColor(rgb[0], rgb[1], rgb[2]))
                }).collect::<HashMap<ProtoTile, RGBColor>>()
        )
    }
}
