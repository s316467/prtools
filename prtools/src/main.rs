#![windows_subsystem = "windows"]
mod utils;
mod painter;
mod menu;

use std::fs;
use std::path::Path;
use std::process::{exit};
use druid::widget::{Align, Flex, Scroll};
use druid::{AppLauncher, Color, PlatformError, Screen, Widget, WidgetExt, WindowDesc};
use clap::Parser;
use crate::utils::{AppState};
use crate::painter::DrawingWidget;


fn ui_builder() -> impl Widget<AppState> {
    let drawing = Flex::row().with_child(DrawingWidget).padding(0.0);
    Align::centered(Scroll::new(drawing))
}

fn main() -> Result<(), PlatformError> {
    let arg = utils::Args::parse();

    //check if the file exists
    if let Err(_) = fs::metadata(arg.path.to_string()) {
        utils::dialog_file_not_found(arg.path.to_string());
        exit(255);
    }

    let extension = std::path::Path::new(arg.path.to_string().as_str()).extension().unwrap().to_os_string().into_string().unwrap().to_lowercase();

    if !extension.eq("png") && !extension.eq("jpeg") && !extension.eq("jpg") && !extension.eq("tiff") && !extension.eq("bmp") {
        utils::dialog_not_supported(arg.path.to_string());
        exit(254);
    }

    let monitor = Screen::get_monitors().first().unwrap().clone();
    let image = image::io::Reader::open(arg.path.to_string()).unwrap().with_guessed_format().unwrap().decode().unwrap();

    let monitor_width = monitor.virtual_work_rect().width();
    let monitor_height = monitor.virtual_rect().height();
    let image_width = image.width() as f64;

    let title_bar_height;
    #[cfg(target_os = "windows")] { title_bar_height = 11.11f64/100f64 * monitor_height;}
    #[cfg(target_os = "macos")] { title_bar_height = 3.3f64/100f64 * monitor_height; }
    #[cfg(target_os = "linux")] { title_bar_height = 11.11f64/100f64 * monitor_height; }

    let initial_state = AppState::new(
        image,
        title_bar_height,
        extension,
        1f64,
        arg.path.to_string(),
        monitor,
        Color::RED
    );

        initial_state.scale_factor.set(image_width / monitor_width + 0.5f64);

    let main_window = WindowDesc::new(ui_builder())
        .title(format!("Rust Screen Grabber Tools - [{}]", Path::new(arg.path.to_string().as_str()).canonicalize().unwrap().to_str().unwrap()))
        .menu(|_, _, _| {
            menu::create_menu()
        });

    AppLauncher::with_window(main_window)
        .log_to_console()
        .configure_env(move |env, _| {
            env.set(druid::theme::WINDOW_BACKGROUND_COLOR, Color::TRANSPARENT);
            env.set(druid::theme::BUTTON_DARK, Color::WHITE);
            env.set(druid::theme::SCROLLBAR_MAX_OPACITY, 0);
            env.set(druid::theme::BUTTON_LIGHT, Color::WHITE);
            env.set(druid::theme::TEXT_COLOR, Color::BLACK);
        })
        .launch(initial_state)
}
