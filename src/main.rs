mod common;
mod interval;
mod patch;
mod simulation;
mod tile;
mod tiling;
mod tilings;

fn main() {
    let tiling = tilings::_3_12_12();
    let mut patch = patch::Patch::new(tiling);
    let (vertex_star_point, component_index) = match patch.add_path(patch::Path {
        vertex_star_point: common::Point(0.,0.),
        component_index: 0,
        edge_indices: vec![],
    }) { Ok(foo) => foo, Err(e) => panic!("{}", e) };
    let (vertex_star_point, component_index) = match patch.add_path(patch::Path {
        vertex_star_point,
        component_index,
        edge_indices: vec![0],
    }) { Ok(foo) => foo, Err(e) => panic!("{}", e) };
    let (vertex_star_point, component_index) = match patch.add_path(patch::Path {
        vertex_star_point,
        component_index,
        edge_indices: vec![3],
    }) { Ok(foo) => foo, Err(e) => panic!("{}", e) };

    println!("{}", patch);
}
