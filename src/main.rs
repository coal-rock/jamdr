mod cli;
mod config;
mod render;

use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use clap::Parser;
use cli::Arguments;

use crate::render::render_files;

fn main() {
    let args = Arguments::parse();

    println!("{:#?}", args);

    if args.file_paths.is_empty() {
        println!("Please enter one valid path.");
    }

    let files = read_files(args.file_paths).unwrap();
    let rendered_files = render_files(&files, read_to_string("template.html").unwrap());
    println!("{:#?}", rendered_files);

    // let config: Config = load_config(args.config_path).unwrap();
}

fn read_files(files: Vec<PathBuf>) -> Option<HashMap<PathBuf, String>> {
    let mut out_files = HashMap::new();

    for file in files {
        match read_to_string(&file) {
            Ok(content) => {
                out_files.insert(file, content);
            }
            Err(_) => return None,
        }
    }

    Some(out_files)
}
