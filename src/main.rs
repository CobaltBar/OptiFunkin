mod cli;
mod logconfig;
use std::path::PathBuf;

use clap::Parser;
use log::{error, info, warn};
use rayon::prelude::*;

fn main() {
    let cli = cli::CLI::parse();
    logconfig::init(cli.verbose);

    let files: Vec<&PathBuf> = cli.files.par_iter().filter(|&f| f.exists()).collect();

    let nonexistent = cli
        .files
        .par_iter()
        .filter(|&f| !f.exists())
        .map(|f| f.to_str().unwrap())
        .collect::<Vec<&str>>()
        .join("', '");

    if nonexistent.len() != 0 {
        println!("WARN: The following paths do not exist: '{}'", nonexistent);
    }

    println!("{:?}", files);
}
