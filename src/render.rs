use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use comrak::{markdown_to_html, ComrakOptions};
use freetype;
use freetype::face::LoadFlag;
use freetype::Face;
use freetype::Library;
use handlebars::Handlebars;
// use headless_chrome::Browser;
use printpdf::*;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Options;
use pulldown_cmark::{Event, Tag};
use serde_json::json;

use crate::extract;

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
        // let browser = Browser::default().unwrap();
        // let tab = browser.new_tab().unwrap();

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

            // let pdf = tab
            //     .navigate_to(&output)
            //     .unwrap()
            //     .wait_until_navigated()
            //     .unwrap()
            //     .print_to_pdf(None)
            //     .unwrap();

            // rendered_files.insert(path.with_extension("pdf"), pdf);

            fs::remove_file("tmp.html").unwrap();
        }

        rendered_files
    }
}

pub struct Inhouse<'a> {
    markdown: Vec<Event<'a>>,
    position: usize,
    page_position: (Mm, Mm),
    // list depth: if entry is none, list is bulleted, if entry is some, list is numbered
    list_depth: Vec<Option<u64>>,
    document: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerReference,
    font: Font,
    style: Style,
}

pub struct Style {
    width: f32,
    height: f32,
    vertical_padding: f32,
    horizontal_padding: f32,
    line_height: f32,
}

