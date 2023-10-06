mod cli;
mod config;
mod fs_utils;
mod render;

use clap::Parser;
use cli::BackendType;
use render::Backend;
use render::Chromium;
use std::fs::read_to_string;

use crate::cli::Arguments;
use crate::render::Inhouse;

fn main() {
    let args = Arguments::parse().validate_args();
    println!("{:#?}", args);

    let files = fs_utils::read_files(args.file_paths).unwrap();
    let css = read_to_string("assets/styles/light.css").unwrap();
    let template = read_to_string("assets/templates/template.html").unwrap();

    let rendered_files = match args.backend {
        BackendType::Inhouse => Inhouse::render_files(&files, template, css),
        BackendType::Chromium => Chromium::render_files(&files, template, css),
    };

    match fs_utils::write_files(&rendered_files) {
        Ok(_) => println!("succsessfully wrote {} file(s)", rendered_files.len()),
        Err(_) => println!("error writing one or more file(s)"),
    }
}
