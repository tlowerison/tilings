use crate::tile::*;
use crate::tiling::vertex_rules::tiling::*;

pub fn _33333() -> Tiling {
    let triangle = ProtoTile::new(vec![(0.,0.), (1.,0.), (1.,(0.75 as f64).sqrt())]);
    triangle.assert_angles(vec![60.,60.,60.]);
    triangle.assert_sides(vec![1.,1.,1.]);

    Tiling::new(String::from("333333"), Config(vec![
        Vertex {
            components: vec![
                Component(triangle.clone(), 0),
                Component(triangle.clone(), 0),
                Component(triangle.clone(), 0),
                Component(triangle.clone(), 0),
                Component(triangle.clone(), 0),
                Component(triangle.clone(), 0),
            ],
            neighbors: vec![
                Neighbor(0, 3, false),
                Neighbor(0, 4, false),
                Neighbor(0, 5, false),
                Neighbor(0, 0, false),
                Neighbor(0, 1, false),
                Neighbor(0, 2, false),
            ],
        },
    ]))
}

pub fn _4444() -> Tiling {
    let square = ProtoTile::new(vec![(0.,0.), (1.,0.), (1.,1.), (0.,1.)]);
    square.assert_angles(vec![90.,90.,90.,90.]);
    square.assert_sides(vec![1.,1.,1.,1.]);

    Tiling::new(String::from("4444"), Config(vec![
        Vertex {
            components: vec![
                Component(square.clone(), 0),
                Component(square.clone(), 1),
                Component(square.clone(), 2),
                Component(square.clone(), 3),
            ],
            neighbors: vec![
                Neighbor(0, 2, false),
                Neighbor(0, 3, false),
                Neighbor(0, 0, false),
                Neighbor(0, 1, false),
            ],
        },
    ]))
}
