#![windows_subsystem = "windows"]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::env;
use anyhow::Context;
use druid::{AppLauncher, LocalizedString, Scale, WidgetExt, WindowDesc};
use druid::piet::{Color, RenderContext};
use druid::widget::{Widget};
use druid::{Data, Env, EventCtx, Point, Rect, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size};
use core_graphics::display::{CGDisplay, CGDisplayBounds, CGMainDisplayID};

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
                    let rect = Rect::from_points(start, end);
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
        data: &AppData,
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

        let background_rect = ctx.size().to_rect();
        ctx.fill(background_rect, &Color::rgba8(0xff, 0xff, 0xff, 0x4));

        // Draw current selection outline
        if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
            let border_width = 2.0;

            let rect = Rect::from_points(start, end);
            ctx.stroke(rect, &Color::BLACK, border_width);
            ctx.fill(rect, &Color::rgba8(0x00, 0x00, 0x00, 0x00));
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let (width, height, x, y) = compute_window_size()?;

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

pub fn compute_window_size() -> anyhow::Result<(f64, f64, f64, f64)> {
    let screens = druid::Screen::get_monitors();
    println!("{:?}", screens);
    let width = screens.to_vec()[0].virtual_rect().width();
    let height = screens.to_vec()[0].virtual_rect().height();
    let top_x = screens.to_vec()[0].virtual_rect().x0;
    let top_y = screens.to_vec()[0].virtual_work_rect().y0;
    Ok((width, height, top_x, top_y))
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