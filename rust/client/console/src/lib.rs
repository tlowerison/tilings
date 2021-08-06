use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! debug {
    ($($args:expr),*) => {{
        let mut to_debug = String::from("");
        $(to_debug = format!("{} {:?}", to_debug, $args);)*
        debug(&to_debug);
    }}
}

#[macro_export]
macro_rules! err {
    ($($args:expr),*) => {{
        let mut to_err = String::from("");
        $(to_err = format!("{} {:?}", to_err, $args);)*
        err(&to_err);
    }}
}

#[macro_export]
macro_rules! log {
    ($($args:expr),*) => {{
        let mut to_log = String::from("");
        $(to_log = format!("{} {:?}", to_log, $args);)*
        log(&to_log);
    }}
}

#[macro_export]
macro_rules! warning {
    ($($args:expr),*) => {{
        let mut to_warning String::from("");
        $(to_warning = format!("{} {:?}", to_warning, $args);)*
        warning(&to_warning);
    }}
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn debug(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn err(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warning(s: &str);
}
