use atlas::Atlas;
use colourado::{ColorPalette, PaletteType};
use itertools::*;
use plotters::style::RGBColor;
use std::collections::HashMap;

pub struct Coloring(pub(crate) HashMap<usize, RGBColor>);

impl Coloring {
    pub fn new(atlas: &Atlas) -> Coloring {
        Coloring(
            izip!(
                atlas.tiles.iter(),
                ColorPalette::new(atlas.tiles.len() as u32, PaletteType::Random, false).colors.iter(),
            )
                .map(|(tile, color)| {
                    let rgb = color.to_array().iter().map(|e| (e * 256.) as u8).collect::<Vec<u8>>();
                    (tile.size(), RGBColor(rgb[0], rgb[1], rgb[2]))
                }).collect::<HashMap<usize, RGBColor>>()
        )
    }
}
