mod cli;
mod logconfig;
use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use log::{error, warn};
use max_rects::{bucket::Bucket, max_rects::MaxRects, packing_box::PackingBox};
use rayon::prelude::*;
use roxmltree::Document;
use tempfile::Builder;
use walkdir::WalkDir;

fn main() {
    let cli = cli::CLI::parse();
    logconfig::init(cli.verbose);

    let all_files = get_files(cli.files, cli.recursive);

    let temp_dir: tempfile::TempDir = match Builder::new().prefix("optifunkin").tempdir() {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create temporary directory: {}", e);
            return;
        }
    };

    repack_atlases(
        all_files
            .par_iter()
            .filter(|f| match f.extension() {
                Some(ext) => ext == "png" && Path::new(&f.with_extension("xml")).exists(),
                None => false,
            })
            .collect::<Vec<&PathBuf>>(),
    );

    match temp_dir.close() {
        Err(e) => error!("Failed to remove temporary directory: {}", e),
        _ => {}
    };
}

fn repack_atlases(files: Vec<&PathBuf>) {
    let text = match fs::read_to_string(files[0].with_extension("xml")) {
        Ok(txt) => txt,
        Err(e) => {
            error!("XML Reading Error: {}", e);
            return;
        }
    };

    let doc = match Document::parse(&text) {
        Ok(doc) => doc,
        Err(e) => {
            error!("XML Parsing Error: {}", e);
            return;
        }
    };

    let mut rects: Vec<Vec<i32>> = Vec::new();

    for element in doc.descendants() {
        if element.is_element() {
            //TODO proper checks
            if let Some(_) = element.attribute("x") {
                let mut rect = vec![
                    element.attribute("x").unwrap().parse::<i32>().unwrap(),
                    element.attribute("y").unwrap().parse::<i32>().unwrap(),
                    element.attribute("width").unwrap().parse::<i32>().unwrap(),
                    element.attribute("height").unwrap().parse::<i32>().unwrap(),
                ];

                if let Some(_) = element.attribute("frameX") {
                    rect.push(element.attribute("frameX").unwrap().parse::<i32>().unwrap());
                    rect.push(element.attribute("frameY").unwrap().parse::<i32>().unwrap());
                    rect.push(
                        element
                            .attribute("frameWidth")
                            .unwrap()
                            .parse::<i32>()
                            .unwrap(),
                    );
                    rect.push(
                        element
                            .attribute("frameHeight")
                            .unwrap()
                            .parse::<i32>()
                            .unwrap(),
                    );
                }

                rects.push(rect);
            }
        }
    }

    let mut boxes: Vec<PackingBox> = vec![];
    let bins = vec![Bucket::new(1024, 1024, 0, 0, 1)];

    for rect in rects {
        boxes.push(PackingBox::new(rect[2], rect[3]));
    }

    let (_, unplaced, _) = MaxRects::new(boxes.clone(), bins.clone()).place();

    if unplaced.len() > 0 {
        error!("Failed to place all objects");
    }
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
