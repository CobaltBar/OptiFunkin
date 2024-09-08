use std::path::PathBuf;

use clap::Parser;
use max_rects::{
    bucket::Bucket, calculate_packed_percentage, max_rects::MaxRects, packing_box::PackingBox,
};
use rayon::prelude::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    /// Files or Folders to optimize and repack
    files: Vec<PathBuf>,

    /// Recursively go through directories for files
    #[arg(short, long)]
    recursive: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = CLI::parse();

    let mut files: Vec<&PathBuf> = cli.files.par_iter().filter(|&f| f.exists()).collect();

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
