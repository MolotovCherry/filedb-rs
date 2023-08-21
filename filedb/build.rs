use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let schema = manifest.join("schema");
    let capnp_bin = Path::new(&manifest).join("bin/capnp");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=schema");

    capnpc::CompilerCommand::new()
        .capnp_executable(&capnp_bin)
        .file(schema.join("file_schema.capnp"))
        .run()
        .expect("schema compiler command");
}
