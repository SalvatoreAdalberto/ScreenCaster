use eframe::egui::{Context};
use egui::{CentralPanel, Color32, FontId, RichText, Window};
pub struct MyApp;
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);


impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self {
        }
    }

    pub fn render_buttons_bar(&self, ctx: &Context){
        let button_bar = CentralPanel::default().show(ctx,
            |ui|{
            ui.add_space(ui.available_height()/2.0);
            let available_width = ui.available_width();
            let button_count = 3; // Number of buttons you want to display

                // Calculate spacing based on available width and button count
            let button_width = 400.0; // Example button width
            let spacing = (available_width - (button_width * button_count as f32)) / (button_count - 1) as f32;
            
            ui.horizontal( |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center),
                 |ui|{
                ui.group(|ui|{
                    for i in 0..button_count {
                        let button_position =  (button_width + spacing);
                        
                            ui.add_space(button_position);
                            ui.button("Button");
                       
                    }
                });
            });
            });
            // let (w,h) = (ui.available_width()/3.0, ui.available_height()/9.0);
            
            // egui::menu::bar( ui, 
            // |ui|{
            // ui.add_sized([w,h], egui::Button::new(RichText::new("New Cast").font(FontId::proportional(40.0)).color(CYAN)));
            // ui.add_sized([w,h], egui::Button::new(RichText::new("Connect").font(FontId::proportional(40.0)).color(CYAN)));
            // ui.add_sized([w,h], egui::Button::new(RichText::new("DIOCANE").font(FontId::proportional(40.0)).color(CYAN)));

            // });
    });

    }
}