use crate::pkg::Pkg;

pub const STORAGE_INTERFACE: &str = "pm:lipl-core/types@0.3.4";
pub const STORAGE: Pkg = Pkg::new("pm", "lipl-storage-fs", "0.1.8");

pub const SERVICE_INTERFACE: &str = "wasi:http/handler@0.3.0";
pub const SERVICE: Pkg = Pkg::new("pm", "lipl-service", "0.1.3");

pub const SERVER: Pkg = Pkg::new("pm", "lipl-server", "0.1.3");
