use std::env::var;
use std::path::Path;
use wayland_scanner::{generate_code, Side};

fn main() {
    let out_dir = var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    generate_code(
        "./protocols/river-layout-v3.xml",
        out_dir.join("river-layout-v3.rs"),
        Side::Client,
    );
}
