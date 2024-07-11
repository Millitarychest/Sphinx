#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0,500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sphinx",
        options,
        Box::new(|cc| {
        Ok(Box::<SphinxApp>::default())
    })
    )
}

//in ui vars
struct SphinxApp { 
    
}

//starting vals for ui vars
impl Default for SphinxApp { 
    fn default() -> Self {
        Self {}
    }
}

//UI definition
impl eframe::App for SphinxApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("project_explorer").resizable(false)
            .min_width(ctx.screen_rect().width()*0.6).show(ctx, |frame| {});
    }
}
