use clap::Parser;

/// Download and re-compress images in a file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

pub struct Args {
    /// Input file with list of urls (one on each line)
    #[arg(short, long)]
    pub input_file: String,

    /// Folder where urls are downloaded to
    #[arg(short, long, default_value_t = String::from("download_path"))]
    pub download_folder: String,

    /// Where the compressed files should be saved to
    #[arg(short, long, default_value_t = String::from("compressed_files"))]
    pub compressed_files_folder: String,
}
