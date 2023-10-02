// DEPRECATED
struct MarkdownBlock {
    block_type: BlockType,
    text_content: Option<String>,
    text_style: Option<Vec<TextStyle>>,
}

enum BlockType {
    Heading(HeadingLevel),
    Paragraph,
    LineBreak,
    Blockquote,
    List,
    Table,
    CodeBlock,
    Image,
    HorizontalRule,
    Link,
}

enum TextStyle {
    Bold,
    Italic,
    Underlined,
}

enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

fn parse_markdown(markdown: &str) -> Vec<MarkdownBlock> {
    let mut parsed_markdown = Vec::new();

    let md_chars = markdown.chars().collect::<Vec<char>>();
    let mut position = 0;

    parsed_markdown
}
