use std::fs;

use serde::Deserialize;
use toml::value::Array;

#[derive(Debug, Default, Deserialize)]
struct Intro {
    content: Array,
}

#[derive(Debug, Default, Deserialize)]
struct Hunk {
    content: Array,
}

#[derive(Debug, Default, Deserialize)]
struct Outro {
    content: Array,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    intro: Intro,
    hunk: Hunk,
    outro: Outro,
}

impl Config {
    pub fn from_path(path: &str) -> Self {
        let toml_file = match fs::read_to_string(path) {
            Err(_) => {
                println!("Could not read TOML file");
                String::new()
            },
            Ok(s) => s,
        };

        toml::from_str(&toml_file).unwrap()
    }

    pub fn get_intro_content(&self) -> Array {
        self.intro.content.clone()
    }

    pub fn get_hunk_content(&self) -> Array {
        self.hunk.content.clone()
    }

    pub fn get_outro_content(&self) -> Array {
        self.outro.content.clone()
    }
}