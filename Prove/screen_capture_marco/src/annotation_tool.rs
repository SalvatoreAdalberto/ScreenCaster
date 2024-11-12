#![windows_subsystem = "windows"]

use anyhow::Context;
use druid::{AppLauncher, LocalizedString, WidgetExt, WindowDesc};
use druid::piet::{Color, RenderContext};
use druid::widget::{Button, Flex, Widget, RadioGroup, LensWrap};
use druid::{Data, Env, EventCtx, Point, Rect, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size, Application};
use druid::kurbo::{Line};

#[derive(PartialEq, Debug, Clone, Data)]
pub enum OverlayState {
    View,
    Drawing,
    Idle,
}

#[derive(Clone, PartialEq, Debug, Data)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
    Highlight,
}

#[derive(Clone, Data)]
pub enum Shapes {
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Rectangle),
    Highlight(Rectangle),
}

#[derive(Clone, Debug, Data)]
pub struct Rectangle {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
}

#[derive(Clone, Debug, Data)]
pub struct Circle {
    center_x: f64,
    center_y: f64,
    radius: f64,
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    overlay_state: OverlayState,
    start_point: Option<Point>,
    end_point: Option<Point>,
    #[data(ignore)]
    shapes: Vec<Shapes>,
    selected_shape: ShapeType, // Currently selected shape type
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
                data.overlay_state = OverlayState::Drawing;
                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                // Update the endpoint while dragging
                if data.overlay_state == OverlayState::Drawing {
                    data.end_point = Some(mouse.pos);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_) => {
                // Complete the drawing and add the shape to the shapes vector
                if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
                    match data.selected_shape {
                        ShapeType::Rectangle => {
                            let rect = Rectangle {
                                start_x: start.x,
                                start_y: start.y,
                                end_x: end.x,
                                end_y: end.y,
                            };
                            data.shapes.push(Shapes::Rectangle(rect));
                        }
                        ShapeType::Circle => {
                            let dx = end.x - start.x;
                            let dy = end.y - start.y;
                            let radius = (dx.powi(2) + dy.powi(2)).sqrt();
                            let circle = Circle {
                                center_x: start.x,
                                center_y: start.y,
                                radius,
                            };
                            data.shapes.push(Shapes::Circle(circle));
                        }
                        ShapeType::Line => {
                            let rect = Rectangle {
                                start_x: start.x,
                                start_y: start.y,
                                end_x: end.x,
                                end_y: end.y,
                            };
                            data.shapes.push(Shapes::Line(rect));
                        }
                        ShapeType::Highlight => {
                            let rect = Rectangle {
                                start_x: start.x,
                                start_y: start.y,
                                end_x: end.x,
                                end_y: end.y,
                            };
                            data.shapes.push(Shapes::Highlight(rect));
                        }
                    }
                }
                data.overlay_state = OverlayState::Idle;
                data.start_point = None;
                data.end_point = None;
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
        if data.overlay_state != _old_data.overlay_state {
            ctx.request_paint();
        }
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
        // Background fill
        if data.overlay_state == OverlayState::View {
            let background_rect = ctx.size().to_rect();
            ctx.fill(background_rect, &Color::rgba8(0xff, 0xff, 0xff, 0));
        }
        else {
            let background_rect = ctx.size().to_rect();
            ctx.fill(background_rect, &Color::rgba8(0xff, 0xff, 0xff, 0x4));
        }

        // Draw existing shapes
        for shape in &data.shapes {
            match shape {
                Shapes::Rectangle(rect) => {
                    let start = Point::new(rect.start_x, rect.start_y);
                    let end = Point::new(rect.end_x, rect.end_y);
                    let rect = Rect::from_points(start, end);
                    let border_color = Color::BLACK;
                    let border_width = 2.0;
                    ctx.stroke(rect, &border_color, border_width);
                }
                Shapes::Circle(circle) => {
                    let center = Point::new(circle.center_x, circle.center_y);
                    let radius = circle.radius;
                    let c = druid::kurbo::Circle::new(center, radius);
                    let border_color = Color::BLACK;
                    let border_width = 2.0;
                    ctx.stroke(c, &border_color, border_width);
                }
                Shapes::Line(rect) => {
                    let start = Point::new(rect.start_x, rect.start_y);
                    let end = Point::new(rect.end_x, rect.end_y);
                    let line = Line::new(start, end);
                    let border_color = Color::BLACK;
                    let border_width = 2.0;
                    ctx.stroke(line, &border_color, border_width);
                }
                Shapes::Highlight(rect) => {
                    let start = Point::new(rect.start_x, rect.start_y);
                    let end = Point::new(rect.end_x, rect.end_y);
                    let rect = Rect::from_points(start, end);
                    ctx.fill(rect, &Color::rgba8(0xff, 0xff, 0x00, 0x5f));
                }
            }
        }

