use futures::future::join_all;
use image;
use image_compare;
use image_compressor::Factor;
use image_compressor::FolderCompressor;
use reqwest;
use reqwest::Url;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};
use tokio::runtime;
mod cmd_args;
use clap::Parser;
use cmd_args::Args;
use std::ffi::OsStr;

async fn download_files(links: &HashSet<Url>, download_folder: &str) -> anyhow::Result<()> {
    let bodies = join_all(links.into_iter().map(|url| async move {
        println!("Downloading {:?}", url.to_string());
        let resp = reqwest::get(url.to_string()).await?;
        let bytes = resp.bytes().await;
        let file_name = url.path_segments().unwrap().last().unwrap();

        println!("file_name: {:?}", file_name);
        Ok((file_name.to_string(), bytes))
    }))
    .await;

    for b in bodies {
        match b {
            Ok((name, Ok(b))) => {
                let path = Path::new(download_folder).join(name);
                if let Ok(mut file) = File::create(path) {
                    file.write_all(&b)?;
                }
            }
            Ok((_, Err(e))) | Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
}

// "2a59ff7edb5987fbf1af35865b52417ddc0032c15b48b042abb75f423efc48ed._RI_V_TTW_SX1920_.jpg"
// Gets renamed as
// "2a59ff7edb5987fbf1af35865b52417ddc0032c15b48b042abb75f423efc48ed.jpg"
// by the compressor, so lets rename the files so they are correct.
fn fix_filenames(uncompressed_folder: &str, compressed_folder: &str) {
    let compressed_paths = fs::read_dir(compressed_folder).unwrap();

    let uncompressed_paths = fs::read_dir(uncompressed_folder)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().file_name().unwrap().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    for compressed_path in compressed_paths {
        let compressed_path = compressed_path.unwrap().path();
        let stem = compressed_path.file_stem().unwrap().to_str().unwrap();
        if let Some(filename) = uncompressed_paths.iter().find(|f| f.starts_with(stem)) {
            let new_path = Path::new(compressed_folder).join(filename);
            fs::rename(compressed_path, new_path);
        }
    }
}

fn resize_all_in_folder(input_folder: &str, output_folder: &str, quality_factor: f32) {
    let origin = PathBuf::from(input_folder);
    let dest = PathBuf::from(output_folder);

    let mut comp = FolderCompressor::new(origin, dest);
    comp.set_factor(Factor::new(quality_factor, 1.0));
    comp.set_thread_count(4);

    match comp.compress() {
        Ok(_) => {}
        Err(e) => println!("Cannot compress the folder!: {}", e),
    }
}

fn compare_all_images_in_folders(uncompressed_folder: &str, compressed_folder: &str) {
    let filenames = fs::read_dir(compressed_folder)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().file_name().unwrap().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    for filename in filenames {
        let compressed_path = Path::new(compressed_folder).join(&filename);
        let uncompressed_path = Path::new(uncompressed_folder).join(&filename);

        let image_one = image::open(&uncompressed_path)
            .expect(&format!("Could not open image: {:?}", uncompressed_path))
            .into_rgb8();
        let image_two = image::open(&compressed_path)
            .expect(&format!("Could not open image: {:?}", compressed_path))
            .into_rgb8();
        let result = image_compare::rgb_similarity_structure(
            &image_compare::Algorithm::RootMeanSquared,
            &image_one,
            &image_two,
        )
        .expect("Images had different dimensions");

        println!(
            "Similarity after compression of {:?} is {:?}",
            filename, result.score
        );
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let download_folder = args.download_folder;
    let compressed_files_folder = args.compressed_files_folder;
    std::fs::create_dir_all(&compressed_files_folder);
    std::fs::create_dir_all(&download_folder);

    let lines = read_lines(args.input_file)?;

    let mut links: HashSet<Url> = HashSet::new();
    for line in lines {
        if let Ok(line) = line {
            if let Ok(url) = line.parse::<Url>() {
                links.insert(url);
            } else {
                println!("Skipping {:?}", line);
            }
        }
    }

    // println!("{:?}", links);

    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let handle = rt.handle();

    handle.block_on(download_files(&links, &download_folder))?;

    resize_all_in_folder(
        &download_folder,
        &compressed_files_folder,
        args.quality_factor,
    );

    fix_filenames(&download_folder, &compressed_files_folder);

    if args.image_comparison {
        compare_all_images_in_folders(&download_folder, &compressed_files_folder);
    }

    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
