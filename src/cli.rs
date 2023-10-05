use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
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

    #[arg(short = 'b', long = "backend", default_value = "chromium")]
    pub backend: BackendType,

    #[arg()]
    pub file_paths: Vec<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Arguments {
    pub fn validate_args(self) -> Arguments {
        if self.file_paths.len() > 1 && (self.stdout || self.output_path.is_some()) {
            eprintln!("output name cannot be specified if multiple files are given");
            process::exit(1);
        }

        return self;
    }
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

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq, Parser)]
pub enum BackendType {
    Inhouse,
    Chromium,
}
