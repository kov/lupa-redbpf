use cargo_bpf_lib as cargo_bpf;
use std::process::Command;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let cargo = PathBuf::from(env::var("CARGO").unwrap());
    let probes = Path::new("probes");

    let mut opts = cargo_bpf::BuildOptions::default();
    opts.target_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    cargo_bpf::build(&cargo, &probes, &mut vec![], &opts)
        .expect("couldn't compile ingraind-probes");

    cargo_bpf::probe_files(&probes)
        .expect("couldn't list probe files")
        .iter()
        .for_each(|file| {
            println!("cargo:rerun-if-changed={}", file);
        });

    env::set_current_dir("web").expect("Failed to enter 'web' directory");
    let web_target_dir = opts.target_dir.join("web");

    let status = Command::new("trunk")
        .arg("build")
        .arg("-d")
        .arg(web_target_dir)
        .status()
        .expect("Failed to build web frontend with trunk");

    assert!(status.success());

    for file in &["Cargo.toml", "index.html", "src/main.rs"] {
        println!("cargo:rerun-if-changed=web/{}", file);
    }
}
