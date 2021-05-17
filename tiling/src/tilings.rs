use crate::tile::regular_polygon;
use crate::tiling::{
    config::{Component, Config, Neighbor, Vertex},
    Tiling,
};

pub fn _3_3_3_3_3_3() -> Tiling {
    let triangle = regular_polygon(1., 3);

    Tiling::new(
        String::from("3.3.3.3.3.3"),
        Config(vec![Vertex {
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
        }]),
    )
}

pub fn _4_4_4_4() -> Tiling {
    let square = regular_polygon(1., 4);

    Tiling::new(
        String::from("4.4.4.4"),
        Config(vec![Vertex {
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
        }]),
    )
}

pub fn _6_6_6() -> Tiling {
    let hexagon = regular_polygon(1., 6);

    Tiling::new(
        String::from("6.6.6"),
        Config(vec![Vertex {
            components: vec![
                Component(hexagon.clone(), 0),
                Component(hexagon.clone(), 0),
                Component(hexagon.clone(), 0),
            ],
            neighbors: vec![
                Neighbor(0, 1, false),
                Neighbor(0, 2, false),
                Neighbor(0, 0, false),
            ],
        }]),
    )
}

pub fn _3_12_12() -> Tiling {
    let triangle = regular_polygon(1., 3);
    let dodecagon = regular_polygon(1., 12);

    Tiling::new(
        String::from("3.12.12"),
        Config(vec![Vertex {
            components: vec![
                Component(triangle.clone(), 0),
                Component(dodecagon.clone(), 0),
                Component(dodecagon.clone(), 0),
            ],
            neighbors: vec![
                Neighbor(0, 1, false),
                Neighbor(0, 0, false),
                Neighbor(0, 2, false),
            ],
        }]),
    )
}

pub fn _4_6_12() -> Tiling {
    let square = regular_polygon(1., 4);
    let hexagon = regular_polygon(1., 6);
    let dodecagon = regular_polygon(1., 12);

    Tiling::new(
        String::from("4.6.12"),
        Config(vec![Vertex {
            components: vec![
                Component(dodecagon.clone(), 0),
                Component(hexagon.clone(), 0),
                Component(square.clone(), 0),
            ],
            neighbors: vec![
                Neighbor(0, 0, true),
                Neighbor(0, 1, true),
                Neighbor(0, 2, true),
            ],
        }]),
    )
}
