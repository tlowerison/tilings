use tiling::{
    config::{Component, Config, Neighbor, Vertex},
    regular_polygon,
    Tiling,
};

pub fn _3_3_3_3_3_3() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);

    Tiling::new(
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

pub fn _4_4_4_4() -> Result<Tiling, String> {
    let square = regular_polygon(1., 4);

    Tiling::new(
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

pub fn _6_6_6() -> Result<Tiling, String> {
    let hexagon = regular_polygon(1., 6);

    Tiling::new(
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

pub fn _3_12_12() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);
    let dodecagon = regular_polygon(1., 12);

    Tiling::new(
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

pub fn _4_6_12() -> Result<Tiling, String> {
    let square = regular_polygon(1., 4);
    let hexagon = regular_polygon(1., 6);
    let dodecagon = regular_polygon(1., 12);

    Tiling::new(
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

pub fn _4_3_4_6() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);
    let square = regular_polygon(1., 4);
    let hexagon = regular_polygon(1., 6);

    Tiling::new(Config(vec![Vertex {
        components: vec![
            Component(square.clone(), 0),
            Component(triangle.clone(), 0),
            Component(square.clone(), 0),
            Component(hexagon.clone(), 0),
        ],
        neighbors: vec![
            Neighbor(0, 3, false),
            Neighbor(0, 2, false),
            Neighbor(0, 1, false),
            Neighbor(0, 0, false),
        ],
    }]))
}

pub fn _4_8_8() -> Result<Tiling, String> {
    let square = regular_polygon(1., 4);
    let octagon = regular_polygon(1., 8);

    Tiling::new(Config(vec![Vertex {
        components: vec![
            Component(octagon.clone(), 0),
            Component(octagon.clone(), 0),
            Component(square.clone(), 0),
        ],
        neighbors: vec![
            Neighbor(0, 2, false),
            Neighbor(0, 1, true),
            Neighbor(0, 0, false),
        ],
    }]))
}

pub fn _3_3_4_3_4() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);
    let square = regular_polygon(1., 4);

    Tiling::new(Config(vec![Vertex {
        components: vec![
            Component(triangle.clone(), 0),
            Component(triangle.clone(), 0),
            Component(square.clone(), 0),
            Component(triangle.clone(), 0),
            Component(square.clone(), 0),
        ],
        neighbors: vec![
            Neighbor(0, 4, false),
            Neighbor(0, 1, false),
            Neighbor(0, 3, false),
            Neighbor(0, 2, false),
            Neighbor(0, 0, false),
        ],
    }]))
}

pub fn _3_3_3_4_4() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);
    let square = regular_polygon(1., 4);

    Tiling::new(Config(vec![Vertex {
        components: vec![
            Component(square.clone(), 0),
            Component(square.clone(), 0),
            Component(triangle.clone(), 0),
            Component(triangle.clone(), 0),
            Component(triangle.clone(), 0),
        ],
        neighbors: vec![
            Neighbor(0, 2, false),
            Neighbor(0, 1, false),
            Neighbor(0, 0, false),
            Neighbor(0, 3, false),
            Neighbor(0, 4, false),
        ],
    }]))
}

pub fn _3_3_3_3_6() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);
    let hexagon = regular_polygon(1., 6);

    Tiling::new(Config(vec![Vertex {
        components: vec![
            Component(hexagon.clone(), 0),
            Component(triangle.clone(), 0),
            Component(triangle.clone(), 0),
            Component(triangle.clone(), 0),
            Component(triangle.clone(), 0),
        ],
        neighbors: vec![
            Neighbor(0, 1, false),
            Neighbor(0, 0, false),
            Neighbor(0, 3, false),
            Neighbor(0, 2, false),
            Neighbor(0, 4, false),
        ],
    }]))
}
