#![windows_subsystem = "windows"]

mod buttons;

use std::{env, thread};
use anyhow::Context;
use druid::{AppLauncher, Screen, WidgetExt, WindowDesc, ExtEventSink, Target, Selector};
use druid::piet::{Color, RenderContext};
use druid::widget::{Flex, MainAxisAlignment, Widget};
use druid::{Data, Env, EventCtx, Point, Rect, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size};
use druid::kurbo::Line;


const STDIN_INPUT: Selector<String> = Selector::new("stdin.input");

/// Represents the current state of the annotation tool.
/// The tool can either be in `Drawing` mode, where the user is actively drawing,
/// or in `Idle` mode, where no drawing is taking place.
#[derive(PartialEq, Debug, Clone, Data)]
pub enum OverlayState {
    Drawing,
    Idle,
}

/// Represents the different types of shapes that can be drawn by the annotation tool.
/// Each variant corresponds to a specific shape the user can create.
#[derive(Clone, PartialEq, Debug, Data)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
    Highlight,
}

/// Represents different drawable shapes in the annotation tool, each with its associated data.
/// Each variant stores the specific parameters needed to define the corresponding shape.
#[derive(Clone, Data, PartialEq)]
pub enum Shapes {
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Rectangle),
    Highlight(Rectangle),
}

/// Represents a rectangle shape in the annotation tool.
/// Defined by its start and end coordinates, as well as its color.
#[derive(Clone, Debug, Data, PartialEq)]
pub struct Rectangle {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    color: Color,
}

/// Represents a circular shape in the annotation tool.
/// Defined by its center coordinates, radius, and color.
#[derive(Clone, Debug, Data, PartialEq)]
pub struct Circle {
    center_x: f64,
    center_y: f64,
    radius: f64,
    color: Color,
}

/// Represents the main application state for the annotation tool.
/// Contains information about the current state of the tool, drawn shapes, 
/// and user-selected options such as shape type and color.
#[derive(Clone, Data, Lens)]
pub struct AppData {
    overlay_state: OverlayState,
    start_point: Option<Point>,
    end_point: Option<Point>,
    #[data(same_fn = "PartialEq::eq")]
    shapes: Vec<Shapes>,
    selected_shape: ShapeType,
    selected_color: Color,
}

/// A widget that provides an interactive overlay for drawing shapes
/// such as rectangles, circles, lines, and highlights.
/// It handles user input events and manages the drawing process.
pub struct DrawingOverlay;

impl DrawingOverlay {
    pub fn new() -> Self {
        DrawingOverlay {}
    }
}

