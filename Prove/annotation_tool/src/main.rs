#![windows_subsystem = "windows"]

use anyhow::Context;
use druid::{AppLauncher, Menu, Screen, WidgetExt, WindowDesc, MenuItem, HasRawWindowHandle, LensExt};
use druid::piet::{Color, RenderContext};
use druid::widget::{Button, Flex, Widget, MainAxisAlignment, Padding};
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

#[derive(Clone, Data, PartialEq)]
pub enum Shapes {
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Rectangle),
    Highlight(Rectangle),
}

#[derive(Clone, Debug, Data, PartialEq)]
pub struct Rectangle {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    color: Color,
}

#[derive(Clone, Debug, Data, PartialEq)]
pub struct Circle {
    center_x: f64,
    center_y: f64,
    radius: f64,
    color: Color,
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    #[data(same_fn = "PartialEq::eq")]
    handled_monitors: Vec<usize>,
    overlay_state: OverlayState,
    start_point: Option<Point>,
    end_point: Option<Point>,
    #[data(same_fn = "PartialEq::eq")]
    shapes: Vec<Shapes>,
    selected_shape: ShapeType, // Currently selected shape type
    selected_color: Color, // Currently selected color
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
                                color: data.selected_color,
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
                                color: data.selected_color,
                            };
                            data.shapes.push(Shapes::Circle(circle));
                        }
                        ShapeType::Line => {
                            let rect = Rectangle {
                                start_x: start.x,
                                start_y: start.y,
                                end_x: end.x,
                                end_y: end.y,
                                color: data.selected_color,
                            };
                            data.shapes.push(Shapes::Line(rect));
                        }
                        ShapeType::Highlight => {
                            let rect = Rectangle {
                                start_x: start.x,
                                start_y: start.y,
                                end_x: end.x,
                                end_y: end.y,
                                color: data.selected_color,
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
        if data.shapes.len() != _old_data.shapes.len() {
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
                    let border_color = rect.color;
                    let rect = Rect::from_points(start, end);
                    let border_width = 2.0;
                    ctx.stroke(rect, &border_color, border_width);
                }
                Shapes::Circle(circle) => {
                    let center = Point::new(circle.center_x, circle.center_y);
                    let radius = circle.radius;
                    let c = druid::kurbo::Circle::new(center, radius);
                    let border_color = circle.color;
                    let border_width = 2.0;
                    ctx.stroke(c, &border_color, border_width);
                }
                Shapes::Line(rect) => {
                    let start = Point::new(rect.start_x, rect.start_y);
                    let end = Point::new(rect.end_x, rect.end_y);
                    let line = Line::new(start, end);
                    let border_color = rect.color;
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
            let outline_color = &data.selected_color;
            let border_width = 2.0;

            match data.selected_shape {
                ShapeType::Rectangle => {
                    let rect = Rect::from_points(start, end);
                    ctx.stroke(rect, outline_color, border_width);
                }
                ShapeType::Circle => {
                    let dx = end.x - start.x;
                    let dy = end.y - start.y;
                    let center = Point::new(start.x, start.y);
                    let radius = (dx.powi(2) + dy.powi(2)).sqrt();
                    let circle = druid::kurbo::Circle::new(center, radius);
                    ctx.stroke(circle, outline_color, border_width);
                }
                ShapeType::Line => {
                    let line = Line::new(start, end);
                    ctx.stroke(line, outline_color, border_width);
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
    let (width, height, x, y) = compute_window_size(0)?;

    let main_window = WindowDesc::new(build_root_widget())
        .title("Draw Shapes")
        .transparent(true)
        .show_titlebar(false)
        .window_size(Size::new(width, height))
        .set_position((x, y))
        .resizable(true);

    let initial_data = AppData {
        handled_monitors: vec![0],
        overlay_state: OverlayState::View,
        start_point: None,
        end_point: None,
        shapes: Vec::new(),
        selected_shape: ShapeType::Rectangle,
        selected_color: Color::BLACK,
    };

    AppLauncher::with_window(main_window)
        .launch(initial_data)
        .expect("Failed to launch application");

    Ok(())
}

fn build_root_widget() -> impl Widget<AppData> {
    let quit_button = Button::new("❌").on_click(|_ctx, _data: &mut AppData, _env| {
        Application::global().quit();
    });

    let draw_button = Button::new("✎").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.overlay_state = OverlayState::Drawing;
        _ctx.request_paint();
    });

    let view_button = Button::new("👁️").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.overlay_state = OverlayState::View;
        _ctx.request_paint();
    });

    let clear_button = Button::new("🗑️").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.shapes.clear();
        _ctx.request_paint();
    });

    let undo_button = Button::new("↺").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.shapes.pop();
        _ctx.request_paint();
    });

    let shape_selector = Button::new("Choose the shape").on_click(|_ctx, _, _| {
        _ctx.show_context_menu(
            Menu::new("")
                .entry(MenuItem::new("Rectangle").on_activate(|_, data: &mut AppData, _| {
                    data.selected_shape = ShapeType::Rectangle;
                }))
                .entry(MenuItem::new("Circle").on_activate(|_, data: &mut AppData, _| {
                    data.selected_shape = ShapeType::Circle;
                }))
                .entry(MenuItem::new("Line").on_activate(|_, data: &mut AppData, _| {
                    data.selected_shape = ShapeType::Line;
                }))
                .entry(MenuItem::new("Highlight").on_activate(|_, data: &mut AppData, _| {
                    data.selected_shape = ShapeType::Highlight;
                })),
                _ctx.to_window(Point::ZERO),
        )
    });

    let color_selector = Button::new("Choose the color").on_click(|_ctx, _, _| {
        _ctx.show_context_menu(
            Menu::new("")
                .entry(MenuItem::new("Black").on_activate(|_, data: &mut AppData, _| {
                    data.selected_color = Color::BLACK;
                }))
                .entry(MenuItem::new("Red").on_activate(|_, data: &mut AppData, _| {
                    data.selected_color = Color::RED;
                }))
                .entry(MenuItem::new("Green").on_activate(|_, data: &mut AppData, _| {
                    data.selected_color = Color::GREEN;
                }))
                .entry(MenuItem::new("Blue").on_activate(|_, data: &mut AppData, _| {
                    data.selected_color = Color::BLUE;
                }))
                .entry(MenuItem::new("Yellow").on_activate(|_, data: &mut AppData, _| {
                    data.selected_color = Color::YELLOW;
                })),
                _ctx.to_window(Point::ZERO),
        )
    });

    let controls = Flex::row()
        .with_flex_spacer(1.0) // Spazio flessibile a sinistra
        .with_child(quit_button)
        .with_spacer(10.0) // Spazio fisso tra i pulsanti
        .with_child(draw_button)
        .with_spacer(10.0)
        .with_child(view_button)
        .with_spacer(10.0)
        .with_child(clear_button)
        .with_spacer(10.0)
        .with_child(undo_button)
        .with_spacer(10.0)
        .with_child(shape_selector)
        .with_spacer(10.0)
        .with_child(color_selector)
        .with_flex_spacer(1.0) // Spazio flessibile a destra
        .main_axis_alignment(MainAxisAlignment::Center) // Centra i widget lungo l'asse principale
        .must_fill_main_axis(true)
        .background(Color::WHITE);

    Flex::column()
        .with_child(Padding::new((0.0, 10.0), controls))
        .with_flex_child(DrawingOverlay::new(), 10.0)
}

pub fn compute_window_size(index: usize) -> anyhow::Result<(f64, f64, f64, f64)> {
    let screens = Screen::get_monitors();
    println!("{:?}", screens);
    let width = screens.to_vec()[index].virtual_rect().width();
    let height = screens.to_vec()[index].virtual_rect().height();
    let top_x = screens.to_vec()[index].virtual_rect().x0;
    let top_y = screens.to_vec()[index].virtual_work_rect().y0;
    Ok((width, height-0.5, top_x, top_y+0.5))
}
