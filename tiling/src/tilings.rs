use crate::tile::*;
use crate::tiling::{Tiling, config::*};

use common::*;
use geometry::*;
use std::{
    f64::consts::{PI, TAU},
    iter,
};

pub fn _3_3_3_3_3() -> Tiling {
    let triangle = regular_polygon(1., 3);
    Tiling::new(String::from("3.3.3.3.3.3"), Config(vec![
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

pub fn _4_4_4_4() -> Tiling {
    let square = regular_polygon(1., 4);
    Tiling::new(String::from("4.4.4.4"), Config(vec![
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

pub fn _6_6_6() -> Tiling {
    let hexagon = regular_polygon(1., 6);
    Tiling::new(String::from("6.6.6"), Config(vec![
        Vertex {
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
        },
    ]))
}

pub fn _3_12_12() -> Tiling {
    let triangle = regular_polygon(1., 3);
    let dodecagon = regular_polygon(1., 12);
    Tiling::new(String::from("3.12.12"), Config(vec![
        Vertex {
            components: vec![
                Component(triangle.clone(), 0),
                Component(dodecagon.clone(), 0),
                Component(dodecagon.clone(), 0),
            ],
            neighbors: vec![
                Neighbor(0, 1, true),
                Neighbor(0, 0, true),
                Neighbor(0, 2, true),
            ],
        },
    ]))
}

pub fn regular_polygon(side_length: f64, num_sides: usize) -> ProtoTile {
    let n = num_sides as f64;
    let centroid_angle_of_inclination = PI * (0.5 - 1./n);
    let radius = side_length / 2. / centroid_angle_of_inclination.cos();
    let centroid = Point(radius * centroid_angle_of_inclination.cos(), radius * centroid_angle_of_inclination.sin());

    let affine = reduce_transforms(&vec![
        Euclid::Translate((-centroid).values()),
        Euclid::Rotate(TAU / n),
        Euclid::Translate(centroid.values()),
    ]);

    let mut generator = Generator::new(affine);

    let proto_tile = ProtoTile {
        points: iter::repeat(Point(0.,0.)).take(num_sides).enumerate().map(|(i, point)| point.transform(&generator(i))).collect(),
        flipped: false,
    };

    proto_tile.assert_angles(iter::repeat(centroid_angle_of_inclination).take(num_sides).collect());
    proto_tile.assert_sides(iter::repeat(side_length).take(num_sides).collect());

    proto_tile
}
