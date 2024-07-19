use druid::{Data, Lens, LocalizedString};

#[derive(Clone, Data, Lens, PartialEq)]
struct ViewState {
    state: bool
}

static mut HOME: bool = false;

use druid::{widget::{Button, Flex, ViewSwitcher}, Widget};

fn build_ui() -> impl Widget<ViewState> {
    
    ViewSwitcher::new(
        |data: &ViewState, _env| data.clone(),
        |selector, _data, _env| match selector.state {
            true => {let navigate_button = Button::new("Go to Detail")
                        .on_click(|_, vs , _| {
                            ViewState::state.with_mut(vs, |state: &mut bool| {
                                *state = false;
                            });
                        });
                    Box::new(Flex::column().with_child(navigate_button))
                },
            false => {let back_button = Button::new("Back")
                            .on_click(|_, vs , _| {
                                ViewState::state.with_mut(vs, |state: &mut bool| {
                                    *state = true;
                                });
                            });

                        let noop_button = Button::new("Do Nothing")
                            .on_click(|_, _, _| {});

                        Box::new(Flex::row().with_child(noop_button).with_child(back_button))
                    }
            }
        )
}


use druid::{AppLauncher, WindowDesc};

fn main() {
    let  initial_state: ViewState = ViewState{state: true};

    let window = WindowDesc::new(|| build_ui(
    ))
        .title(LocalizedString::new("Druid Navigation Example"))
        .resizable(true);

    AppLauncher::with_window(window)
        .launch(initial_state)
        .expect("Failed to launch application");
}



























/*Xilem */

// use xilem::{Application, Window, BoxLayout, Button, Alignment};
// struct AppState {}
// fn main() {
//     let app = Application::new(AppState {}).unwrap();

//     let window = Window::new()
//         .with_title("Xilem Example")
//         .with_size(400, 300);

//     let button1 = Button::new("Button 1");
//     let button2 = Button::new("Button 2");
//     let button3 = Button::new("Button 3");

//     let layout = BoxLayout::vertical()
//         .with_alignment(Alignment::Center)
//         .with_children([button1, button2, button3]);

//     window.set_content(layout);
//     app.run(window).unwrap();
// }


/*EGUI */
// use eframe::{egui::CentralPanel, App, run_native};
// use eframe::egui::Layout;
// use egui::{Align, Button};
 
// mod myapp;
// use myapp::MyApp;

// impl App for MyApp{
//     fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame){
//         self.render_buttons_bar(ctx);
//     }
// }

// fn main(){
//     let app = MyApp;
//     let options = eframe::NativeOptions::default();
//     run_native("myapp", options,Box::new(|cc| Ok(Box::new(MyApp::new(cc)))));
// }

/*DRUID */

// use druid::{
//     widget::{Align, Button, Flex},
//     AppLauncher, Color, Data, Lens, LocalizedString, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetExt, WindowDesc,
// };

// // Step 1: Define your application state
// #[derive(Clone, Data, Lens)]
// struct AppState {}

// fn main() {
//     // Describe the main window
//     let main_window = WindowDesc::new(|| build_ui())
//         .title(LocalizedString::new("druid-example-window"))
//         .window_size((400.0, 300.0));

//     // Create the initial application state
//     let initial_state = AppState {};

//     // Launch the application
//     AppLauncher::with_window(main_window)
//         .use_simple_logger()
//         .launch(initial_state)
//         .expect("Failed to launch application");
// }

// // Step 2: Build the UI
// fn build_ui() -> impl Widget<AppState> {
//     // Create three buttons
//     let button1 = Button::new("Button 1").padding(10.0);
//     let button2 = Button::new("Button 2").padding(10.0);
//     let button3 = Button::new("Button 3").padding(10.0);

//     // Arrange them vertically
//     let buttons = Flex::row().with_child(button1).with_child(button2).with_child(button3);

//     // Center the column within the window
//     Align::centered(buttons)
// }
