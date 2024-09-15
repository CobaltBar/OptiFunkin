mod cli;
mod logconfig;
use std::path::{Path, PathBuf};

use clap::Parser;
use log::{error, warn};
use rayon::prelude::*;
use walkdir::WalkDir;

fn main() {
    let cli = cli::CLI::parse();
    logconfig::init(cli.verbose);

    let all_files = get_files(cli.files, cli.recursive);

    let to_repack = all_files
        .par_iter()
        .filter(|f| match f.extension() {
            Some(ext) => ext == "png" && Path::new(&f.with_extension("xml")).exists(),
            None => false,
        })
        .collect::<Vec<&PathBuf>>();

    
}

fn get_files(files: Vec<PathBuf>, recursive: bool) -> Vec<PathBuf> {
    //Check and warn of paths that don't exist
    let nonexistent = files
        .par_iter()
        .filter(|f| !f.exists())
        .filter_map(|f| match f.to_str() {
            Some(s) => Some(s),
            None => {
                error!("Invalid Path: {}", f.display());
                None
            }
        })
        .collect::<Vec<&str>>()
        .join("', '");

    if !nonexistent.is_empty() {
        warn!("The following paths do not exist: '{}'", nonexistent);
    }

    // Get listed files from CLI
    let mut iterfiles = files
        .par_iter()
        .filter(|f| !f.is_dir() && f.exists())
        .cloned()
        .collect::<Vec<PathBuf>>();

    // Iterate through the folders and push all files to `iterfiles``
    for folder in files
        .par_iter()
        .filter(|f| f.is_dir() && f.exists())
        .collect::<Vec<&PathBuf>>()
    {
        let walk = if recursive {
            WalkDir::new(folder)
        } else {
            WalkDir::new(folder).max_depth(1)
        };

        for maybeentry in walk {
            if maybeentry.is_err() {
                error!("Error: {}", maybeentry.unwrap_err());
                continue;
            }

            let entry = maybeentry.ok().unwrap();
            if entry.file_type().is_file() {
                iterfiles.push(entry.into_path());
            }
        }
    }

    return iterfiles;
}