impl<'a> Inhouse<'a> {
    fn new(markdown: &'a str, title: String) -> Inhouse<'a> {
        let width = 209.9;
        let height = 297.0;

        let (doc, page1, layer1) = PdfDocument::new(title, Mm(width), Mm(height), "Layer 1");

        let font = Font::new(
            "assets/fonts/Roboto-Regular.ttf",
            "assets/fonts/Roboto-Bold.ttf",
            "assets/fonts/Roboto-Italic.ttf",
            "assets/fonts/Roboto-BoldItalic.ttf",
            &doc,
        );

        let style = Style {
            width,
            height,
            vertical_padding: 10.0,
            horizontal_padding: 2.0,
            line_height: font.regular_size + 2.0,
        };

        let current_layer = doc.get_page(page1).get_layer(layer1);
        current_layer.begin_text_section();

        current_layer.set_font(&font.get(), font.regular_size);
        current_layer.set_text_cursor(
            Mm(style.horizontal_padding),
            Mm(height - style.vertical_padding + (style.line_height * 0.45)),
        );
        current_layer.set_line_height(style.line_height);
        current_layer.set_text_rendering_mode(TextRenderingMode::Fill);

        Inhouse {
            markdown: pulldown_cmark::Parser::new_ext(&markdown, Options::ENABLE_STRIKETHROUGH)
                .collect(),
            position: 0,
            page_position: (
                Mm(style.horizontal_padding),
                Mm(height - style.vertical_padding),
            ),
            list_depth: vec![],
            document: doc,
            page: page1,
            layer: current_layer,
            font,
            style,
        }
    }

    fn render_doc(&mut self) {
        while !self.is_at_end() {
            self.render();
        }

        self.layer.end_text_section();
    }

    fn save_doc(self) -> Vec<u8> {
        self.document.save_to_bytes().unwrap()
    }

    fn render(&mut self) {
        // println!("{:#?}", self.peek());

        match self.peek() {
            Event::Start(_) => self.handle_start(),
            Event::End(_) => self.handle_end(),
            Event::Text(_) => self.render_text(),
            Event::Code(_) => todo!(),
            Event::Html(_) => todo!(),
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak => self.line_break(),
            Event::HardBreak => self.line_break(),
            Event::Rule => todo!(),
            Event::TaskListMarker(_) => todo!(),
        }
    }

    fn handle_start(&mut self) {
        let tag = extract!(self.consume().clone(), Event::Start);

        match tag {
            Tag::Paragraph => {
                self.font.current_size = self.font.regular_size;
            }
            Tag::Heading(heading_level, _, _) => {
                self.font.start_header(heading_level);
                self.render();

                println!("{}", self.style.horizontal_padding);
                println!("{:#?}", self.page_position.0);
                let line = Line {
                    points: vec![
                        (
                            Point {
                                x: Mm(self.style.horizontal_padding).into_pt(),
                                y: (self.page_position.1 + Pt(self.style.line_height).into())
                                    .into_pt(),
                            },
                            true,
                        ),
                        (
                            Point {
                                x: self.page_position.0.into_pt(),
                                y: (self.page_position.1 + Pt(self.style.line_height).into())
                                    .into_pt(),
                            },
                            true,
                        ),
                    ],
                    is_closed: true,
                };

                println!("{:#?}", line);
                self.layer.add_line(line);
            }
            Tag::BlockQuote => todo!(),
            Tag::CodeBlock(_) => todo!(),
            Tag::List(list) => {
                self.list_depth.push(list);
            }
            Tag::Item => {
                let number = self.list_depth.pop().unwrap();

                let number_str = match number {
                    Some(number) => number.to_string(),
                    None => "*".to_string(),
                };

                let text = format!("    {} ", number_str);

                self.layer.write_text(&text, &self.font.get());
                self.list_depth.push(number.map(|x| x + 1));

                self.page_position.0 += self.calc_text_width(text.to_string());
            }
            Tag::FootnoteDefinition(_) => todo!(),
            Tag::Table(_) => todo!(),
            Tag::TableHead => todo!(),
            Tag::TableRow => todo!(),
            Tag::TableCell => todo!(),
            Tag::Emphasis => self.font.is_italic = true,
            Tag::Strong => self.font.is_bold = true,
            Tag::Strikethrough => self.font.is_strikethrough = true,
            Tag::Link(_, _, _) => todo!(),
            Tag::Image(_, _, _) => todo!(),
        }
    }

    fn handle_end(&mut self) {
        let tag = extract!(self.consume().clone(), Event::End);

        match tag {
            Tag::Paragraph => self.line_break(),
            Tag::Heading(_, _, _) => {
                self.line_break();
                self.line_break();
                self.font.reset_formatting();
            }
            Tag::BlockQuote => todo!(),
            Tag::CodeBlock(_) => todo!(),
            Tag::List(_) => {
                self.list_depth.pop();
                self.line_break();
            }
            Tag::Item => self.line_break(),
            Tag::FootnoteDefinition(_) => todo!(),
            Tag::Table(_) => todo!(),
            Tag::TableHead => todo!(),
            Tag::TableRow => todo!(),
            Tag::TableCell => todo!(),
            Tag::Emphasis => self.font.is_italic = false,
            Tag::Strong => self.font.is_bold = false,
            Tag::Strikethrough => self.font.is_strikethrough = false,
            Tag::Link(_, _, _) => todo!(),
            Tag::Image(_, _, _) => todo!(),
        }
    }

    fn render_text(&mut self) {
        let text = extract!(self.peek(), Event::Text);

        self.layer
            .set_font(&self.font.get(), self.font.current_size);
        self.layer.write_text(text.to_string(), &self.font.get());

        let x_before = self.page_position.0;

        self.page_position.0 += self.calc_text_width(text.to_string());

        if self.font.is_strikethrough {
            let y = (self.page_position.1 + Pt(self.style.line_height * 1.55).into()).into_pt();

            let line = Line {
                points: vec![
                    (
                        Point {
                            x: x_before.into_pt(),
                            y,
                        },
                        true,
                    ),
                    (
                        Point {
                            x: self.page_position.0.into_pt(),
                            y,
                        },
                        true,
                    ),
                ],
                is_closed: true,
            };

            self.layer.add_line(line);
        }

        self.consume();
    }

    fn calc_vert_scale(&self) -> i64 {
        let font = self.font.get_freetype();

        if let Ok(_) = font.load_char(0x0020, LoadFlag::NO_SCALE) {
            font.glyph().metrics().vertAdvance
        } else {
            1000
        }
    }

    fn calc_text_width(&self, text: String) -> Mm {
        let font = self.font.get_freetype();

        let sum_width = text.chars().fold(0, |acc, ch| {
            if let Ok(_) = font.load_char(ch as usize, freetype::face::LoadFlag::NO_SCALE) {
                let glyph_w = font.glyph().metrics().horiAdvance;
                acc + glyph_w
            } else {
                acc
            }
        });

        Pt(sum_width as f32 / (self.calc_vert_scale() as f32 / self.font.current_size)).into()
    }

    fn line_break(&mut self) {
        self.layer.add_line_break();
        self.page_position.1 -= Pt(self.style.line_height).into();
        self.page_position.0 = Mm(self.style.horizontal_padding);
    }

    fn load(&mut self) {}

    fn consume(&mut self) -> &Event {
        let event = self.markdown.get(self.position).unwrap();
        self.position += 1;
        event
    }

    fn peek(&self) -> &Event {
        self.markdown.get(self.position).unwrap()
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
            let mut renderer = Inhouse::new(
                content,
                path.file_name().unwrap().to_str().unwrap().to_string(),
            );

            renderer.render_doc();

            rendered_files.insert(
                path.to_path_buf().with_extension("pdf"),
                renderer.save_doc(),
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
    ft_regular: Face,
    ft_bold: Face,
    ft_italic: Face,
    ft_bold_italic: Face,
    is_bold: bool,
    is_italic: bool,
    is_strikethrough: bool,
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
        let library = freetype::Library::init().unwrap();

        let regular = Font::load_font(&doc, &library, 0, regular_path);
        let bold = Font::load_font(&doc, &library, 0, bold_path);
        let italic = Font::load_font(&doc, &library, 0, italic_path);
        let bold_italic = Font::load_font(&doc, &library, 0, bold_italic_path);

        Font {
            regular: regular.0,
            bold: bold.0,
            italic: italic.0,
            bold_italic: bold_italic.0,
            ft_regular: regular.1,
            ft_bold: bold.1,
            ft_italic: italic.1,
            ft_bold_italic: bold_italic.1,
            is_bold: false,
            is_italic: false,
            is_strikethrough: false,
            current_size: 8.0,
            regular_size: 8.0,
            header_size: 14.0,
            header_size_scale_increment: 2.0,
        }
    }

    fn load_font(
        doc: &PdfDocumentReference,
        library: &Library,
        count: isize,
        path: &str,
    ) -> (IndirectFontRef, Face) {
        (
            doc.add_external_font(File::open(path).unwrap()).unwrap(),
            library.new_face(path, count).unwrap(),
        )
    }

    pub fn get(&self) -> IndirectFontRef {
        match (self.is_bold, self.is_italic) {
            (true, true) => self.bold_italic.clone(),
            (true, false) => self.bold.clone(),
            (false, true) => self.italic.clone(),
            (false, false) => self.regular.clone(),
        }
    }

    pub fn get_freetype(&self) -> &Face {
        match (self.is_bold, self.is_italic) {
            (true, true) => &self.ft_bold_italic,
            (true, false) => &self.ft_bold,
            (false, true) => &self.ft_italic,
            (false, false) => &self.ft_regular,
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

    fn reset_formatting(&mut self) {
        self.is_bold = false;
        self.is_italic = false;
        self.is_strikethrough = false;
    }
}

/// i am so very sorry
/// THIS CAN PANIC (it shouldn't tho, just use it properly PLEASE)
/// usage:
///
/// ```
/// enum Animal {
///     Cat(String),
///     Dog(String)
/// }
///
/// let animal = Animal::Cat("meow");
/// let sound = extract!(animal, Animal::Cat);
/// assert!(sound, "meow");
/// ```
///
#[macro_export]
macro_rules! extract {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            // #1
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat)); // #2
        }
    }};
}
