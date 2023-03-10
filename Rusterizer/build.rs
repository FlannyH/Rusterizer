use copy_to_output::copy_to_output;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=res/*");
    copy_to_output("assets", &env::var("PROFILE").unwrap()).expect("Failed to copy assets folder!")
}
