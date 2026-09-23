pub mod config;
pub mod hal;
pub mod security;
pub mod server;
pub mod utils;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
