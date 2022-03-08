use clap_complete::{generate_to, shells};

include!("src/cli.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cli.rs");

    let out_dir = std::path::Path::new("target/completions/");
    drop(std::fs::remove_dir_all(out_dir));
    std::fs::create_dir_all(out_dir).unwrap();

    let bin_name = env!("CARGO_PKG_NAME");

    generate_to(shells::Bash, &mut build(), bin_name, out_dir).unwrap();
    generate_to(shells::Elvish, &mut build(), bin_name, out_dir).unwrap();
    generate_to(shells::Fish, &mut build(), bin_name, out_dir).unwrap();
    generate_to(shells::PowerShell, &mut build(), bin_name, out_dir).unwrap();
    generate_to(shells::Zsh, &mut build(), bin_name, out_dir).unwrap();
}
