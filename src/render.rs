use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use comrak::{markdown_to_html, ComrakOptions};
use handlebars::Handlebars;
use headless_chrome::Browser;
use serde_json::json;

pub trait Backend {
    fn render_files(
        files: &HashMap<PathBuf, String>,
        template: String,
        style_sheet: String,
    ) -> HashMap<PathBuf, Vec<u8>>;
}

pub struct Chromium {}

impl Backend for Chromium {
    fn render_files(
        files: &HashMap<PathBuf, String>,
        template: String,
        style_sheet: String,
    ) -> HashMap<PathBuf, Vec<u8>> {
        let mut templated_files: HashMap<PathBuf, String> = HashMap::new();
        let mut hb = Handlebars::new();

        hb.register_template_string("default", template)
            .expect("invalid html template");

        for (path, content) in files {
            let html = markdown_to_html(&content, &ComrakOptions::default());
            let context = json!({ "content": html,  "css": style_sheet});
            let context = handlebars::Context::from(context);

            templated_files.insert(
                path.to_path_buf(),
                hb.render_with_context("default", &context).expect("?"),
            );
        }

        let mut rendered_files: HashMap<PathBuf, Vec<u8>> = HashMap::new();
        let browser = Browser::default().unwrap();
        let tab = browser.new_tab().unwrap();

        for file in templated_files {
            let output = format!(
                "file:///{}",
                env::current_dir()
                    .unwrap()
                    .join("tmp.html")
                    .to_str()
                    .unwrap()
            );
            println!("{}", output);
            fs::write(&output, file.1).unwrap();

            let pdf = tab
                .navigate_to(&output)
                .unwrap()
                .wait_until_navigated()
                .unwrap()
                .print_to_pdf(None)
                .unwrap();

            rendered_files.insert(output.into(), pdf);

            fs::remove_file("tmp.html").unwrap();
        }

        rendered_files
    }
}
