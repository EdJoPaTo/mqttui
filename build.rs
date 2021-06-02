use std::env;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::Path;

use clap::Shell;

include!("src/cli.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cli.rs");

    let outdir = Path::new("completions/");
    drop(remove_dir_all(outdir));
    create_dir_all(outdir).unwrap();

    let mut app = build();
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Elvish, outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Fish, outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::PowerShell, outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, outdir);
}
