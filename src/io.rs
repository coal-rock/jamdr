use dirs;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::process;

pub fn write_files(files: &HashMap<PathBuf, Vec<u8>>) -> io::Result<()> {
    for (filename, data) in files {
        match fs::write(&filename, data) {
            Ok(_) => {}
            Err(_) => (),
        }
    }

    Ok(())
}

pub fn read_files(files: Vec<PathBuf>) -> Option<HashMap<PathBuf, String>> {
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

pub fn try_create_config_dir() {
    match env::consts::OS {
        "linux" => {
            let home_directory = dirs::home_dir().unwrap().join(".config/jamdr/");
        }
        "windows" => {}
        "macos" => {}
        _ => {
            eprintln!("operating system not supported");
            process::exit(1);
        }
    }
}

pub fn try_create_config_dir_linux() {}
