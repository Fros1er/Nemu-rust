use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    std::env::set_current_dir("./disasm-util").unwrap();
    Command::new("cargo").args(&["--build", "-release"])
        .status().unwrap();
}