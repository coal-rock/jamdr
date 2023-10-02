use comrak::{markdown_to_html, ComrakOptions};
use handlebars::Handlebars;
use serde_json::json;
use std::{collections::HashMap, path::PathBuf};

// TODO: multithread!
pub fn render_files(
    files: &HashMap<PathBuf, String>,
    template: String,
) -> HashMap<PathBuf, String> {
    let mut rendered_files: HashMap<PathBuf, String> = HashMap::new();
    let mut hb = Handlebars::new();

    hb.register_template_string("default", template)
        .expect("invalid html template");

    for (path, content) in files {
        let html = markdown_to_html(&content, &ComrakOptions::default());
        let context = json!({ "content": html });
        let context = handlebars::Context::from(context);

        rendered_files.insert(
            path.to_path_buf(),
            hb.render_with_context("default", &context).expect("?"),
        );
    }

    rendered_files
}
