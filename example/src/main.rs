use quikia::app::{run_app, SharedApp};
use quikia::Color;
use quikia::dpi::{LogicalSize, Size};
#[cfg(target_os = "android")]
use quikia::platform::android::activity::AndroidApp;
use quikia::theme::material_theme;
use quikia::ui::Item;
use quikia::widget::RectangleExt;
use quikia::window::WindowBuilder;

#[cfg(not(target_os = "android"))]
fn main() {
    let window_builder = WindowBuilder::new()
        .with_title("Hello, world!")
        .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)))
        .with_min_inner_size(Size::Logical(LogicalSize::new(400.0, 300.0)));
    run_app(window_builder, material_theme(Color::BLUE, true), main_ui);
}

fn main_ui(app: SharedApp) -> Item {
    app.rectangle()
        .color(Color::RED)
        .item()
}

/*#[cfg(target_os = "android")]
use quikia::platform::android::activity::AndroidApp;*/

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    let window_builder = WindowBuilder::new()
        .with_title("Hello, world!")
        .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));
    run_app(app, window_builder, Box::new(MainPage::new()));
}

#[cfg(target_os = "android")]
fn main() {}

