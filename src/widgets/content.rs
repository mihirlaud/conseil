use iced_native::Color;

#[derive(Clone)]
pub enum Content {
    Heading(String),
    Subheading(usize, String),
    Filename(String),
    Paragraph(usize, String),
    Hunk(Vec<(String, Color)>),
}

impl Content {
    pub fn to_md_string(&self) -> String {
        match self {
            Content::Heading(text) => format!("# {}\n", text),
            Content::Subheading(_, text) => format!("## {}\n", text),
            Content::Filename(filename) => format!("File: `{}`\n", filename),
            Content::Paragraph(_, text) => format!("{}\n", text),
            Content::Hunk(arr) => format!("```diff\n{}```\n", arr.iter().map(|(line, _color)| {
                line.clone()
            }).collect::<String>())
        }
    }
}