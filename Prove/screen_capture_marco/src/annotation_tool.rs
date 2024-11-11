#![windows_subsystem = "windows"]

use anyhow::Context;
use druid::{AppLauncher, LocalizedString, WidgetExt, WindowDesc};
use druid::piet::{Color, RenderContext};
use druid::widget::{Button, Flex, Widget};
use druid::{Data, Env, EventCtx, Point, Rect, Lens, Event, LifeCycle, LifeCycleCtx, UpdateCtx, LayoutCtx, BoxConstraints, Size, Application};
use eframe::App;

#[derive(PartialEq, Debug, Clone, Data)]
pub enum OverlayState {
    View,
    Drawing,
    Idle,
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    overlay_state: OverlayState,
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
                // Inizia a tracciare il rettangolo
                data.start_point = Some(mouse.pos);
                data.overlay_state = OverlayState::Drawing;
                ctx.request_paint();
            }
            Event::MouseMove(mouse) => {
                // Aggiorna il punto finale mentre si trascina
                if data.overlay_state == OverlayState::Drawing {
                    data.end_point = Some(mouse.pos);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_) => {
                // Completa il disegno e torna allo stato inattivo
                data.overlay_state = OverlayState::Idle;
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
        if _old_data.overlay_state != data.overlay_state {
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

        if data.overlay_state == OverlayState::View {
            println!("Rendering view");
            let background_rect = ctx.size().to_rect();
            ctx.fill(background_rect, &Color::rgba8(0xff, 0xff, 0xff, 0)); // Cambia il colore di sfondo qui
            // Disegna solo il bordo del rettangolo selezionato
            if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
                let rect = Rect::from_points(start, end);
                let border_color = Color::rgb8(0, 0, 255); // Colore del bordo (blu)
                let border_width = 2.0; // Spessore del bordo

                ctx.stroke(rect, &border_color, border_width);
            }
        }
        else {
            // Riempie l'intera area della finestra con il colore di sfondo
            let background_rect = ctx.size().to_rect();
            ctx.fill(background_rect, &Color::rgba8(0xff, 0xff, 0xff, 0x4)); // Cambia il colore di sfondo qui

            // Disegna solo il bordo del rettangolo selezionato
            if let (Some(start), Some(end)) = (data.start_point, data.end_point) {
                let rect = Rect::from_points(start, end);
                let border_color = Color::rgb8(0, 0, 255); // Colore del bordo (blu)
                let border_width = 2.0; // Spessore del bordo

                ctx.stroke(rect, &border_color, border_width);
            }
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let (width, height) = compute_window_size()?;


    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("Disegna figure"))
        .set_always_on_top(true)
        .transparent(true)
        .show_titlebar(false)
        .window_size(Size::new(width, height))
        .set_position((0f64, 0.0f64))
        .resizable(false);

    let initial_data = AppData {
        overlay_state: OverlayState::View,
        start_point: None,
        end_point: None,
    };

    AppLauncher::with_window(main_window)
        .launch(initial_data)
        .expect("Failed to launch application");

    Ok(())
}

fn build_root_widget() -> impl Widget<AppData> {
    // let background = Painter::new(|ctx, _data, _env| {
    //     let boundaries = ctx.size().to_rect();
    //     ctx.fill(boundaries, &Color::rgba8(0xff, 0xff, 0xff, 0xf)); // Colore di sfondo trasparente
    // });
    let button = Button::new("Ciao").on_click(|_ctx, _data: &mut AppData, _env| {
        Application::global().quit(); // Termina l'applicazione quando si preme il bottone
    });

    let button2 = Button::new("Disegna").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.overlay_state = OverlayState::Drawing;
        _ctx.request_paint();
        println!("Disegna");
    });

    let button3 = Button::new("Visualizza").on_click(|_ctx, _data: &mut AppData, _env| {
        _data.overlay_state = OverlayState::View;
        _ctx.request_paint();
        println!("{:?}", _data.overlay_state);
    });

    let row = Flex::row()
        .with_child(button)
        .with_child(button2)
        .with_child(button3)
        .background(Color::rgba8(0x00, 0x00, 0x00, 0xff));

    Flex::column()
        //.with_child(DrawingOverlay::new())
        .with_child(row)
        .with_flex_child(DrawingOverlay::new(), 10.0)
}

pub fn compute_window_size() -> anyhow::Result<(f64, f64)> {
    let screens = druid::Screen::get_monitors();

    let width = screens.to_vec()[0].virtual_work_rect().width();
    let height = screens.to_vec()[0].virtual_work_rect().height();

    Ok((width, height))
}