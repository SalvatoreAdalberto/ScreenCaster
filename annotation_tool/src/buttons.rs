use druid::{Menu, MenuItem, Point, WidgetExt};
use druid::piet::{Color, RenderContext};
use druid::widget::{Controller, Flex, Painter, Svg, SvgData, Widget};
use druid::{Env, EventCtx, Event, Application};
use std::str::FromStr;

use crate::{AppData, OverlayState, ShapeType};
struct QuitButtonController;

impl<W: Widget<AppData>> Controller<AppData, W> for QuitButtonController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            Application::global().quit();
        }
        child.event(ctx, event, data, env);
    }
}

pub fn quit_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/close_app.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(QuitButtonController) // Aggiungi comportamento al click
}


struct DrawButtonController;

impl<W: Widget<AppData>> Controller<AppData, W> for DrawButtonController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            data.overlay_state = OverlayState::Drawing;
            data.current_background_color = Color::rgba8(0xff, 0xff, 0xff, 0x4);
            ctx.request_paint();
        }
        child.event(ctx, event, data, env);
    }
}

pub fn draw_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/draw.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(DrawButtonController) // Aggiungi comportamento al click
}

struct ViewButtonController;

impl<W: Widget<AppData>> Controller<AppData, W> for ViewButtonController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::MouseDown(_) = event {
            data.overlay_state = OverlayState::View;
            data.current_background_color = Color::rgba8(0xff, 0xff, 0xff, 0x00);
            ctx.request_paint();
        }
        child.event(ctx, event, data, env);
    }
}

pub fn view_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/view.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(ViewButtonController) // Aggiungi comportamento al click
}

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

pub fn clear_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/clear.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(ClearButtonController) // Aggiungi comportamento al click
}

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

pub fn undo_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/undo.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(UndoButtonController) // Aggiungi comportamento al click
}

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

pub fn choose_shape_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/shapes.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(ChooseShapeController) // Aggiungi comportamento al click
}


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

pub fn choose_color_button() -> impl Widget<AppData> {
    // Carica l'SVG
    let svg_data = include_str!("../../assets/colors.svg"); // Percorso al tuo SVG
    let svg = Svg::new(SvgData::from_str(svg_data).unwrap())
        .fix_size(25.0, 25.0); // Dimensione del contenuto SVG

    // Pittura per il background del bottone
    let button_painter = Painter::new(|ctx, _data: &AppData, env| {
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &Color::rgb8(51, 89, 218));
    });

    // Contenitore per il bottone
    Flex::column()
        .with_child(svg)
        .background(button_painter) // Sfondo colorato
        .fix_size(30.0, 30.0) // Dimensioni del bottone
        .controller(ChooseColorController) // Aggiungi comportamento al click
}