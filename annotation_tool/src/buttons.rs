use druid::{Menu, MenuItem, Point, UnitPoint, WidgetExt};
use druid::piet::{Color, RenderContext};
use druid::widget::{Controller, Flex, Painter, Svg, SvgData, Widget};
use druid::{Env, EventCtx, Event, Application};
use std::str::FromStr;

use crate::{AppData, ShapeType};

/// Controller for the quit button
/// When the button is clicked, the application will quit
struct QuitButtonController;

impl<W: Widget<AppData>> Controller<AppData, W> for QuitButtonController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            Application::global().quit();
        }
        child.event(ctx, event, data, env);
    }
}

/// Create a custom quit button
pub fn quit_button() -> impl Widget<AppData> {
    let svg_data = include_str!("../../assets/close_app.svg");
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); 

    let button_painter = Painter::new(|ctx, _data: &AppData, _env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    Flex::column()
        .with_child(svg)
        .align_vertical(UnitPoint::CENTER)
        .align_horizontal(UnitPoint::CENTER)
        .background(button_painter) 
        .fix_size(30.0, 30.0) 
        .controller(QuitButtonController)
}

/// Controller for the clear button
/// When the button is clicked, the shapes will be cleared
struct ClearButtonController;

impl<W: Widget<AppData>> Controller<AppData, W> for ClearButtonController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            data.shapes.clear();
            ctx.request_paint();
        }
        child.event(ctx, event, data, env);
    }
}

/// Create a custom clear button
pub fn clear_button() -> impl Widget<AppData> {
    let svg_data = include_str!("../../assets/clear.svg");
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); 

    let button_painter = Painter::new(|ctx, _data: &AppData, _env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    Flex::column()
        .with_child(svg)
        .align_vertical(UnitPoint::CENTER)
        .align_horizontal(UnitPoint::CENTER)
        .background(button_painter) 
        .fix_size(30.0, 30.0) 
        .controller(ClearButtonController) 
}

/// Controller for the undo button
/// When the button is clicked, the last drawn shape will be removed
struct UndoButtonController;

impl<W: Widget<AppData>> Controller<AppData, W> for UndoButtonController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            data.shapes.pop();
            ctx.request_paint();
        }
        child.event(ctx, event, data, env);
    }
}

/// Create a custom undo button
pub fn undo_button() -> impl Widget<AppData> {
    let svg_data = include_str!("../../assets/undo.svg"); 
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); 

    let button_painter = Painter::new(|ctx, _data: &AppData, _env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    Flex::column()
        .with_child(svg)
        .align_vertical(UnitPoint::CENTER)
        .align_horizontal(UnitPoint::CENTER)
        .background(button_painter) 
        .fix_size(30.0, 30.0) 
        .controller(UndoButtonController)
}

/// Controller for the shape selection button.
/// This button triggers the display of a list of available shapes.
/// When a shape is selected from the list, the app's data is updated.
struct ChooseShapeController;

impl<W: Widget<AppData>> Controller<AppData, W> for ChooseShapeController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            ctx.show_context_menu(
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
                ctx.to_window(Point::ZERO),
            );
        }
        child.event(ctx, event, data, env);
    }
}

/// Create a custom shape list button
pub fn choose_shape_button() -> impl Widget<AppData> {
    let svg_data = include_str!("../../assets/shapes.svg"); 
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); 

    let button_painter = Painter::new(|ctx, _data: &AppData, _env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    Flex::column()
        .with_child(svg)
        .align_vertical(UnitPoint::CENTER)
        .align_horizontal(UnitPoint::CENTER)
        .background(button_painter) 
        .fix_size(30.0, 30.0) 
        .controller(ChooseShapeController) 
}

/// Controller for the color selection button.
/// This button triggers the display of a list of available colors.
/// When a color is selected from the list, the app's data is updated.
struct ChooseColorController;

impl<W: Widget<AppData>> Controller<AppData, W> for ChooseColorController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            ctx.show_context_menu(
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
                ctx.to_window(Point::ZERO),
            )
        }
        child.event(ctx, event, data, env);
    }
}

/// Create a custom color list button
pub fn choose_color_button() -> impl Widget<AppData> {
    let svg_data = include_str!("../../assets/colors.svg");
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); 

    let button_painter = Painter::new(|ctx, _data: &AppData, _env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    Flex::column()
        .with_child(svg)
        .align_vertical(UnitPoint::CENTER)
        .align_horizontal(UnitPoint::CENTER)
        .background(button_painter) 
        .fix_size(30.0, 30.0) 
        .controller(ChooseColorController)
}