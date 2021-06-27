const SERVER_HOST_ENV_KEY: &'static str = "SERVER_HOST";

fn main() {
    let server_host = std::env::var(SERVER_HOST_ENV_KEY).unwrap();
    println!("cargo:rustc-env={}={}", SERVER_HOST_ENV_KEY, server_host);
}
