mod common;
mod tile;
mod tiling;

use svg::node::element::path::Data;
use std::io;

fn main() {
    let tiling = tiling::vertex_rules::tilings::_4444();
    println!("{}", tiling);
}

fn read_prototile<'a>() -> tile::ProtoTile {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("expected input: svg.path.d");
    tile::ProtoTile::new_from_svg(match Data::parse(&line) { Ok(data) => data, Err(e) => panic!("{}", e) })
}