        // Draw current selection outline
        if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
            let outline_color = Color::BLACK;
            let border_width = 2.0;

            match data.selected_shape {
                ShapeType::Rectangle => {
                    let rect = Rect::from_points(start, end);
                    ctx.stroke(rect, &outline_color, border_width);
                }
                ShapeType::Circle => {
                    let dx = end.x - start.x;
                    let dy = end.y - start.y;
                    let center = Point::new(start.x, start.y);
                    let radius = (dx.powi(2) + dy.powi(2)).sqrt();
                    let circle = druid::kurbo::Circle::new(center, radius);
                    ctx.stroke(circle, &outline_color, border_width);
                }
                ShapeType::Line => {
                    let line = Line::new(start, end);
                    ctx.stroke(line, &outline_color, border_width);
                }
                ShapeType::Highlight => {
                    let rect = Rect::from_points(start, end);
                    ctx.fill(rect, &Color::rgba8(0xff, 0xff, 0x00, 0x5f));
                }
            }
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let (width, height, x, y) = compute_window_size()?;

    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("Draw Shapes"))
        .set_always_on_top(true)
        .transparent(true)
        .show_titlebar(false)
        .window_size(Size::new(width, height))
        .set_position((x, y))
        .resizable(false);

    let initial_data = AppData {
        overlay_state: OverlayState::View,
        start_point: None,
        end_point: None,
        shapes: Vec::new(),
        selected_shape: ShapeType::Rectangle,
    };

    AppLauncher::with_window(main_window)
        .launch(initial_data)
        .expect("Failed to launch application");

    Ok(())
}

fn build_root_widget() -> impl Widget<AppData> {
    let quit_button = Button::new("Quit").on_click(|_ctx, _data: &mut AppData, _env| {
        Application::global().quit();
    });

    let draw_button = Button::new("Draw").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.overlay_state = OverlayState::Drawing;
        _ctx.request_paint();
    });

    let view_button = Button::new("View").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.overlay_state = OverlayState::View;
        _ctx.request_paint();
    });

    let clear_button = Button::new("Clear").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.shapes.clear();
        _ctx.request_paint();
    });

    // Wrap `shape_selector` in `LensWrap`
    let shape_selector = LensWrap::new(
        RadioGroup::column(vec![
            ("Rectangle", ShapeType::Rectangle),
            ("Circle", ShapeType::Circle),
            ("Line", ShapeType::Line),
            ("Highlight", ShapeType::Highlight),
        ]),
        AppData::selected_shape, // Lens for focusing on `selected_shape`
    );

    let controls = Flex::row()
        .with_child(quit_button)
        .with_child(draw_button)
        .with_child(view_button)
        .with_child(clear_button)
        .with_child(shape_selector)
        .background(Color::rgba8(0x00, 0x00, 0x00, 0xff));

    Flex::column()
        .with_child(controls)
        .with_flex_child(DrawingOverlay::new(), 10.0)
}

pub fn compute_window_size() -> anyhow::Result<(f64, f64, f64, f64)> {
    let screens = druid::Screen::get_monitors();
    let width = screens.to_vec()[0].virtual_work_rect().width();
    let height = screens.to_vec()[0].virtual_work_rect().height();
    let top_x = screens.to_vec()[0].virtual_work_rect().x0;
    let top_y = screens.to_vec()[0].virtual_work_rect().y0;
    Ok((width, height, top_x, top_y))
}
