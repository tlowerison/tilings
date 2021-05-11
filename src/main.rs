mod common;
mod interval;
mod patch;
mod simulation;
mod tile;
mod tiling;
mod tilings;

use common::Point;
use patch::*;

fn main() {
    let tiling = tilings::_4_4_4_4();
    let mut patch = Patch::new(tiling);
    match patch.add_path(Path {
        vertex_star_point: Point(0.,0.),
        component_index: 0,
        edge_indices: vec![0, 2],
    }) { Ok(_) => {}, Err(err) => println!("{}", err) }
    println!("{}", patch);
    // let allowed_states: Vec<(ProtoTile, u8)> = tiling.proto_tiles.iter().map(|proto_tile| (proto_tile.clone(), 2)).collect();
}
