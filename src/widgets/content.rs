use iced_native::Color;

pub enum Content {
    Heading(String),
    Subheading(String),
    Filename(String),
    Paragraph(String, Color),
    Hunk(Vec<(String, Color)>),
}

impl Content {
    pub fn to_md_string(&self) -> String {
        match self {
            Content::Heading(text) => format!("# {}\n", text),
            Content::Subheading(text) => format!("## {}\n", text),
            Content::Filename(filename) => format!("File: `{}`\n", filename),
            Content::Paragraph(line, _) => line.clone(),
            Content::Hunk(arr) => format!("```diff\n{}```\n", arr.iter().map(|(line, color)| {
                line.clone()
            }).collect::<String>())
        }
    }
}