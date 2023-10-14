use quikia::app::create_window;
use quikia::dpi::{LogicalSize, Size};
use quikia::window::WindowBuilder;

fn main() {
    let window_builder = WindowBuilder::new()
        .with_title("Hello, world!")
        .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));
    create_window(window_builder);
}
