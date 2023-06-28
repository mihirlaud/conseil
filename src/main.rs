mod app;
mod widgets;

use iced::{Application, Settings};

use crate::app::app::ConseilApp;

pub fn main() -> iced::Result {
    ConseilApp::run(Settings::default())
}
