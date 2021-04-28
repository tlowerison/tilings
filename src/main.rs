use std::{
    collections::HashMap,
    env,
    io,
    str::FromStr,
};

// Coord is a coordinate in the R2-plane.
// It can be used as an exact drawing location
// or as an input to some transform.
struct Coord(f64, f64);

enum Transform {
    Rotation(f64),
    Reflection(f64),
    None,
}

// Node in the tiling graph, has sub-labels shape and transform.
struct Node<'a> {
    name: &'a str,
    shape: usize,
    transform: Transform,
}

// Adjacencies is a node and its neighbors.
// The tuple (usize, usize) represents (neighbor node index, neighbor node's matching edge index).
struct Adjacencies<'a>(Node<'a>, &'a[(usize, usize)]);

// Tiling is an adjacencies list of nodes.
struct Tiling<'a> {
    name: &'a str,
    // shapes is a list of shape degrees (i.e. for each shape, the number of edges it has)
    // - By specifying only the shape degree instead of the actual shape, we decouple the
    //   tiling from the actual shapes used in its implementation, as long as they match
    //   the specified edge requirements.
    // - It's ok and expected to have duplicate values in shapes, for example, if a tiling
    //   expects to use a square and a rhombus in topologically different scenarios, they
    //   should each be specified separately in shapes as 4.
    shapes: &'a[usize],
    graph: &'a[Adjacencies<'a>],
}

// Path is a traversal through a graph.
// Its first component represents the starting node,
// the second component represents each step of the path,
// where each member is that step's edge taken as a node's neighbor.
struct Path<'a>(usize, &'a[usize]);

impl<'a> Tiling<'a> {
    fn get(&self, n: usize) -> Result<&Adjacencies, String> {
        match self.graph.get(n) {
            Some(a) => Ok(a),
            None => Err(format!("missing node: {}", n)),
        }
    }

    fn traverse(&self, path: &Path) -> Result<usize, String> {
        let mut n = path.0;
        let mut adjacencies = match self.get(path.0) {
            Ok(a) => a,
            Err(_) => {
                return Err(format!("invalid starting node in path: {}", path.0));
            },
        };
        for edge in path.1.iter() {
            if *edge >= adjacencies.1.len() {
                return Err(format!("invalid edge {} from node {}", edge, n))
            }
            adjacencies = match adjacencies.1.get(*edge) {
                Some(neighbor) => {
                    println!("{}", adjacencies.0.name);
                    n = neighbor.0;
                    match self.get(n) {
                        Ok(adj) => adj,
                        Err(e) => { return Err(e); },
                    }
                },
                None => { return Err(format!("invalid starting node in path: {}", path.0)); },
            };
        }
        return Ok(n);
    }
}

