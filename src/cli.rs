use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CLI {
    /// Files or Folders to optimize and repack
    pub files: Vec<PathBuf>,

    /// Recursively go through directories for files
    #[arg(short, long)]
    pub recursive: bool,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
