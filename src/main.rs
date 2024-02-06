mod main_page;

use std::ptr;
use quikia::app::create_window;
use quikia::Color;
use quikia::dpi::{LogicalSize, Size};
#[cfg(target_os = "android")]
use quikia::platform::android::activity::AndroidApp;
use quikia::theme::material_theme;
use quikia::window::WindowBuilder;
use crate::main_page::MainPage;

#[cfg(not(target_os = "android"))]
fn main() {
    let window_builder = WindowBuilder::new()
        .with_title("Hello, world!")
        .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));
    create_window(window_builder,material_theme(Color::GREEN, true), Box::new(MainPage::new()));
}

/*#[cfg(target_os = "android")]
use quikia::platform::android::activity::AndroidApp;*/

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    let window_builder = WindowBuilder::new()
        .with_title("Hello, world!")
        .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));
    create_window(app,window_builder, Box::new(MainPage::new()));
}

#[cfg(target_os = "android")]
fn main() {}

