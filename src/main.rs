use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    /// Files or Folders to optimize and repack
    files: Vec<String>,

    /// Recursively go through directories for files
    #[arg(short, long)]
    recursive: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = CLI::parse();
}
