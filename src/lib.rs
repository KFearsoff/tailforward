pub mod axumlib;
mod errors;
pub mod handle;
mod tailscale;

pub const CURRENT_VERSION: &str = "v1";
pub const SECRET: &str = "tskey";