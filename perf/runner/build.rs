use std::process::Command;

fn main() {
    let output = Command::new(std::env::var_os("RUSTC").unwrap())
        .args(["--version", "--verbose"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let compiler = String::from_utf8(output.stdout).unwrap().replace('\n', " | ");
    println!("cargo:rustc-env=BUILD_COMPILER={compiler}");
    println!("cargo:rustc-env=BUILD_PROFILE={}", std::env::var("PROFILE").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
}
