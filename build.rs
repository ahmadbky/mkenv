use rustc_version::{version_meta, Channel};

fn main() {
    if let Ok(Channel::Nightly | Channel::Dev) = version_meta().map(|v| v.channel) {
        println!("cargo:rustc-cfg=nightly");
    }
    println!("cargo:rustc-check-cfg=cfg(nightly)")
}
