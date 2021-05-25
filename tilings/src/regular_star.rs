use std::f64::consts::PI;
use tiling::{
    config::{Component, Config, Neighbor, Vertex},
    regular_polygon,
    star_polygon,
    Tiling,
};

// 9.9.6∗4π/9
// https://www.polyomino.org.uk/publications/2004/star-polygon-tiling.pdf - Figure 4.e
pub fn _9_9_6a4pio9() -> Result<Tiling, String> {
    let nonagon = regular_polygon(1., 9);
    let star = star_polygon(1., 6, 4. * PI / 9.);

    Tiling::new(
        Config(vec![
            Vertex {
                components: vec![
                    Component(star.clone(), 0),
                    Component(nonagon.clone(), 0),
                    Component(nonagon.clone(), 0),
                ],
                neighbors: vec![
                    Neighbor(1, 0, false),
                    Neighbor(1, 1, false),
                    Neighbor(0, 2, false),
                ],
            },
            Vertex {
                components: vec![
                    Component(nonagon.clone(), 0),
                    Component(star.clone(), 1),
                ],
                neighbors: vec![
                    Neighbor(0, 0, false),
                    Neighbor(0, 1, false),
                ],
            },
        ]),
    )
}

// 4.6∗π/6.6∗∗π/2.6∗π/6
// https://www.polyomino.org.uk/publications/2004/star-polygon-tiling.pdf - Figure 3.f
pub fn _4_6apio6_6aapio2_6apio6() -> Result<Tiling, String> {
    let square = regular_polygon(1., 4);
    let star_1 = star_polygon(1., 6, PI / 6.);
    let star_2 = star_polygon(1., 6, PI / 2.);

    Tiling::new(
        Config(vec![
            Vertex {
                components: vec![
                    Component(star_2.clone(), 0),
                    Component(star_1.clone(), 1),
                ],
                neighbors: vec![
                    Neighbor(1, 0, false),
                    Neighbor(1, 3, false),
                ],
            },
            Vertex {
                components: vec![
                    Component(star_1.clone(), 0),
                    Component(square.clone(), 0),
                    Component(star_1.clone(), 0),
                    Component(star_2.clone(), 1),
                ],
                neighbors: vec![
                    Neighbor(0, 0, false),
                    Neighbor(2, 1, false),
                    Neighbor(2, 0, false),
                    Neighbor(0, 1, false),
                ],
            },
            Vertex {
                components: vec![
                    Component(square.clone(), 0),
                    Component(star_1.clone(), 1),
                ],
                neighbors: vec![
                    Neighbor(1, 2, false),
                    Neighbor(1, 1, false),
                ],
            },
        ]),
    )
}