impl Widget<AppData> for DrawingOverlay {
    /// Handles various user input events (commands, mouse events) 
    /// to update the application state and trigger redraws.
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            // Handles custom input commands from stdin to clear the shapes.
            Event::Command(cmd) if cmd.is(STDIN_INPUT) => {
                if let Some(input) = cmd.get(STDIN_INPUT) {
                    if input == "clear" {
                        data.shapes.clear();
                        ctx.request_paint();
                    }
                }
            }
            // Handles mouse down event: begins a new shape drawing.
            Event::MouseDown(mouse) => {
                // Start tracking the rectangle
                data.start_point = Some(mouse.pos);
                data.overlay_state = OverlayState::Drawing;
                ctx.request_paint();
            }
            // Handles mouse move event: updates the shape endpoint during drawing.
            Event::MouseMove(mouse) => {
                if data.overlay_state == OverlayState::Drawing {
                    data.end_point = Some(mouse.pos);
                    ctx.request_paint();
                }
            }
            // Handles mouse up event: finalizes the shape and adds it to the shapes list.
            Event::MouseUp(_) => {
                if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
                    match data.selected_shape {
                        // Create and store the appropriate shape based on the selected type.
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
                // Reset the overlay state and clear temporary points.
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

    /// Handles updates to the application state.
    /// Triggers a repaint when the overlay state or shapes list changes.
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

    /// Handles the drawing of the overlay and shapes on the screen.
    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppData, _env: &Env) {
        // Fill the background with a semi-transparent white color.
        let background_rect = ctx.size().to_rect();
        ctx.fill(background_rect, &Color::rgba8(0xff, 0xff, 0xff, 0x4));
    
        // Draw all existing shapes.
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

        // Draw an outline for the current shape being drawn, if any.
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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Errore: nessun argomento passato. Specifica un numero intero positivo >= 1.");
        return Ok(());
    }

    let index = args[1].parse::<usize>().context("Errore: l'argomento passato non è un numero intero positivo.")?;

    let (width, height, x, y) = compute_window_size(index)?;

    // Configure the main window of the application.
    let main_window = WindowDesc::new(build_root_widget())
        .title("Annotation Tool")
        .set_always_on_top(true)
        .transparent(true)
        .show_titlebar(false)
        .window_size(Size::new(width, height))
        .set_position((x, y));

    let initial_data = AppData {
        overlay_state: OverlayState::Idle,
        start_point: None,
        end_point: None,
        shapes: Vec::new(),
        selected_shape: ShapeType::Rectangle,
        selected_color: Color::BLACK,
    };

    // Create a launcher to start the application with the main window.
    let launcher = AppLauncher::with_window(main_window);

    // Get an external event handle for sending events from outside the main thread.
    let event_sink = launcher.get_external_handle();

    // Start a separate thread to read commands from stdin.
    start_stdin_reader(event_sink);

    launcher
        .launch(initial_data)
        .expect("Failed to launch application");

    Ok(())
}

// Function to start a separate thread for reading from stdin
fn start_stdin_reader(event_sink: ExtEventSink) {
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut buffer = String::new();

        loop {
            buffer.clear();
            // Read a line from stdin and store it in the buffer
            if stdin.read_line(&mut buffer).is_ok() {
                let input = buffer.trim().to_string();
                if input == "quit" {
                    // If input is "quit", send a command to quit the application
                    event_sink.submit_command(druid::commands::QUIT_APP, (), Target::Global).unwrap();
                    break;
                } else {
                    // Otherwise, send the input command to the main thread for processing.
                    event_sink.submit_command(STDIN_INPUT, input, Target::Global).unwrap();
                }
            }
        }
    });
}

/// Builds the main root widget for the application, containing the drawing overlay and control buttons.
fn build_root_widget() -> impl Widget<AppData> {
    let quit_button = buttons::quit_button(); // Button to quit the application

    let clear_button = buttons::clear_button(); // Button to clear all shapes

    let undo_button = buttons::undo_button(); // Button to undo the last shape

    let shape_selector = buttons::choose_shape_button(); // Button to choose the shape type

    let color_selector = buttons::choose_color_button(); // Button to choose the color

    let controls = Flex::row()
        .with_flex_spacer(1.0) 
        .with_child(quit_button)
        .with_spacer(25.0) 
        .with_child(clear_button)
        .with_spacer(25.0)
        .with_child(undo_button)
        .with_spacer(25.0)
        .with_child(shape_selector)
        .with_spacer(25.0)
        .with_child(color_selector)
        .with_flex_spacer(1.0) 
        .main_axis_alignment(MainAxisAlignment::Center) 
        .must_fill_main_axis(true)
        .padding(10.0)
        .background(Color::rgba8(31,34,37,255));

    Flex::column()
        .with_child(controls)
        .with_flex_child(DrawingOverlay::new(), 10.0)
}

/// Computes the window size based on the index of the monitor to display the application on.
/// Returns the width, height, top-left x-coordinate, and top-left y-coordinate of the window.
/// The index is 1-based, where 1 corresponds to the primary monitor.
/// The index is the first command-line argument passed to the application.
pub fn compute_window_size(index: usize) -> anyhow::Result<(f64, f64, f64, f64)> {
    let screen = Screen::get_monitors().to_vec()[index-1].clone();
    let width = screen.virtual_work_rect().width();
    let height = screen.virtual_work_rect().height();
    let top_x = screen.virtual_work_rect().x0;
    let top_y = screen.virtual_work_rect().y0;
    #[cfg(target_os = "macos")] {
        // On macOS, the height of the window is reduced by 20 pixels to account for the menu bar on secondary screens.
        // On the primary screen, the height is not reduced since druid automatically accounts for the menu bar.
        if index > 1 {
            return Ok((width, height-20.0, top_x, top_y+20.0));
        }
    }
    Ok((width, height, top_x, top_y))
}
