#[macro_use] extern crate rocket;

mod account;
mod atlas;
mod common;
mod health_check;
mod labels;
mod polygon;
mod search;
mod tiling_type;
mod tiling;

pub use self::account::*;
pub use self::atlas::*;
pub use self::health_check::*;
pub use self::labels::*;
pub use self::polygon::*;
pub use self::search::*;
pub use self::tiling_type::*;
pub use self::tiling::*;

pub const BASE_PATH: &'static str = "/api/tilings";
