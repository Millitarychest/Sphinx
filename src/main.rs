#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tree;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Runtime;
use std::time::Duration;
use eframe::egui;
use egui::Widget;
use tree::*;

fn main() -> eframe::Result {
    let rt = Runtime::new().expect("Unable to create Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();
    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0,500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sphinx",
        options,
        Box::new(|cc| {
        Ok(Box::new(SphinxApp::new(cc)))
    })
    )
}

//in ui vars
struct SphinxApp { 
    tx: Sender<Directory>,
    rx: Receiver<Directory>,
    root_dir: String,
    explorer_dirs: Directory
}

impl SphinxApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self{
        let mut a: SphinxApp = Default::default();
        a.explorer_dirs = dir_walk(0, &PathBuf::from(a.root_dir.clone()), is_dir, sort_by_name).unwrap();
        return a;
    }
}

//starting vals for ui vars
impl Default for SphinxApp { 
    fn default() -> Self {
        let (tx, rx): (Sender<Directory>, Receiver<Directory>) = std::sync::mpsc::channel();

        Self {
            tx,
            rx,
            root_dir: "E:\\dev\\code".to_owned(),
            explorer_dirs: Default::default()
        }
    }
}

//UI definition
impl eframe::App for SphinxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        /////////////////////////Explorer///////////////////////////////////
        egui::SidePanel::left("project_explorer")
            .resizable(false)
            .exact_width(ctx.screen_rect().width()*0.6)
            .show(ctx, |frame| {
                frame.horizontal(|ui|{
                    if ui.button("⟲").clicked() { 
                       refresh_explorer(&self.root_dir, self.tx.clone());
                    }
                    ui.add(egui::Label::new("Root Dir:"));
                    ui.add(egui::TextEdit::singleline(&mut self.root_dir).desired_width(f32::INFINITY));
                });
                frame.separator();//-------------
                frame.with_layout(egui::Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                    ui.add_sized(
                        [ui.available_width(),ui.available_height()*0.1],
                        egui::Button::new("Add new project")
                    );
                    ui.separator();//-------------
                    egui::ScrollArea::vertical().max_height(f32::INFINITY).show(ui, |ui|{
                        ui.visuals_mut().indent_has_left_vline = true;
                        if let Ok(dir) = self.rx.try_recv() {
                            self.explorer_dirs = dir;
                        }
                        self.explorer_dirs.ui(ui);
                    });
                })
            }
        );
        /////////////////////////Tools///////////////////////////////////
        /////////////////////////Git/////////////////////////////////////
        egui::TopBottomPanel::bottom("git_view")
            .resizable(false)
            .exact_height(ctx.screen_rect().height()*0.5)
            .show(ctx, |frame| {
                
            }
        );
        /////////////////////////Ideas///////////////////////////////////
        egui::TopBottomPanel::top("idea_view")
            .resizable(false)
            .exact_height(ctx.screen_rect().height()*0.5)
            .show(ctx, |frame| {
                
            }
        );
    }
}

//UI events
fn refresh_explorer(root: &str, tx: Sender<Directory>) {
    let path = PathBuf::from(root);
    tokio::spawn(async move {
        let dir: Directory = dir_walk(0,&path, is_dir, sort_by_name).unwrap();
        let _ = tx.send(dir);
    });
    //print_tree(&root, &dir);
}