const TILINGS: &'static [Tiling] = &[
    Tiling {
        name: "333",
        shapes: &[3],
        graph: &[
            Adjacencies(
                Node {
                    name: "upward triangle",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(1,1),(1,2),(1,0)],
            ),
            Adjacencies(
                Node {
                    name: "downward triangle",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(0,2),(0,0),(0,1)],
            ),
        ],
    },
    Tiling {
        name: "4444",
        shapes: &[4],
        graph: &[
            Adjacencies(
                Node {
                    name: "square",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(0,2),(0,3),(0,0),(0,1)],
            ),
        ],
    },
    Tiling {
        name: "666",
        shapes: &[6],
        graph: &[
            Adjacencies(
                Node {
                    name: "hexagon",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(0,3),(0,4),(0,5),(0,0),(0,1),(0,2)],
            ),
        ],
    },
    Tiling {
        name: "488",
        shapes: &[4, 8],
        graph: &[
            Adjacencies(
                Node {
                    name: "square",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(1,4),(1,6),(1,0),(1,2)],
            ),
            Adjacencies(
                Node {
                    name: "octagon",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(0,2),(1,5),(0,3),(1,7),(0,0),(1,1),(0,1),(1,3)],
            ),
        ],
    },
    Tiling {
        // https://www.mi.sanu.ac.rs/vismath/crowe/cr3.htm - Figure 1
        name: "4612",
        shapes: &[4,6,12],
        graph: &[
            Adjacencies(
                Node {
                    name: "bottom square",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(5,6),(3,4),(5,0),(4,4)],
            ),
            Adjacencies(
                Node {
                    name: "top-right square",
                    shape: 0,
                    transform: Transform::Rotation(120 as f64),
                },
                &[(5,10),(3,0),(5,4),(4,0)],
            ),
            Adjacencies(
                Node {
                    name: "top-left square",
                    shape: 0,
                    transform: Transform::Rotation(240 as f64),
                },
                &[(5,2),(3,2),(5,8),(4,2)],
            ),
            Adjacencies(
                Node {
                    name: "southward hexagon",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(1,1),(5,9),(2,1),(5,1),(0,1),(5,5)],
            ),
            Adjacencies(
                Node {
                    name: "northward hexagon",
                    shape: 1,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(1,3),(5,3),(2,3),(5,7),(0,3),(5,11)],
            ),
            Adjacencies(
                Node {
                    name: "dodecagon",
                    shape: 2,
                    transform: Transform::None,
                },
                &[(0,2),(3,3),(2,0),(4,1),(1,2),(3,5),(0,0),(4,3),(2,2),(3,1),(1,0),(4,5)],
            ),
        ],
    },
    Tiling {
        name: "31212",
        shapes: &[3,12],
        graph: &[
            Adjacencies(
                Node {
                    name: "upward triangle",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(2,7),(2,11),(2,3)],
            ),
            Adjacencies(
                Node {
                    name: "downward triangle",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(2,9),(2,1),(2,5)],
            ),
            Adjacencies(
                Node {
                    name: "dodecagon",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(2,6),(1,1),(2,8),(0,2),(2,10),(1,2),(2,0),(0,0),(2,2),(1,0),(2,4),(0,1)],
            ),
        ],
    },
    Tiling {
        name: "3636",
        shapes: &[3,6],
        graph: &[
            Adjacencies(
                Node {
                    name: "upward triangle",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(2,3),(2,5),(2,1)],
            ),
            Adjacencies(
                Node {
                    name: "downward triangle",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(2,4),(2,0),(2,2)],
            ),
            Adjacencies(
                Node {
                    name: "hexagon",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(1,1),(0,2),(1,2),(0,0),(1,0),(0,1)],
            ),
        ],
    },
    Tiling {
        // https://www.mi.sanu.ac.rs/vismath/crowe/cr3.htm - Figure 5
        name: "33434",
        shapes: &[3,4],
        graph: &[
            Adjacencies(
                Node {
                    name: "upward triangle",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(5,2),(2,1),(4,1)],
            ),
            Adjacencies(
                Node {
                    name: "leftward triangle",
                    shape: 0,
                    transform: Transform::Rotation(90 as f64),
                },
                &[(5,3),(3,1),(4,2)],
            ),
            Adjacencies(
                Node {
                    name: "downward triangle",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(5,0),(0,1),(4,3)],
            ),
            Adjacencies(
                Node {
                    name: "rightward triangle",
                    shape: 0,
                    transform: Transform::Rotation(270 as f64),
                },
                &[(5,1),(1,1),(4,0)],
            ),
            Adjacencies(
                Node {
                    name: "xy-aligned square",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(3,2),(0,2),(1,2),(2,2)],
            ),
            Adjacencies(
                Node {
                    name: "rotated square",
                    shape: 1,
                    transform: Transform::Rotation(30 as f64),
                },
                &[(2,0),(3,0),(0,0),(1,0)],
            ),
        ],
    },
    Tiling {
        name: "33344",
        shapes: &[3,4],
        graph: &[
            Adjacencies(
                Node {
                    name: "upward triangle",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(1,0),(1,1),(2,1)],
            ),
            Adjacencies(
                Node {
                    name: "downward triangle",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(0,0),(0,1),(2,3)],
            ),
            Adjacencies(
                Node {
                    name: "square",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(2,2),(0,2),(2,0),(1,2)],
            ),
        ],
    },
    Tiling {
        name: "33336",
        shapes: &[3,6],
        graph: &[
            Adjacencies(
                Node {
                    name: "northward triangle on hexagon",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(3,0),(7,1),(8,1)],
            ),
            Adjacencies(
                Node {
                    name: "north-westward triangle on hexagon",
                    shape: 0,
                    transform: Transform::Rotation(60 as f64),
                },
                &[(4,0),(6,0),(8,2)],
            ),
            Adjacencies(
                Node {
                    name: "south-westward triangle on hexagon",
                    shape: 0,
                    transform: Transform::Rotation(120 as f64),
                },
                &[(5,0),(7,2),(8,3)],
            ),
            Adjacencies(
                Node {
                    name: "southward triangle on hexagon",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(0,0),(6,1),(8,4)],
            ),
            Adjacencies(
                Node {
                    name: "south-eastward triangle on hexagon",
                    shape: 0,
                    transform: Transform::Rotation(240 as f64),
                },
                &[(1,0),(7,0),(8,5)],
            ),
            Adjacencies(
                Node {
                    name: "north-eastward triangle on hexagon",
                    shape: 0,
                    transform: Transform::Rotation(300 as f64),
                },
                &[(2,0),(6,2),(8,0)],
            ),
            Adjacencies(
                Node {
                    name: "northward triangle in triangles",
                    shape: 0,
                    transform: Transform::None,
                },
                &[(1,1),(3,1),(5,1)],
            ),
            Adjacencies(
                Node {
                    name: "southward triangle in triangles",
                    shape: 0,
                    transform: Transform::Rotation(180 as f64),
                },
                &[(4,1),(0,1),(2,1)],
            ),
            Adjacencies(
                Node {
                    name: "hexagon",
                    shape: 1,
                    transform: Transform::None,
                },
                &[(5,2),(0,2),(1,2),(2,2),(3,2),(4,2)],
            ),
        ],
    },
];

