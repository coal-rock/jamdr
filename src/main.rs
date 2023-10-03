mod cli;
mod config;
mod render;

use clap::Parser;
use cli::Arguments;
use handlebars::Output;
use headless_chrome::{types::PrintToPdfOptions, Browser};
use std::{
    collections::HashMap,
    fs::read_to_string,
    fs::remove_file,
    fs::{self, write},
    path::PathBuf,
};

use crate::render::render_files;

fn main() {
    let args = Arguments::parse();

    println!("{:#?}", args);

    if args.file_paths.is_empty() {
        println!("Please enter one valid path.");
    }

    let files = read_files(args.file_paths).unwrap();
    let css = read_to_string("assets/styles/light.css").unwrap();
    let rendered_files = render_files(
        &files,
        read_to_string("assets/templates/template.html").unwrap(),
        css,
    );

    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();

    for file in rendered_files {
        write("tmp.html", file.1).unwrap();

        let pdf = tab
            .navigate_to(&"file:///home/nicole/Coding/Rust/jamdr/tmp.html")
            .unwrap()
            .wait_until_navigated()
            .unwrap()
            .print_to_pdf(None)
            .unwrap();

        // remove_file("tmp.html").unwrap();

        let mut output_file = file.0.clone();
        output_file.set_extension("pdf");
        println!("{:#?}", output_file);

        write(&output_file, pdf).unwrap();
    }

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
