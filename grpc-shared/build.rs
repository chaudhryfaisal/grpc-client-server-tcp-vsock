use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Configure tonic-build
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&out_dir)
        .compile(&["proto/signing.proto"], &["proto"])?;
    
    // Tell cargo to rerun this build script if the proto file changes
    println!("cargo:rerun-if-changed=proto/signing.proto");
    
    Ok(())
}