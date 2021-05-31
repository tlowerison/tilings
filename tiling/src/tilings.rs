extern crate rmp_serde;
use crate::tiling::Tiling;
use tiling_config::SerializedTilings;

static SER_TILINGS_BIN: &'static [u8] = include_bytes!("tilings.bin");
static mut SER_TILINGS: Option<SerializedTilings> = None;

pub fn init() {
    unsafe {
        match SER_TILINGS {
            Some(_) => {},
            None => {
                let ser_tilings: SerializedTilings = rmp_serde::from_read_ref(&SER_TILINGS_BIN).unwrap();
                SER_TILINGS = Some(ser_tilings);
            }
        }
    }
}

pub fn get_tiling(name: &str) -> Result<Tiling, String> {
    unsafe {
        match &SER_TILINGS {
            Some(ser_tilings) => match ser_tilings.0.get(name) {
                None => Err(String::from(format!("no tiling found with name {}", name))),
                Some(ser_tiling) => match ser_tiling.as_tiling() {
                    Err(e) => Err(e),
                    Ok(tiling) => Tiling::new(tiling),
                },
            },
            None => {
                init();
                return get_tiling(name)
            }
        }
    }
}

pub fn ser_tilings() -> &'static Option<SerializedTilings> {
    unsafe {
        &SER_TILINGS
    }
}
