use std::cell::Cell;
use std::path::Path;
use druid::{Affine, Color, ImageBuf, Monitor, Point};
use druid::{Data, Lens};
use clap::Parser;
use image::DynamicImage;
#[cfg(target_os="windows")]
use winapi::um::winuser::GetSystemMetrics;
#[cfg(target_os="windows")]
use winapi::um::winuser::SM_CYCAPTION;

/// Annotation Tools
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    pub path: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Selection {
    Pen,
    Highlighter,
    Rectangle,
    Circle,
    Ellipse,
    Arrow,
    Text,
    Crop,
}

impl Default for Selection {
    fn default() -> Self {
        return Self::Pen
    }
}
#[derive(PartialEq, Debug, Clone)]
pub enum Action {
    Pen(Vec<Affine>, Vec<Point>, Color, f64),
    Highlighter(Vec<Affine>, Vec<Point>, Color, f64),
    Rectangle(Vec<Affine>, Point, Point, Color, bool, f64), // Stores rectangle points and color
    Circle(Vec<Affine>, Point, f64, Color, bool, f64), // Stores circle points and color
    Ellipse(Vec<Affine>, Point, Point, Color, bool, f64), // Stores ellipse points and color
    Arrow(Vec<Affine>, Point, Point, Color, f64), // Stores arrow points and color
    Text(Vec<Affine>, Point, String, Color, f64),  // Stores position, text, and color
    Crop(DynamicImage, Point, Point),
}

impl Action {
    pub fn new(selection: &Selection) -> Self {
        match selection {
            Selection::Pen => Self::Pen(Vec::<Affine>::new(), Vec::new(), Color::RED, 2.0),
            Selection::Highlighter => Self::Highlighter(Vec::<Affine>::new(),Vec::new(), Color::RED, 2.0),
            Selection::Rectangle => Self::Rectangle(Vec::<Affine>::new(),Point::ZERO, Point::ZERO, Color::RED, false, 2.0),
            Selection::Circle => Self::Circle(Vec::<Affine>::new(),Point::ZERO, 0.0, Color::RED, false,2.0),
            Selection::Ellipse => Self::Ellipse(Vec::<Affine>::new(),Point::ZERO, Point::ZERO, Color::RED, false, 2.0),
            Selection::Arrow => Self::Arrow(Vec::<Affine>::new(),Point::ZERO, Point::ZERO, Color::RED, 2.0),
            Selection::Text => Self::Text(Vec::<Affine>::new(),Point::ZERO, String::from("") ,Color::RED, 24f64),
            Selection::Crop => Self::Crop(DynamicImage::default(), Point::ZERO, Point::ZERO),
        }
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct AppState {
    #[data(same_fn = "PartialEq::eq")]
    pub center: Cell<Point>,
    #[data(same_fn = "PartialEq::eq")]
    pub affine: Vec<Affine>,
    #[data(same_fn = "PartialEq::eq")]
    pub scale_factor: Cell<f64>,
    #[data(same_fn = "PartialEq::eq")]
    pub selection: Selection,
    pub image: ImageBuf,
    pub extension: String,
    #[data(same_fn = "PartialEq::eq")]
    pub actions: Vec<Action>,
    #[data(same_fn = "PartialEq::eq")]
    pub redo_actions: Vec<Action>,
    pub is_drawing: bool,
    pub image_path: String,
    #[data(same_fn = "PartialEq::eq")]
    pub monitor: Monitor,
    pub color: Color,
    pub repaint: bool,
    pub is_picking_color: bool,
    pub custom_color: bool,
    pub fill_color: bool,
    pub stroke: f64,
    pub is_writing_text: bool,
    #[data(same_fn = "PartialEq::eq")]
    pub save: Cell<bool>,
    #[data(same_fn = "PartialEq::eq")]
    pub update: Cell<bool>,
    pub zoom: f64,
    #[data(same_fn = "PartialEq::eq")]
    pub crop: Cell<bool>,
    pub font_size: f64,
    pub title_bar_height: f64,
}

impl AppState {
    pub fn new(image: DynamicImage, height: f64, extension: String, scale: f64, image_path: String, monitor: Monitor, color: Color) -> Self {

        AppState {
            title_bar_height: height,
            extension,
            center: Cell::new(Point::ORIGIN),
            scale_factor: Cell::new(scale),
            affine: Vec::<Affine>::new(),
            selection: Selection::default(),
            image: ImageBuf::from_dynamic_image(image),
            actions: Vec::<Action>::new(),
            redo_actions: Vec::<Action>::new(),
            is_drawing: false,
            image_path,
            monitor,
            color,
            repaint: false,
            is_picking_color: false,
            custom_color: false,
            fill_color: false,
            stroke: 2.0,
            is_writing_text: false,
            update: Cell::new(false),
            zoom: 1f64,
            save: Cell::new(false),
            crop: Cell::new(false),
            font_size: 24f64,
        }
    }
}

pub fn dialog_file_not_found(path: String) {
    tauri_dialog::DialogBuilder::new()
        .title("File Not Found!")
        .message(&format!("No such file \"{}\".\nPlease check that the file exists and try again.", Path::new(path.as_str()).file_name().unwrap().to_str().unwrap()))
        .style(tauri_dialog::DialogStyle::Error)
        .buttons(tauri_dialog::DialogButtons::Quit)
        .build()
        .show();
}

pub fn dialog_not_supported(path: String) {
    tauri_dialog::DialogBuilder::new()
        .title("File Not Supported!")
        .message(&format!("The file \"{}\" has an unsupported file format. Please try again with an image format.", Path::new(path.as_str()).file_name().unwrap().to_str().unwrap()))
        .style(tauri_dialog::DialogStyle::Error)
        .buttons(tauri_dialog::DialogButtons::Quit)
        .build()
        .show();
}


