mod app;
mod widgets;

use iced::{Application, Settings};

use crate::app::app::ConseilApp;

pub fn main() -> iced::Result {
    println!("{:?}", app::config::Config::from_path("configs/default.toml"));
    ConseilApp::run(Settings::default())
}
