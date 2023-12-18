use crate::utils::{AppState, Action};
use druid::{Cursor, Rect, Widget, Code, TextLayout, ImageBuf, Affine, FontDescriptor, FontFamily};
use druid::RenderContext;
use druid::{Env, Color};
use druid::{Data, Lens};
use druid::kurbo::{Circle, Line, Point, Vec2, Ellipse};
use druid::piet::{ImageFormat, InterpolationMode, StrokeStyle};
use druid::Event;
use image::{GenericImageView, DynamicImage};
use num_traits::cast::FromPrimitive;
use druid::Screen as dScreen;
use screenshots::Screen;
use crate::utils;

fn calculate_arrowhead(start: Point, end: Point, arrowhead_length: f64, arrowhead_width: f64) -> (Point, Point) {
    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * arrowhead_width / 2.0;
    let arrowhead_base = end - direction * arrowhead_length;
    let left_point = arrowhead_base + perpendicular;
    let right_point = arrowhead_base - perpendicular;
    (left_point, right_point)
}

#[derive(Clone, Data, Lens)]
struct TextInputState {
    text: String,
}


pub struct DrawingWidget;

impl Widget<AppState> for DrawingWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut AppState, _env: &Env) {
        // Handle user input events for drawing here
        match event {
            Event::KeyDown(key) => {
                if data.is_writing_text {
                    if key.code.eq(&Code::Enter) {
                        data.is_writing_text = false;
                        return;
                    }
                    if let Some(action) = data.actions.last_mut() {
                        if let Action::Text(affine, _, string, _, _) = action {
                            *affine = data.affine.clone();
                            if key.code.eq(&Code::Backspace) {
                                if !string.is_empty() {
                                    string.pop();
                                }
                            } else if key.code.eq(&Code::ShiftLeft) ||
                                key.code.eq(&Code::ShiftRight) ||
                                key.code.eq(&Code::Tab) ||
                                key.code.eq(&Code::MetaLeft) ||
                                key.code.eq(&Code::MetaRight) ||
                                key.code.eq(&Code::AltLeft) ||
                                key.code.eq(&Code::AltRight) {} else {
                                let char = key.key.to_string();
                                string.push(char.chars().next().unwrap());
                            }
                        }
                    }
                }
            }
            Event::Paste(clipboard) => {
                if data.is_writing_text {
                    if let Some(action) = data.actions.last_mut() {
                        if let Action::Text(affine, _, string, _, _) = action {
                            *affine = data.affine.clone();
                            *string = clipboard.get_string().unwrap();
                        }
                    }
                }
            }
            Event::MouseDown(e) => {
                if data.is_picking_color {
                    ctx.set_cursor(&Cursor::Pointer);
                    return;
                }
                data.is_drawing = true;
                let mut action = Action::new(&data.selection);
                ctx.set_cursor(&Cursor::Crosshair);
                match action {
                    Action::Pen(ref mut affine, ref mut points, ref mut color, ref mut stroke) => {
                        points.push(e.pos);
                        *color = data.color;
                        *stroke = data.stroke;
                        *affine = data.affine.clone();
                    }
                    Action::Highlighter(ref mut affine, ref mut points, ref mut color, ref mut stroke) => {
                        points.push(e.pos);
                        *color = data.color;
                        *stroke = data.stroke;
                        *affine = data.affine.clone();
                    }
                    Action::Rectangle(ref mut affine, ref mut start_point, ref mut end_point, ref mut color, ref mut fill, ref mut stroke) => {
                        *start_point = e.pos;
                        *end_point = e.pos;
                        *fill = data.fill_color;
                        *color = data.color;
                        *stroke = data.stroke;
                        *affine = data.affine.clone();
                    }
                    Action::Circle(ref mut affine, ref mut center, _, ref mut color, ref mut fill, ref mut stroke) => {
                        *center = e.pos;
                        *fill = data.fill_color;
                        *color = data.color;
                        *stroke = data.stroke;
                        *affine = data.affine.clone();
                    }
                    Action::Ellipse(ref mut affine, ref mut start_point, ref mut end_point, ref mut color, ref mut fill, ref mut stroke) => {
                        *start_point = e.pos;
                        *end_point = e.pos;
                        *fill = data.fill_color;
                        *color = data.color;
                        *stroke = data.stroke;
                        *affine = data.affine.clone();
                    }
                    Action::Arrow(ref mut affine, ref mut start_point, ref mut end_point, ref mut color, ref mut stroke) => {
                        *start_point = e.pos;
                        *end_point = e.pos;
                        *color = data.color;
                        *stroke = data.stroke;
                        *affine = data.affine.clone();
                    }
                    Action::Text(ref mut affine, ref mut position, _, ref mut color, ref mut font_size) => {
                        if data.is_writing_text { return; }
                        ctx.request_focus();
                        *position = e.pos;
                        *color = data.color;
                        *affine = data.affine.clone();
                        *font_size = data.font_size;
                        // Set a flag or state indicating that text input is needed
                        data.is_writing_text = true;
                    }
                    Action::Crop(ref mut prev_image, ref mut start_point, ref mut end_point) => {

                        let x = ctx.window().get_position().x.floor();
                        let y = ctx.window().get_position().y.floor() + data.title_bar_height;

                        let width = ctx.size().width.ceil();
                        let height = ctx.size().height.ceil();

                        #[cfg(not(target_os = "macos"))]
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        ctx.set_active(true);
                        *prev_image = capture_image_area(Rect::new(x, y, width, height));
                        ctx.request_layout();
                        *start_point = e.pos;
                        *end_point = e.pos;
                    }
                }
                data.actions.push(action);
                ctx.request_paint();
            }
            Event::MouseMove(e) => {
                if data.is_picking_color {
                    ctx.set_cursor(&Cursor::Pointer);
                    return;
                }
                ctx.set_cursor(&Cursor::Crosshair);
                if data.is_drawing {
                    if let Some(action) = data.actions.last_mut() {
                        match action {
                            Action::Pen(_, points, _, _) => { points.push(e.pos); }
                            Action::Highlighter(_, points, _, _) => { points.push(e.pos); }
                            Action::Rectangle(_, _, end_point, _, _, _) => {
                                *end_point = e.pos;
                            }
                            Action::Circle(_, center, radius, _, _, _) => {
                                *radius = f64::sqrt(num_traits::pow(center.x - e.pos.x, 2) + num_traits::pow(center.y - e.pos.y, 2));
                            }
                            Action::Ellipse(_, _, end_point, _, _, _) => {
                                *end_point = e.pos;
                            }
                            Action::Arrow(_, _, end_point, _, _) => {
                                *end_point = e.pos;
                            }
                            Action::Crop(_, _, end_point) => {
                                *end_point = e.pos;
                            }
                            _ => {}
                        }
                    }
                    ctx.request_paint();
                }
            }
            Event::MouseUp(e) => {
                if data.is_picking_color {
                    let img = image::open(data.image_path.to_string()).unwrap();
                    let x = (img.width() * u32::from_f64(e.pos.x).unwrap()) / u32::from_f64(ctx.size().width).unwrap();
                    let y = (img.height() * u32::from_f64(e.pos.y).unwrap()) / u32::from_f64(ctx.size().height).unwrap();
                    let pixel = img.get_pixel(x, y);
                    data.color = Color::rgba8(pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);
                    ctx.set_cursor(&Cursor::Arrow);
                    data.custom_color = true;
                    data.is_picking_color = false;
                    return;
                }
                if let Some(Action::Rectangle(_, _, end_point, _, _, _)) = data.actions.last_mut() {
                    *end_point = e.pos;
                }
                if let Some(Action::Circle(_, center, radius, _, _, _)) = data.actions.last_mut() {
                    *radius = f64::sqrt(num_traits::pow(center.x - e.pos.x, 2) + num_traits::pow(center.y - e.pos.y, 2));
                }
                if let Some(Action::Ellipse(_, _, end_point, _, _, _)) = data.actions.last_mut() {
                    *end_point = e.pos;
                }
                if let Some(Action::Arrow(_, _, _, _, _)) = data.actions.last_mut() {}
                if let Some(Action::Text(_, position, _, color, _)) = data.actions.last_mut() {
                    if data.is_writing_text { return; }
                    ctx.request_focus();
                    *position = e.pos;
                    *color = data.color;
                    // Set a flag or state indicating that text input is needed
                    data.is_writing_text = true;
                }
                if let Some(Action::Crop(prev_image, start_point, end_point)) = data.actions.last_mut() {
                    *end_point = e.pos;
                    let mut x = start_point.x;
                    let mut y = start_point.y;
                    let mut width = end_point.x - x;
                    let mut height = end_point.y - y ;

                    x = ((x * prev_image.width() as f64) / ctx.size().width).floor();
                    y = ((y * prev_image.height() as f64) / ctx.size().height).floor();
                    width = ((width * prev_image.width() as f64) / ctx.size().width).ceil();
                    height = ((height * prev_image.height() as f64) / ctx.size().height).ceil();

                    if data.extension.eq("png") || data.extension.eq("tiff") || data.extension.eq("bmp") {
                        data.image = ImageBuf::from_dynamic_image(prev_image.crop(x as u32, y as u32, width as u32, height as u32));
                    } else {
                        data.image = ImageBuf::from_dynamic_image_without_alpha(prev_image.crop(x as u32, y as u32, width as u32, height as u32));
                    }
                    data.actions.clear();
                    data.redo_actions.clear();
                    data.crop.set(false);
                    data.selection = utils::Selection::Pen;
                }
                data.is_drawing = false;
                data.update.set(true);
                ctx.set_cursor(&Cursor::Arrow);
                data.repaint = true;
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &AppState, _env: &Env) {}

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &AppState, data: &AppState, _env: &Env) {
        ctx.request_layout();
        if data.repaint {
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, _bc: &druid::BoxConstraints, data: &AppState, _env: &Env) -> druid::Size {
        let monitor = dScreen::get_monitors().first().unwrap().clone();
        let monitor_width = monitor.virtual_work_rect().width();
        let monitor_height = monitor.virtual_work_rect().height();


        let image_width = data.image.width() as f64;
        let image_height = data.image.height() as f64;

        let window_width;
        let window_height;
        if image_width > image_height {
            data.scale_factor.set(image_width / monitor_width + 0.5f64);
            window_width = image_width / data.scale_factor.get();
            window_height = (image_height * window_width) / image_width;
        } else {
            data.scale_factor.set(image_height / monitor_height + 0.5f64);
            window_height = image_height / data.scale_factor.get();
            window_width = (image_width * window_height) / image_height;
        }

        ctx.window().set_size((window_width, window_height + data.title_bar_height));
        druid::Size::new(window_width, window_height)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &Env) {
        let width;
        let height;

        width = ctx.size().width;
        height = ctx.size().height;
        data.center.set(Point::new(width / 2f64, height / 2f64));


        ctx.with_save(|ctx| {
            for a in &data.affine {
                ctx.render_ctx.transform(*a);
                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
            }
            let image;
            if data.extension.eq("png") || data.extension.eq("tiff") || data.extension.eq("bmp") {
                image = ctx.render_ctx.make_image(data.image.width(), data.image.height(), data.image.raw_pixels(), ImageFormat::RgbaSeparate).unwrap();
            } else {
                image = ctx.render_ctx.make_image(data.image.width(), data.image.height(), data.image.raw_pixels(), ImageFormat::Rgb).unwrap();
            }
            ctx.render_ctx.draw_image(&image, Rect::new(0f64, 0f64, width, height), InterpolationMode::Bilinear);
        });

        for action in &data.actions {
            match action {
                Action::Highlighter(affine, action, color, stroke) => {
                    if action.len() < 2 {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.fill(Circle::new(*action.last().unwrap(), stroke * 2f64), &color.with_alpha(0.25));
                        });
                    }
                    for pair in action.windows(2) {
                        if let [start, end] = pair {
                            ctx.with_save(|ctx| {
                                for a in &data.affine {
                                    ctx.render_ctx.transform(*a);
                                    if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                    if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                                }
                                for a in affine {
                                    ctx.render_ctx.transform(*a);
                                    if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                    if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                                }
                                let line = Line::new(*start, *end);
                                ctx.render_ctx.stroke(line, &color.with_alpha(0.25), stroke * 3f64)
                            });
                        }
                    }
                }
                Action::Pen(affine, action, color, stroke) => {
                    if action.len() < 2 {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.fill(Circle::new(*action.last().unwrap(), stroke / 2f64), color);
                        });
                    }
                    for pair in action.windows(2) {
                        if let [start, end] = pair {
                            ctx.with_save(|ctx| {
                                for a in &data.affine {
                                    ctx.render_ctx.transform(*a);
                                    if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                    if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                                }
                                for a in affine {
                                    ctx.render_ctx.transform(*a);
                                    if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                    if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                                }
                                let line = Line::new(*start, *end);
                                ctx.render_ctx.stroke(line, color, *stroke);
                            });
                        }
                    }
                }
                Action::Rectangle(affine, start_point, end_point, color, fill, stroke) => {
                    if *fill {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.fill_even_odd(Rect::new(start_point.x, start_point.y, end_point.x, end_point.y), color);
                        });
                    } else {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.stroke(Rect::new(start_point.x, start_point.y, end_point.x, end_point.y), color, *stroke);
                        });
                    }
                }
                Action::Circle(affine, center, radius, color, fill, stroke) => {
                    if *fill {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.fill_even_odd(Circle::new(*center, *radius), color);
                        });
                    } else {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.stroke(Circle::new(*center, *radius), color, *stroke);
                        });
                    }
                }
                Action::Ellipse(affine, start_point, end_point, color, fill, stroke) => {
                    if *fill {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.fill_even_odd(Ellipse::from_rect(Rect::new(start_point.x, start_point.y, end_point.x, end_point.y)), color);
                        });
                    } else {
                        ctx.with_save(|ctx| {
                            for a in &data.affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            for a in affine {
                                ctx.render_ctx.transform(*a);
                                if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                                if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                            }
                            ctx.render_ctx.stroke(Ellipse::from_rect(Rect::new(start_point.x, start_point.y, end_point.x, end_point.y)), color, *stroke);
                        });
                    }
                }
                Action::Arrow(affine, start_point, end_point, color, stroke) => {
                    ctx.with_save(|ctx| {
                        for a in &data.affine {
                            ctx.render_ctx.transform(*a);
                            if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                            if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                        }
                        for a in affine {
                            ctx.render_ctx.transform(*a);
                            if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                            if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                        }
                        // Draw the line
                        let line = Line::new(*start_point, *end_point);
                        let len = line.length();
                        ctx.render_ctx.stroke(line, color, *stroke);
                        // Calculate the arrowhead points
                        let arrowhead_length = len / 10f64;
                        let arrowhead_width = len * 5f64 / 100f64;
                        let (left_point, right_point) = calculate_arrowhead(*start_point, *end_point, arrowhead_length, arrowhead_width);
                        // Draw the arrowhead
                        let arrowhead = Line::new(left_point, *end_point);
                        ctx.render_ctx.stroke(arrowhead, color, *stroke);
                        let arrowhead = Line::new(right_point, *end_point);
                        ctx.render_ctx.stroke(arrowhead, color, *stroke);
                    });
                }
                Action::Text(affine, pos, text, color, font_size) => {
                    ctx.with_save(|ctx| {
                        for a in &data.affine {
                            ctx.render_ctx.transform(*a);
                            if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                            if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                        }
                        for a in affine {
                            ctx.render_ctx.transform(*a);
                            if a == &Affine::FLIP_Y { ctx.render_ctx.transform(Affine::translate((0.0, -height))); }
                            if a == &Affine::FLIP_X { ctx.render_ctx.transform(Affine::translate((-width, 0.0))); }
                        }
                        let mut layout = TextLayout::<String>::from_text(text.to_string());
                        layout.set_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(*font_size));
                        layout.set_text_color(*color);
                        layout.rebuild_if_needed(ctx.text(), env);
                        layout.draw(ctx, *pos);
                    });
                }
                Action::Crop(_, start_point, end_point) => {
                    if data.crop.get() {
                        let background_color = Color::rgba(1.0, 1.0, 1.0, 0.05);
                        ctx.fill(Rect::from_points(*start_point, *end_point), &background_color);

                        // Set the border color
                        let border_color = Color::GRAY;

                        // Draw the border
                        let border_width = 1.0;
                        let border_rect = Rect::from_points(*start_point, *end_point).inset(-border_width / 2.0);
                        let stroke_style = StrokeStyle::new().dash_pattern(&[2.0]);
                        ctx.stroke_styled(border_rect, &border_color, border_width, &stroke_style);
                    }
                }
            }
        }

        if data.save.get() {
            let x = ctx.window().get_position().x;
            let y = ctx.window().get_position().y + data.title_bar_height;
            let width = ctx.size().width;
            let height = ctx.size().height;
            #[cfg(not(target_os = "macos"))]
            std::thread::sleep(std::time::Duration::from_millis(300));
            let image = capture_image_area(Rect::new(x, y, width, height));
            image.save(data.image_path.to_string()).unwrap();
            data.save.set(false);
        }
    }
}

fn capture_image_area(rect: Rect) -> DynamicImage {
    let screens = Screen::all().unwrap();
    let screen = screens.iter().map(|screen| { (screen, num_traits::abs(rect.x0.floor() as i32 - screen.display_info.x)) }).min_by_key(|screen| { screen.1 }).unwrap().0;
    return DynamicImage::ImageRgba8(screen.capture_area(num_traits::abs(rect.x0.floor() as i32 - screen.display_info.x), num_traits::abs(rect.y0.floor() as i32 - screen.display_info.y), rect.x1.ceil() as u32, rect.y1.ceil() as u32).unwrap());
}
