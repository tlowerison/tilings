use std::f64::consts::PI;
use tiling::{
    config::{Component, Config, Neighbor, Vertex},
    regular_polygon,
    star_polygon,
    Tiling,
};

// 4.6∗π/6.6∗∗π/2.6∗π/6
// https://www.polyomino.org.uk/publications/2004/star-polygon-tiling.pdf - Figure 3.f
pub fn _4_6apio6_6aapio2_6apio6() -> Result<Tiling, String> {
    let square = regular_polygon(1., 4);
    let star_1 = star_polygon(1., 6, PI / 6., true);
    let star_2 = star_polygon(1., 6, PI / 2., true);

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
            }
        ]),
    )
}
