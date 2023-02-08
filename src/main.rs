use futures::future::join_all;
use image_compressor::Factor;
use image_compressor::FolderCompressor;
use reqwest;
use reqwest::Url;
use std::collections::HashSet;
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

async fn download_files(links: &HashSet<Url>, download_folder: &str) -> anyhow::Result<()> {
    let bodies = join_all(links.into_iter().map(|url| async move {
        println!("Downloading {:?}", url);
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
                println!("Got b: {:?}", b.len());
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

    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
