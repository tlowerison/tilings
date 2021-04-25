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

struct Tile {
    degree: usize,
}

enum Transform {
    Rotation(f64),
    Reflection(f64),
    None,
}

// Node in the tiling graph, has sub-labels tile and transform.
struct Node<'a> {
    tile: &'a Tile,
    transform: &'a Transform,
}

// Adjacency is a node and its neighbors.
// The tuple (usize, usize) represents (neighbor node index, neighbor node's matching edge index).
struct Adjacency<'a>(Node<'a>, &'a[(usize, usize)]);

// Tiling is an adjacency list of nodes.
struct Tiling<'a> {
    name: &'a str,
    graph: &'a[Adjacency<'a>],
}

// Path is a traversal through a graph.
// Its first component represents the starting node,
// the second component represents each step of the path,
// where each member is that step's edge taken as a node's neighbor.
struct Path<'a>(usize, &'a[usize]);

impl<'a> Tiling<'a> {
    fn get(&self, n: usize) -> Result<&Adjacency, String> {
        match self.graph.get(n) {
            Some(a) => Ok(a),
            None => Err(format!("missing node: {}", n)),
        }
    }

    fn traverse(&self, path: &Path) -> Result<usize, String> {
        let mut n = path.0;
        let mut adjacency = match self.get(path.0) {
            Ok(a) => a,
            Err(_) => {
                return Err(format!("invalid starting node in path: {}", path.0));
            },
        };
        for edge in path.1.iter() {
            if *edge >= adjacency.1.len() {
                return Err(format!("invalid edge {} from node {}", edge, n))
            }
            adjacency = match adjacency.1.get(*edge) {
                Some(neighbor) => {
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

const SQUARE: Tile = Tile { degree: 4 };
const HEXAGON: Tile = Tile { degree: 6 };
const OCTAGON: Tile = Tile { degree: 8 };
const DODECAGON: Tile = Tile { degree: 12 };

const TILINGS: &'static [Tiling] = &[
    Tiling{
        name: "4444",
        graph: &[
            Adjacency(
                Node {
                    tile: &SQUARE,
                    transform: &Transform::None,
                },
                &[(0,2),(0,3),(0,0),(0,1)],
            ),
        ],
    },
    Tiling{
        name: "488",
        graph: &[
            Adjacency(
                Node {
                    tile: &SQUARE,
                    transform: &Transform::None,
                },
                &[(1,4),(1,6),(1,0),(1,2)],
            ),
            Adjacency(
                Node {
                    tile: &SQUARE,
                    transform: &Transform::None,
                },
                &[(0,2),(1,5),(0,3),(1,7),(0,0),(1,1),(0,1),(1,3)],
            ),
        ],
    },
    Tiling{
        name: "4612",
        graph: &[
            Adjacency(
                Node {
                    tile: &SQUARE,
                    transform: &Transform::None,
                },
                &[(5,6),(4,4),(5,0),(3,1)],
            ),
            Adjacency(
                Node {
                    tile: &SQUARE,
                    transform: &Transform::Rotation(60 as f64),
                },
                &[(3,3),(5,10),(4,0),(5,4)],
            ),
            Adjacency(
                Node {
                    tile: &SQUARE,
                    transform: &Transform::Rotation(120 as f64),
                },
                &[(5,8),(3,5),(5,2),(4,2)],
            ),
            Adjacency(
                Node {
                    tile: &HEXAGON,
                    transform: &Transform::None,
                },
                &[(5,7),(0,3),(5,11),(1,0),(5,3),(2,1)],
            ),
            Adjacency(
                Node {
                    tile: &HEXAGON,
                    transform: &Transform::None,
                },
                &[(1,2),(5,9),(2,3),(5,1),(0,1),(5,5)],
            ),
            Adjacency(
                Node {
                    tile: &DODECAGON,
                    transform: &Transform::None,
                },
                &[(0,2),(4,3),(2,2),(3,4),(1,3),(4,5),(0,0),(3,0),(2,0),(4,1),(1,2),(3,2)],
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
        Ok(n) => println!("{}", n),
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
