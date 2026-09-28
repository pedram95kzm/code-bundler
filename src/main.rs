#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod compression;
mod embedder;
mod extractor;
mod output;
mod paths;
mod workflow;

use iced::{Font, Size, Theme, window};

const VAZIRMATN_REGULAR: &[u8] = include_bytes!("../assets/fonts/Vazirmatn-Regular.ttf");
const VAZIRMATN_BOLD: &[u8] = include_bytes!("../assets/fonts/Vazirmatn-Bold.ttf");

fn main() -> iced::Result {
    iced::application(app::CodeBundlerApp::default, app::update, app::view)
        .title("Code Bundler")
        .theme(Theme::Dark)
        .font(VAZIRMATN_REGULAR)
        .font(VAZIRMATN_BOLD)
        .default_font(Font::with_name("Vazirmatn"))
        .window(window::Settings {
            size: Size::new(760.0, 760.0),
            min_size: Some(Size::new(580.0, 600.0)),
            position: window::Position::Centered,
            icon: application_icon(),
            ..window::Settings::default()
        })
        .run()
}

fn application_icon() -> Option<window::Icon> {
    let image = image::load_from_memory(include_bytes!("../assets/code-bundler-icon.png"))
        .ok()?
        .into_rgba8();
    let (width, height) = image.dimensions();
    window::icon::from_rgba(image.into_raw(), width, height).ok()
}
