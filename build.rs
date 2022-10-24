use cargo_bpf_lib as cargo_bpf;
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
}
