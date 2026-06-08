use std::{fs, path::PathBuf};

#[test]
fn build_script_declares_frontend_dist_inputs() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let build_rs = fs::read_to_string(manifest_dir.join("build.rs"))
        .expect("build.rs should be readable");

    assert!(
        build_rs.contains("cargo:rerun-if-changed=../dist"),
        "build.rs must make Cargo rerun when frontend dist changes"
    );
}
