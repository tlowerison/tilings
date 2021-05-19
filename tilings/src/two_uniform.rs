use tiling::{
    config::{Component, Config, Neighbor, Vertex},
    regular_polygon,
    Tiling,
};

pub fn _3_3_3_3_3_3__3_3_4_3_4() -> Result<Tiling, String> {
    let triangle = regular_polygon(1., 3);
    let square = regular_polygon(1., 4);

    Tiling::new(Config(vec![
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
                Neighbor(1, 4, false),
                Neighbor(1, 4, false),
                Neighbor(1, 4, false),
                Neighbor(1, 4, false),
                Neighbor(1, 4, false),
                Neighbor(1, 4, false),
            ],
        },
        Vertex {
            components: vec![
                Component(square.clone(), 0),
                Component(triangle.clone(), 0),
                Component(square.clone(), 0),
                Component(triangle.clone(), 0),
                Component(triangle.clone(), 0),
            ],
            neighbors: vec![
                Neighbor(1, 3, false),
                Neighbor(1, 2, false),
                Neighbor(1, 1, false),
                Neighbor(1, 0, false),
                Neighbor(0, 4, false),
            ],
        },
    ]))
}
