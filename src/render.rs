use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use comrak::{markdown_to_html, ComrakOptions};
use handlebars::Handlebars;
use headless_chrome::Browser;
use printpdf::*;
use pulldown_cmark::CowStr;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::{Event, Tag};
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

        for (path, content) in templated_files {
            let output = format!(
                "file:///{}",
                env::current_dir()
                    .unwrap()
                    .join("tmp.html")
                    .to_str()
                    .unwrap()
            );

            fs::write(&output, content).unwrap();

            let pdf = tab
                .navigate_to(&output)
                .unwrap()
                .wait_until_navigated()
                .unwrap()
                .print_to_pdf(None)
                .unwrap();

            rendered_files.insert(path.with_extension("pdf"), pdf);

            fs::remove_file("tmp.html").unwrap();
        }

        rendered_files
    }
}

pub struct Inhouse<'a> {
    markdown: &'a Vec<Event<'a>>,
    position: usize,
    document: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerReference,
    width: f32,
    height: f32,
    font: Font,
}

impl<'a> Inhouse<'a> {
    fn new(markdown: String, title: String) -> Inhouse<'a> {
        let width = 209.9;
        let height = 297.0;
        let mut list_level = 0;

        let (doc, page1, layer1) = PdfDocument::new(title, Mm(width), Mm(height), "Layer 1");

        let mut font = Font::new(
            "assets/fonts/Roboto-Regular.ttf",
            "assets/fonts/Roboto-Bold.ttf",
            "assets/fonts/Roboto-Italic.ttf",
            "assets/fonts/Roboto-BoldItalic.ttf",
            &doc,
        );

        let current_layer = doc.get_page(page1).get_layer(layer1);

        current_layer.begin_text_section();

        current_layer.set_font(&font.get(), font.regular_size);
        current_layer.set_text_cursor(Mm(1.0), Mm(height - 7.0));
        current_layer.set_line_height(font.regular_size);
        // current_layer.set_word_spacing(font.calc_word_spacing());
        // current_layer.set_character_spacing(font.regular_size);
        current_layer.set_text_rendering_mode(TextRenderingMode::Fill);

        Inhouse {
            markdown: &pulldown_cmark::Parser::new(&markdown).collect(),
            position: 0,
            document: doc,
            page: page1,
            layer: current_layer,
            width,
            height,
            font,
        }
    }

    fn render_doc(&mut self) {
        while !self.is_at_end() {
            self.render();
        }

        self.layer.end_text_section();
    }

    fn render(&mut self) {
        match self.consume() {
            Event::Start(tag) => self.handle_start(tag),
            Event::End(tag) => self.handle_end(tag),
            Event::Text(text) => self.render_text(*text),
            Event::Code(_) => todo!(),
            Event::Html(_) => todo!(),
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak => self.layer.add_line_break(),
            Event::HardBreak => self.layer.add_line_break(),
            Event::Rule => todo!(),
            Event::TaskListMarker(_) => todo!(),
        }
    }

    fn handle_start(&mut self, tag: &Tag) {
        match tag {
            Tag::Paragraph => todo!(),
            Tag::Heading(_, _, _) => todo!(),
            Tag::BlockQuote => todo!(),
            Tag::CodeBlock(_) => todo!(),
            Tag::List(_) => todo!(),
            Tag::Item => todo!(),
            Tag::FootnoteDefinition(_) => todo!(),
            Tag::Table(_) => todo!(),
            Tag::TableHead => todo!(),
            Tag::TableRow => todo!(),
            Tag::TableCell => todo!(),
            Tag::Emphasis => todo!(),
            Tag::Strong => todo!(),
            Tag::Strikethrough => todo!(),
            Tag::Link(_, _, _) => todo!(),
            Tag::Image(_, _, _) => todo!(),
        }
    }

    fn handle_end(&mut self, tag: &Tag) {
        match tag {
            Tag::Paragraph => todo!(),
            Tag::Heading(_, _, _) => todo!(),
            Tag::BlockQuote => todo!(),
            Tag::CodeBlock(_) => todo!(),
            Tag::List(_) => todo!(),
            Tag::Item => todo!(),
            Tag::FootnoteDefinition(_) => todo!(),
            Tag::Table(_) => todo!(),
            Tag::TableHead => todo!(),
            Tag::TableRow => todo!(),
            Tag::TableCell => todo!(),
            Tag::Emphasis => todo!(),
            Tag::Strong => todo!(),
            Tag::Strikethrough => todo!(),
            Tag::Link(_, _, _) => todo!(),
            Tag::Image(_, _, _) => todo!(),
        }
    }

    fn render_text(&mut self, text: CowStr) {}

    fn load(&mut self) {}

    fn consume(&mut self) -> &Event {
        let event = self.markdown.get(self.position).unwrap();
        self.position += 1;
        event
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.markdown.len()
    }
}