fn main() {
    let tilings: HashMap<_, _> = TILINGS.into_iter().map(|tiling| (tiling.name, tiling)).collect();

    let tiling = match env::args().collect::<Vec<String>>().get(1) {
        Some(name) => match tilings.get(&name[..]) {
            Some(tiling) => tiling,
            None => panic!("no tiling named '{}'", name),
        },
        None => panic!("please give a tiling name"),
    };

    let path: Vec<usize> = split_words(&read_line());
    let start = match path.get(0) {
        Some(start) => *start,
        None => panic!("provide a path of length > 0"),
    };

    let path = Path(start, &path[1..]);

    match tiling.traverse(&path) {
        Ok(n) => println!("{}", tiling.graph[n].0.name),
        Err(e) => panic!("{}", e),
    }
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("expected input from stdin");
    input
}

fn split_words<T>(s: &String) -> Vec<T> where T: FromStr {
    s.trim().split_whitespace().map(|x| match x.parse() {
        Ok(number) => number,
        Err(_) => panic!("could not parse {}", x),
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiling_shapes_match() -> Result<(), String> {
        for tiling in TILINGS.into_iter() {
            for (i, adjacencies) in tiling.graph.into_iter().enumerate() {
                if adjacencies.0.shape >= tiling.shapes.len() {
                    return Err(format!(
                        "tiling {}: node {}: shape {} is out of range",
                        tiling.name, i, adjacencies.0.shape,
                    ))
                }
                if adjacencies.1.len() != tiling.shapes[adjacencies.0.shape] {
                    return Err(format!(
                        "tiling {}: node {}: expected {} adjacencies but found {}",
                        tiling.name, i, tiling.shapes[adjacencies.0.shape], adjacencies.1.len(),
                    ))
                }
            }
        }
        Ok(())
    }

    #[test]
    fn tiling_edges_match() -> Result<(), String> {
        for tiling in TILINGS.into_iter() {
            for (i, adjacencies) in tiling.graph.into_iter().enumerate() {
                for (j, adjacency) in adjacencies.1.into_iter().enumerate() {
                    if adjacency.0 >= tiling.graph.len() {
                        return Err(format!(
                            "tiling {}: node {}: adjacency {}: node {} is out of range",
                            tiling.name, i, j, adjacency.0,
                        ))
                    }
                    if adjacency.1 >= tiling.graph[adjacency.0].1.len() {
                        return Err(format!(
                            "tiling {}: node {}: adjacency {}: edge {} is out of range",
                            tiling.name, i, j, adjacency.1,
                        ))
                    }
                    if tiling.graph[adjacency.0].1[adjacency.1].0 != i {
                        return Err(format!(
                            "tiling {}: node {}: adjacency {}: expected this node to be node-{}'s edge {}'s neighbor, found {}",
                            tiling.name, i, j, adjacency.0, adjacency.1, tiling.graph[adjacency.0].1[adjacency.1].0,
                        ))
                    }
                    if tiling.graph[adjacency.0].1[adjacency.1].1 != j {
                        return Err(format!(
                            "tiling {}: node {}: adjacency {}: expected this adjacency to be node-{}'s edge {}'s neighbor edge, found {}",
                            tiling.name, i, j, adjacency.0, adjacency.1, tiling.graph[adjacency.0].1[adjacency.1].1,
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}
