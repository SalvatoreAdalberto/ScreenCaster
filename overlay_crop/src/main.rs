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

#[derive(Clone, Data, Lens)]
pub struct AppData {
    start_point: Option<Point>,
    end_point: Option<Point>,
}

pub struct DrawingOverlay;

impl DrawingOverlay {
    pub fn new() -> Self {
        DrawingOverlay {}
    }
}

impl Widget<AppData> for DrawingOverlay {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                // Start tracking the rectangle
                data.start_point = Some(mouse.pos);
                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                // Update the endpoint
                data.end_point = Some(mouse.pos);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                // Complete the drawing and add the shape to the shapes vector
                if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
                    let mut x = start;
                    let mut y = end;
                    #[cfg(any(target_os = "windows", target_os = "linux"))]
                    {
                        x = ctx.to_screen(start);
                        y = ctx.to_screen(end);
                    }

                    let rect = Rect::from_points(x, y).trunc();
                    let scale = ctx.scale();
                    save_point(rect, scale);
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


        // Draw current selection outline, ensuring transparency inside
        if let (Some(start), Some(end)) = (data.start_point, data.end_point) {

            let rect = Rect::from_points(start, end);
            let background_rect = ctx.size().to_rect();
            let surrounding_rects = surrounding_rectangles(background_rect, rect);

            for surrounding_rect in surrounding_rects {
                ctx.fill(surrounding_rect, &Color::rgba(0.0, 0.0, 0.0, 0.5));
            }
        }
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

    AppLauncher::with_window(main_window)
        .launch(initial_data)
        .expect("Failed to launch application");

    Ok(())
}

pub fn compute_window_size(index: usize) -> anyhow::Result<(f64, f64, f64, f64)> {
    let screens = druid::Screen::get_monitors();
    println!("{:?}", screens);
    let width = screens.to_vec()[index-1].virtual_rect().width();
    let height = screens.to_vec()[index-1].virtual_rect().height();
    let top_x = screens.to_vec()[index-1].virtual_rect().x0;
    let top_y = screens.to_vec()[index-1].virtual_work_rect().y0;
    Ok((width, height-0.5, top_x, top_y+0.5))
}

fn save_point(rect: Rect, scale: Scale) {
    // Save the coordinates of the rectangle in a file
    println!("Rectangle: {:?}", rect);
    println!("Scale factor: {:?}", scale);
    let x = rect.x0 * scale.x();
    let y = rect.y0 * scale.y();
    let width = rect.width() * scale.x();
    let height = rect.height() * scale.y();
    let data = format!("{},{},{},{}", x, y, width, height);
    println!("{}", data);
    let mut path = get_project_src_path();
    path.push("config/crop.txt");
    println!("{:?}", path);
    let mut file = fs::File::create(path).expect("Impossibile creare il file");
    file.write_all(data.as_bytes()).expect("Errore nella scrittura");
}

pub fn get_project_src_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current executable path");

    let mut exe_dir = exe_path.parent().expect("Failed to get parent directory");

    for _ in 0..3 {
        exe_dir = exe_dir.parent().expect("Failed to get parent directory");
    }
    exe_dir.to_path_buf()
}

fn surrounding_rectangles(a: Rect, b: Rect) -> Vec<Rect> {
    let mut result = Vec::new();

    // Calcola il rettangolo sopra B
    if b.y1 < a.y1 {
        result.push(Rect {
            x0: b.x0,
            x1: b.x1,
            y0: b.y1,
            y1: a.y1,
        });
    }

    // Calcola il rettangolo sotto B
    if b.y0 > a.y0 {
        result.push(Rect {
            x0: b.x0,
            x1: b.x1,
            y0: a.y0,
            y1: b.y0,
        });
    }

    // Calcola il rettangolo a sinistra di B
    if b.x0 > a.x0 {
        result.push(Rect {
            x0: a.x0,
            x1: b.x0,
            y0: a.y0,
            y1: a.y1,
        });
    }

    // Calcola il rettangolo a destra di B
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