impl Backend for Inhouse<'_> {
    fn render_files(
        files: &HashMap<PathBuf, String>,
        template: String,
        style_sheet: String,
    ) -> HashMap<PathBuf, Vec<u8>> {
        let mut rendered_files: HashMap<PathBuf, Vec<u8>> = HashMap::new();

        for (path, content) in files {
            let parser: Vec<Event> = pulldown_cmark::Parser::new(content).collect();

            for event in parser {
                match event {
                    Event::Start(start_event) => match start_event {
                        Tag::Paragraph => font.current_size = font.regular_size,
                        Tag::Heading(depth, _, _) => font.start_header(depth),
                        Tag::BlockQuote => todo!(),
                        Tag::CodeBlock(_) => todo!(),
                        Tag::List(_) => list_level += 1,
                        Tag::Item => todo!(),
                        Tag::FootnoteDefinition(_) => todo!(),
                        Tag::Table(_) => todo!(),
                        Tag::TableHead => todo!(),
                        Tag::TableRow => todo!(),
                        Tag::TableCell => todo!(),
                        Tag::Emphasis => font.is_italic = true,
                        Tag::Strong => font.is_bold = true,
                        Tag::Strikethrough => todo!(),
                        Tag::Link(_, _, _) => todo!(),
                        Tag::Image(_, _, _) => todo!(),
                    },
                    Event::End(end_event) => match end_event {
                        Tag::Paragraph => {
                            current_layer.add_line_break();
                            current_layer.add_line_break();
                        }
                        Tag::Heading(depth, _, _) => current_layer.add_line_break(),
                        Tag::BlockQuote => todo!(),
                        Tag::CodeBlock(_) => todo!(),
                        Tag::List(_) => list_level -= 1,
                        Tag::Item => todo!(),
                        Tag::FootnoteDefinition(_) => todo!(),
                        Tag::Table(_) => todo!(),
                        Tag::TableHead => todo!(),
                        Tag::TableRow => todo!(),
                        Tag::TableCell => todo!(),
                        Tag::Emphasis => font.is_italic = false,
                        Tag::Strong => font.is_bold = false,
                        Tag::Strikethrough => todo!(),
                        Tag::Link(_, _, _) => todo!(),
                        Tag::Image(_, _, _) => todo!(),
                    },
                    Event::Text(text) => {
                        current_layer.set_font(&font.get(), font.current_size);
                        current_layer.write_text(
                            format!("{}{}", " -> ".repeat(list_level), text.to_string()),
                            &font.get(),
                        );
                    }
                    Event::Code(_) => todo!(),
                    Event::Html(_) => todo!(),
                    Event::FootnoteReference(_) => todo!(),
                    Event::SoftBreak => current_layer.add_line_break(),
                    Event::HardBreak => current_layer.add_line_break(),
                    Event::Rule => todo!(),
                    Event::TaskListMarker(_) => todo!(),
                }
            }

            current_layer.end_text_section();

            rendered_files.insert(
                path.to_path_buf().with_extension("pdf"),
                doc.save_to_bytes().unwrap(),
            );
        }

        rendered_files
    }
}

pub struct Font {
    regular: IndirectFontRef,
    bold: IndirectFontRef,
    italic: IndirectFontRef,
    bold_italic: IndirectFontRef,
    is_bold: bool,
    is_italic: bool,
    current_size: f32,
    header_size: f32,
    regular_size: f32,
    header_size_scale_increment: f32,
}

impl Font {
    pub fn new(
        regular_path: &str,
        bold_path: &str,
        italic_path: &str,
        bold_italic_path: &str,
        doc: &PdfDocumentReference,
    ) -> Self {
        Font {
            regular: doc
                .add_external_font(File::open(regular_path).unwrap())
                .unwrap(),

            bold: doc
                .add_external_font(File::open(bold_path).unwrap())
                .unwrap(),

            italic: doc
                .add_external_font(File::open(italic_path).unwrap())
                .unwrap(),

            bold_italic: doc
                .add_external_font(File::open(bold_italic_path).unwrap())
                .unwrap(),

            is_bold: false,
            is_italic: false,
            current_size: 8.0,
            regular_size: 8.0,
            header_size: 14.0,
            header_size_scale_increment: 2.0,
        }
    }

    pub fn get(&self) -> IndirectFontRef {
        println!("{} {}", self.is_bold, self.is_italic);
        match (self.is_bold, self.is_italic) {
            (true, true) => self.bold_italic.clone(),
            (true, false) => self.bold.clone(),
            (false, true) => self.italic.clone(),
            (false, false) => self.regular.clone(),
        }
    }

    pub fn start_header(&mut self, depth: HeadingLevel) {
        self.is_bold = true;
        self.current_size = self.header_size
            - (self.header_size_scale_increment
                * match depth {
                    HeadingLevel::H1 => 1.0,
                    HeadingLevel::H2 => 2.0,
                    HeadingLevel::H3 => 3.0,
                    HeadingLevel::H4 => 4.0,
                    HeadingLevel::H5 => 5.0,
                    HeadingLevel::H6 => 6.0,
                });
    }

    pub fn calc_character_spacing(&self) -> f32 {
        0.0
    }

    pub fn calc_word_spacing(&self) -> f32 {
        0.0
    }

    pub fn calc_line_height(&self) -> f32 {
        0.0
    }
}

pub enum FontMode {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}
