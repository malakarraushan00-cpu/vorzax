use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Add linker script path
    let linker_script = PathBuf::from("boot/linker.ld");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-arg=-T{}", linker_script.display());
    
    // Specify ARM64 target
    println!("cargo:rustc-link-arg=-march=armv8-a");
    
    // Ensure no standard library linking
    println!("cargo:rustc-link-arg=-nostdlib");
    
    // Recompile if linker script changes
    println!("cargo:rerun-if-changed=boot/linker.ld");
    println!("cargo:rerun-if-changed=boot/entry.asm");
}
