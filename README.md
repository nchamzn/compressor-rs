# compressor-rs
Doing dubious image compression things

```
Download and re-compress images. Don't ask why

Usage: compressor-rs.exe [OPTIONS] --input-file <INPUT_FILE>

Options:
  -i, --input-file <INPUT_FILE>
          Input file with list of urls (one on each line)
  -d, --download-folder <DOWNLOAD_FOLDER>
          Folder where images are downloaded [default: download_path]
  -c, --compressed-files-folder <COMPRESSED_FILES_FOLDER>
          Where the compressed images should be saved [default: compressed_files]
  -q, --quality-factor <QUALITY_FACTOR>
          The jpeg compression quality setting [default: 80]
  -h, --help
          Print help
  -V, --version
          Print version
```
