#![windows_subsystem = "windows"]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::env;
use anyhow::Context;
use druid::{AppLauncher, LocalizedString, Scale, WindowDesc};
use druid::piet::{Color, RenderContext};
use druid::widget::Widget;
use druid::{Data, Env, EventCtx, Point, Rect, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size};


/// Data structure to store the start and end points of the rectangle.
#[derive(Clone, Data, Lens)]
pub struct AppData {
    start_point: Option<Point>,
    end_point: Option<Point>,
}

/// Widget to draw a rectangle on the screen.
pub struct DrawingOverlay;

impl DrawingOverlay {
    pub fn new() -> Self {
        DrawingOverlay {}
    }
}

impl Widget<AppData> for DrawingOverlay {
    /// Handles mouse events to draw the rectangle.
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            // When the mouse is pressed, store the start point.
            Event::MouseDown(mouse) => {
                data.start_point = Some(mouse.pos);
                ctx.request_paint();
            }
            // When the mouse is moved, update the end point.
            Event::MouseMove(mouse) => {
                data.end_point = Some(mouse.pos);
                ctx.request_paint();
            }
            // When the mouse is released, complete the drawing, save the coordinates and close the application.
            Event::MouseUp(_) => {
                if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
                    #[cfg(target_os = "macos")]
                    {
                        let x = start;
                        let y = end;
                        let rect = Rect::from_points(x, y).trunc();
                        let scale = ctx.scale();
                        save_point(rect, scale);
                    }
                    #[cfg(any(target_os = "windows", target_os = "linux"))]
                    {
                        let x = ctx.to_screen(start);
                        let y = ctx.to_screen(end);
                        let rect = Rect::from_points(x, y).trunc();
                        let scale = ctx.scale();
                        save_point(rect, scale);
                    }
                    ctx.submit_command(druid::commands::QUIT_APP);
                }
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {}

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &AppData,
        _data: &AppData,
        _env: &Env,
    ) {
       ctx.request_paint();
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppData, _env: &Env) {
        // Draw current selection outline, ensuring transparency inside.
        if let (Some(start), Some(end)) = (data.start_point, data.end_point) {

            let rect = Rect::from_points(start, end);
            let background_rect = ctx.size().to_rect();
            let surrounding_rects = surrounding_rectangles(background_rect, rect);

            for surrounding_rect in surrounding_rects {
                ctx.fill(surrounding_rect, &Color::rgba(0.0, 0.0, 0.0, 0.5));
            }
        }
        // If the rectangle is not drawn, fill the entire screen with a semi-transparent color.
        else {
            let background_rect = ctx.size().to_rect();
            ctx.fill(background_rect, &Color::rgba(0.0, 0.0, 0.0, 0.5));
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Errore: nessun argomento passato. Specifica un numero intero positivo >= 1.");
        return Ok(());
    }

    let index = args[1].parse::<usize>().context("Errore: l'argomento passato non è un numero intero positivo.")?;

    let (width, height, x, y) = compute_window_size(index)?;

    // Create the main window
    let main_window = WindowDesc::new(DrawingOverlay::new())
        .title(LocalizedString::new("Draw Shapes"))
        .set_always_on_top(true)
        .transparent(true)
        .show_titlebar(false)
        .window_size(Size::new(width, height))
        .set_position((x, y))
        .resizable(false);

    let initial_data = AppData {
        start_point: None,
        end_point: None,
    };

    // Launch the application with the main window
    AppLauncher::with_window(main_window)
        .launch(initial_data)
        .expect("Failed to launch application");

    Ok(())
}

/// Computes the window size based on the index of the monitor to display the application on.
/// Returns the width, height, top-left x-coordinate, and top-left y-coordinate of the window.
/// The index is 1-based, where 1 corresponds to the primary monitor.
/// The index is the first command-line argument passed to the application.
pub fn compute_window_size(index: usize) -> anyhow::Result<(f64, f64, f64, f64)> {
    let screens = druid::Screen::get_monitors();
    let width = screens.to_vec()[index-1].virtual_rect().width();
    let height = screens.to_vec()[index-1].virtual_rect().height();
    let top_x = screens.to_vec()[index-1].virtual_rect().x0;
    let top_y = screens.to_vec()[index-1].virtual_work_rect().y0;
    Ok((width, height, top_x, top_y))
}

/// Save the coordinates of the rectangle to a file.
/// The coordinates are saved in the format: x0,y0,width,height.
/// The coordinates are scaled based on the screen DPI.
/// The file is saved in the config directory of the project.
fn save_point(rect: Rect, scale: Scale) {
    let x = rect.x0 * scale.x();
    let y = rect.y0 * scale.y();
    let width = rect.width() * scale.x();
    let height = rect.height() * scale.y();
    let data = format!("{},{},{},{}", x, y, width, height);
    let mut path = get_project_src_path();
    path.push("config/crop.txt");
    let mut file = fs::File::create(path).expect("Impossibile creare il file");
    file.write_all(data.as_bytes()).expect("Errore nella scrittura");
}

/// Get the path of the project source directory.
pub fn get_project_src_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current executable path");

    let mut exe_dir = exe_path.parent().expect("Failed to get parent directory");

    for _ in 0..3 {
        exe_dir = exe_dir.parent().expect("Failed to get parent directory");
    }
    exe_dir.to_path_buf()
}

/// Compute the surrounding rectangles of the given rectangle.
/// The surrounding rectangles are the rectangles that are not covered by the given rectangle.
/// b is the given rectangle and a is the background rectangle.
/// Allows to fill the background with a semi-transparent color except for the given rectangle.
fn surrounding_rectangles(a: Rect, b: Rect) -> Vec<Rect> {
    let mut result = Vec::new();

    // Compute the rectangle above B
    if b.y1 < a.y1 {
        result.push(Rect {
            x0: b.x0,
            x1: b.x1,
            y0: b.y1,
            y1: a.y1,
        });
    }

    // Compute the rectangle below B
    if b.y0 > a.y0 {
        result.push(Rect {
            x0: b.x0,
            x1: b.x1,
            y0: a.y0,
            y1: b.y0,
        });
    }

    // Compute the rectangle to the left of B
    if b.x0 > a.x0 {
        result.push(Rect {
            x0: a.x0,
            x1: b.x0,
            y0: a.y0,
            y1: a.y1,
        });
    }

    // Compute the rectangle to the right of B
    if b.x1 < a.x1 {
        result.push(Rect {
            x0: b.x1,
            x1: a.x1,
            y0: a.y0,
            y1: a.y1,
        });
    }

    result
}