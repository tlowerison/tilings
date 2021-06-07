extern crate serde_yaml;
extern crate rmp_serde;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use tiling_config::SerializedTilings;

static IN_PATH: &str = "../tilings.yaml";
static OUT_PATH: &str = "./src/tilings.bin";

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed={}", IN_PATH);

    let in_path = Path::new(IN_PATH);
    let ser_yaml = fs::read_to_string(&in_path).expect(&format!("could not find expected input data file {}", IN_PATH));
    let ser_tilings = serde_yaml::from_str::<SerializedTilings>(&ser_yaml).unwrap().obfuscate_proto_tile_names();
    let ser_rmp: Vec<u8> = rmp_serde::to_vec(&ser_tilings).unwrap();

    let out_path = Path::new(OUT_PATH);
    let mut file = File::create(&out_path)?;
    file.write_all(ser_rmp.as_slice())?;
    file.flush()?;

    Ok(())
}
