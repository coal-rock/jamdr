use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
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
    // it makes more sense to store page dimensions in millimeters,
    // we however store the position in points, as it makes line height and certain
    // formatting calculations easier
    page_position: (Pt, Pt),
    temp_position: (Pt, Pt),
    // list depth: if entry is none, list is bulleted, if entry is some, list is numbered
    list_depth: Vec<Option<u64>>,
    document: PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerReference,
    font: Font,
    style: Style,
}

pub struct Style {
    width: Mm,
    height: Mm,
    vertical_padding: Mm,
    horizontal_padding: Mm,
    underline_headings: HeaderUnderline,
}

pub enum HeaderUnderline {
    FullPage,
    TextOnly,
    None,
}

impl<'a> Inhouse<'a> {
    fn new(markdown: &'a str, title: String) -> Inhouse<'a> {
        let width = Mm(209.9);
        let height = Mm(297.0);

        let (doc, page1, layer1) = PdfDocument::new(title, width, height, "Layer 1");

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
            vertical_padding: Mm(14.0),
            horizontal_padding: Mm(14.0),
            underline_headings: HeaderUnderline::FullPage,
        };

        let current_layer = doc.get_page(page1).get_layer(layer1);
        current_layer.begin_text_section();

        current_layer.set_font(&font.get(), font.regular_size);
        current_layer.set_text_cursor(style.horizontal_padding, height - style.vertical_padding);
        current_layer.set_line_height(font.line_height);
        current_layer.set_text_rendering_mode(TextRenderingMode::Fill);

        Inhouse {
            markdown: pulldown_cmark::Parser::new_ext(&markdown, Options::ENABLE_STRIKETHROUGH)
                .collect(),
            position: 0,
            page_position: (
                style.horizontal_padding.into_pt(),
                (height - style.vertical_padding).into_pt(),
            ),
            temp_position: (Pt(0.0), Pt(0.0)),
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
        match self.peek() {
            Event::Start(_) => self.handle_start(),
            Event::End(_) => self.handle_end(),
            Event::Text(_) => self.render_text(),
            Event::Code(_) => todo!(),
            Event::Html(_) => todo!(),
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak => {
                self.line_break();
                self.consume();
            }
            Event::HardBreak => {
                self.line_break();
                self.consume();
            }
            Event::Rule => {
                self.horizontal_rule();
                self.consume();
            }
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
                self.font.is_bold = true;
                self.font.current_size = self.font.header_size
                    - (self.font.header_size_scale_increment
                        * match heading_level {
                            HeadingLevel::H1 => 1.0,
                            HeadingLevel::H2 => 2.0,
                            HeadingLevel::H3 => 3.0,
                            HeadingLevel::H4 => 4.0,
                            HeadingLevel::H5 => 5.0,
                            HeadingLevel::H6 => 6.0,
                        });

                self.render();

                match self.style.underline_headings {
                    HeaderUnderline::FullPage => self.draw_line(
                        self.style.horizontal_padding.into_pt(),
                        (self.style.width - self.style.horizontal_padding).into_pt(),
                        LineLocation::Underline,
                    ),
                    HeaderUnderline::TextOnly => self.draw_line(
                        self.style.horizontal_padding.into_pt(),
                        self.page_position.0,
                        LineLocation::Underline,
                    ),
                    HeaderUnderline::None => {}
                };
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

                self.page_position.0 += self.calc_text_width(text.to_string()).into_pt();
            }
            Tag::FootnoteDefinition(_) => todo!(),
            Tag::Table(_) => todo!(),
            Tag::TableHead => todo!(),
            Tag::TableRow => todo!(),
            Tag::TableCell => todo!(),
            Tag::Emphasis => self.font.is_italic = true,
            Tag::Strong => self.font.is_bold = true,
            Tag::Strikethrough => self.font.is_strikethrough = true,
            Tag::Link(_, _, _) => {
                self.temp_position = self.page_position;
            }
            Tag::Image(_, _, _) => todo!(),
        }
    }

    fn handle_end(&mut self) {
        let tag = extract!(self.consume().clone(), Event::End);

        match tag {
            Tag::Paragraph => self.line_break(),
            Tag::Heading(_, _, _) => {
                self.line_break();
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
            Tag::Link(link_type, dest, _) => {
                self.layer.add_link_annotation(LinkAnnotation::new(
                    printpdf::Rect::new(Mm(10.0), Mm(200.0), Mm(100.0), Mm(212.0)),
                    Some(printpdf::BorderArray::default()),
                    Some(printpdf::ColorArray::default()),
                    printpdf::Actions::uri("aslkdj".to_string()),
                    Some(printpdf::HighlightingMode::Invert),
                ));
            }
            Tag::Image(_, _, _) => todo!(),
        }
    }

    fn render_text(&mut self) {
        let text = extract!(self.peek(), Event::Text);

        self.layer
            .set_font(&self.font.get(), self.font.current_size);
        self.layer.write_text(text.to_string(), &self.font.get());

        let x_before = self.page_position.0;

        self.page_position.0 += self.calc_text_width(text.to_string()).into();

        if self.font.is_strikethrough {
            self.draw_line(x_before, self.page_position.0, LineLocation::Strikethrough);
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

    fn draw_line(&self, start: Pt, end: Pt, location: LineLocation) {
        let offset = match location {
            LineLocation::Underline => Pt(-self.font.line_height * 0.25),
            LineLocation::Strikethrough => Pt(self.font.line_height * 0.20),
        };

        let line = Line {
            points: vec![
                (
                    Point {
                        x: start,
                        y: self.page_position.1 + offset,
                    },
                    true,
                ),
                (
                    Point {
                        x: end,
                        y: self.page_position.1 + offset,
                    },
                    true,
                ),
            ],
            is_closed: true,
        };

        self.layer.add_line(line);
    }

    fn line_break(&mut self) {
        self.layer.add_line_break();
        self.page_position.1 -= Pt(self.font.line_height);
        self.page_position.0 = self.style.horizontal_padding.into_pt();
        self.reset_formatting();
    }

    fn horizontal_rule(&mut self) {
        self.draw_line(
            self.style.horizontal_padding.into_pt(),
            (self.style.width - self.style.horizontal_padding).into_pt(),
            LineLocation::Strikethrough,
        );
        self.line_break();
    }

    // resets formatting, AND applies changes made to underlying objects
    fn reset_formatting(&mut self) {
        self.font.clear_typography();
        self.layer.set_line_height(self.font.line_height);
        self.layer
            .set_font(&self.font.get(), self.font.current_size);
    }

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

enum LineLocation {
    Underline,
    Strikethrough,
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

// not-so thin wrapper around IndirectFontRef
// manages typopgraphic state
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
    line_height: f32,
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
            current_size: 12.0,
            regular_size: 12.0,
            header_size: 20.0,
            header_size_scale_increment: 4.0,
            line_height: 20.0,
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

    // reset internal formatting
    // does **NOT** apply to layer immediately
    fn clear_typography(&mut self) {
        self.is_bold = false;
        self.is_italic = false;
        self.is_strikethrough = false;
        self.current_size = self.regular_size;
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
