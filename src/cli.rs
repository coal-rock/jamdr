use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(long = "css")]
    pub custom_css: Option<String>,

    #[arg(short = 'w', long = "watch", default_value = "false")]
    pub watch: bool,

    #[arg(long = "stdout", default_value = "false")]
    pub stdout: bool,

    #[arg(short = 't', long = "type", default_value = "pdf")]
    pub output_type: Option<OutputType>,

    #[arg(short = 'o', long = "output")]
    pub output_path: Option<PathBuf>,

    #[arg()]
    pub file_paths: Vec<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Render {},
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq, Parser)]
pub enum OutputType {
    PDF,
    HTML,
}
