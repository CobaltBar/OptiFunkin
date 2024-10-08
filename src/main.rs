mod cli;
mod logconfig;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use clap::Parser;
use image::{imageops, DynamicImage, ImageBuffer, ImageReader};
use log::{error, warn};

use rayon::prelude::*;
use roxmltree::Document;
use sanitize_filename::sanitize;
use tempfile::Builder;
use walkdir::WalkDir;

fn main() {
    let cli = cli::CLI::parse();
    logconfig::init(cli.verbose);

    let all_files = get_files(cli.files, cli.recursive);

    let temp_dir: tempfile::TempDir = match Builder::new().prefix("optifunkin").tempdir() {
        Ok(t) => t,
        Err(e) => {
            error!("Temporary Directory Creation Error: {}", e);
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
        &temp_dir.as_ref().to_path_buf(),
    );

    println!("{}", temp_dir.as_ref().display());
    thread::sleep(Duration::from_secs(30));

    match temp_dir.close() {
        Err(e) => error!("Temporary Directory Removal Error: {}", e),
        _ => {}
    };
}

fn repack_atlases(files: Vec<&PathBuf>, temp_dir: &PathBuf) {
    for file in files {
        //Read XML
        let text = match fs::read_to_string(file.with_extension("xml")) {
            Ok(txt) => txt,
            Err(e) => {
                error!("XML Reading Error: {}", e);
                continue;
            }
        };

        let doc = match Document::parse(&text) {
            Ok(doc) => doc,
            Err(e) => {
                error!("XML Parsing Error: {}", e);
                continue;
            }
        };

        let image = match ImageReader::open(file) {
            Ok(image) => match image.decode() {
                Ok(image) => image,
                Err(e) => {
                    error!("Image decoding error: {}", e);
                    continue;
                }
            },
            Err(e) => {
                error!("Image reading error: {}", e);
                continue;
            }
        };

        let mut rects: HashMap<String, Vec<i32>> = HashMap::new();
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

                    rects.retain(|_, r| &r[..4] != &rect[..4]);
                    rects.insert(element.attribute("name").unwrap().to_string(), rect);
                }
            }
        }

        let folder = temp_dir.join(file.with_extension("").file_name().unwrap());

        let mut images: HashMap<String, DynamicImage> = HashMap::new();

        for (name, rect) in rects {
            let mut img = image.crop_imm(
                rect[0].try_into().unwrap(),
                rect[1].try_into().unwrap(),
                rect[2].try_into().unwrap(),
                rect[3].try_into().unwrap(),
            );

            if rect.len() == 8 {
                let mut new_img = ImageBuffer::from_pixel(
                    rect[6].try_into().unwrap(),
                    rect[7].try_into().unwrap(),
                    image::Rgba([0, 0, 0, 0]),
                );

                let x: i64 = rect[4].try_into().unwrap();
                let y: i64 = rect[5].try_into().unwrap();

                imageops::overlay(&mut new_img, &img, -x, -y);

                img = DynamicImage::ImageRgba8(new_img);
            }

            images.insert(name, img);
        }

        match fs::create_dir(&folder) {
            Err(e) => {
                error!("Directory Creation Error: {}", e);
                continue;
            }
            _ => {}
        };

        for (name, img) in images {
            match img.save_with_format(
                temp_dir.join(&folder).join(sanitize(name) + ".png"),
                image::ImageFormat::Png,
            ) {
                Err(e) => {
                    error!("Image Saving Error: {}", e);
                    continue;
                }
                _ => {}
            };
        }
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

    iterfiles
}
