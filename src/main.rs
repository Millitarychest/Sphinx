#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(unused_parens)]
mod dir_tree;
#[allow(unused_parens)]
mod project;
#[allow(unused_parens)]
mod sphinx_git;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use egui_dropdown::DropDownBox;
use sphinx_git::GitWidget;
use tokio::runtime::Runtime;
use std::time::Duration;
use eframe::egui;

use dir_tree::*;
use project::*;

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

#[derive(Default,Clone)]
struct AddDialog {
    //ui state
    open: bool,
    //autocomplete
    known_langs: Vec<String>,
    known_category: Vec<String>,
    //project params
    lang: String,
    category: String,
    name: String,
}

impl AddDialog {
    fn reset(&mut self){
            self.open = false;
            self.lang = Default::default();
            self.category = Default::default();
            self.name = "new_project".to_owned();
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
struct AppSettings{
    open: bool,
    root_dir: String,
    selected_project_path: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default,Clone)]
struct CommitSettings{
    git_user: String,
    git_mail: String,
}

#[derive(Default)]
struct AppState{
    git_history: GitWidget,
    explorer_dirs: Directory,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SphinxApp {
    #[serde(skip)]
    add_dialog: AddDialog,
    app_settings: AppSettings,
    commit_settings: CommitSettings,
    #[serde(skip)]
    app_state: AppState,

    #[serde(skip)]
    tx: Sender<Directory>,
    #[serde(skip)]
    rx: Receiver<Directory>,
}

impl SphinxApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self{
        if let Some(storage) = _cc.storage {
            let mut app: SphinxApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.app_state.explorer_dirs = dir_walk(0, &PathBuf::from(app.app_settings.root_dir.clone()), is_dir, sort_by_name).unwrap();
            app.add_dialog.reset();
            return app;
        }
        let mut app: SphinxApp = Default::default();
        app.app_settings.root_dir = ".".to_owned();
        app.app_state.explorer_dirs = dir_walk(0, &PathBuf::from(app.app_settings.root_dir.clone()), is_dir, sort_by_name).unwrap();
        app.add_dialog.reset();
        return app;
    }
}

//starting vals for ui vars
impl Default for SphinxApp { 
    fn default() -> Self {
        let (tx, rx): (Sender<Directory>, Receiver<Directory>) = std::sync::mpsc::channel();

        Self {
            add_dialog: Default::default(),
            app_settings: Default::default(),
            tx,
            rx,
            app_state: Default::default(),
            commit_settings: Default::default(),
        }
    }
}

//UI definition
impl eframe::App for SphinxApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        /////////////////////////Menu-Bar///////////////////////////////////
        egui::TopBottomPanel::top("menu_bar")
        .resizable(false).exact_height(25.0)
        .show(ctx, |frame|{
            #[allow(unused_parens)]
            if(self.add_dialog.open  || self.app_settings.open){frame.disable()}
            if frame.button("settings").clicked(){
                self.app_settings.open = true;
            }
        });
        /////////////////////////Explorer///////////////////////////////////
        egui::SidePanel::left("project_explorer")
            .resizable(false)
            .exact_width(ctx.screen_rect().width()*0.6)
            .show(ctx, |frame| {
                #[allow(unused_parens)]
                if(self.add_dialog.open  || self.app_settings.open){frame.disable()}
                frame.horizontal(|ui|{
                    if ui.button("⟲").clicked() { 
                       refresh_explorer(&self.app_settings.root_dir, self.tx.clone());
                    }
                    ui.add(egui::Label::new("Root Dir:"));
                    ui.add(egui::TextEdit::singleline(&mut self.app_settings.root_dir).desired_width(f32::INFINITY));
                });
                frame.separator();//-------------
                frame.with_layout(egui::Layout::bottom_up(egui::Align::BOTTOM), |ui| {
                    if ui.add_sized(
                        [ui.available_width(),ui.available_height()*0.1],
                        egui::Button::new("Add new project")
                    ).clicked() {
                        self.add_dialog.open = true;
                    };
                    ui.separator();//-------------
                    egui::ScrollArea::vertical().max_height(f32::INFINITY).show(ui, |ui|{
                        ui.visuals_mut().indent_has_left_vline = true;
                        if let Ok(dir) = self.rx.try_recv() {
                            self.app_state.explorer_dirs = dir;
                        }
                        let mut add_dialog = &mut self.add_dialog;
                        explorer_tree(&self.app_state.explorer_dirs, ui, &mut add_dialog, &mut self.app_settings.selected_project_path);
                        self.add_dialog = add_dialog.clone();
                    });
                })
            }
        );
        /////////////////////////Tools///////////////////////////////////
        /////////////////////////Git/////////////////////////////////////
        egui::TopBottomPanel::bottom("git_view")
            .resizable(false)
            .exact_height(ctx.available_rect().height()*0.5)
            .show(ctx, |frame| {
                #[allow(unused_parens)]
                if(self.add_dialog.open  || self.app_settings.open){frame.disable()}
                frame.vertical_centered(|ui| {ui.heading("Git history:");});
                frame.separator();
                #[allow(unused_parens)]
                if(self.app_settings.selected_project_path != String::default()){
                    let git_history = GitWidget::new(&PathBuf::from(&self.app_settings.selected_project_path), &self.commit_settings, self.app_state.git_history.clone()).unwrap();
                    self.app_state.git_history = git_history.clone();
                    frame.add(git_history);
                }else {
                    frame.vertical_centered(|ui| {ui.label("No project selected")});
                }
            }
        );
        /////////////////////////Ideas///////////////////////////////////
        egui::TopBottomPanel::top("idea_view")
            .resizable(false)
            .exact_height(ctx.available_rect().height())
            .show(ctx, |frame| {
                #[allow(unused_parens)]
                if(self.add_dialog.open || self.app_settings.open){frame.disable()}
                
            }
        );
        ////////////////////////Dialogs//////////////////////////////////
        //////////////////////////Add////////////////////////////////////
        if self.add_dialog.open {
            let mut open = self.add_dialog.open;
            egui::Window::new("Add Project..")
                .fixed_size(egui::vec2(220f32, 100f32))
                .anchor(egui::Align2::CENTER_CENTER, [0f32, 0f32])
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Language:");
                    ui.add(DropDownBox::from_iter(&self.add_dialog.known_langs,
                         "lang_dropbox",
                         &mut self.add_dialog.lang, 
                         |ui, text| ui.selectable_label(false, text)));
                    ui.label("Category:");
                    ui.add(DropDownBox::from_iter(&self.add_dialog.known_category,
                        "category_dropbox",
                        &mut self.add_dialog.category, 
                        |ui, text| ui.selectable_label(false, text)));
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.add_dialog.name);
                    if ui.add_sized(
                        [ui.available_width(),ui.available_height()*0.1],
                        egui::Button::new("Create project")
                    ).clicked() {
                        create_project(PathBuf::from(&self.app_settings.root_dir)
                        .join(&self.add_dialog.lang)
                        .join(&self.add_dialog.category)
                        .join(&self.add_dialog.name), &self.add_dialog.lang);
                        self.app_settings.selected_project_path = PathBuf::from(&self.app_settings.root_dir)
                            .join(&self.add_dialog.lang)
                            .join(&self.add_dialog.category)
                            .join(&self.add_dialog.name).to_str().unwrap().to_owned();
                        self.add_dialog.reset();
                    };
                });
            #[allow(unused_parens)]
            if(open==false){
                self.add_dialog.open = open;
                self.add_dialog.reset();
            }
        }
        ////////////////////////Settings/////////////////////////////////
        else if self.app_settings.open {
            let mut open = self.app_settings.open;
            egui::Window::new("Sphinx Settings")
                .fixed_size(egui::vec2(220f32, 100f32))
                .anchor(egui::Align2::CENTER_CENTER, [0f32, 0f32])
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.heading("Commit settings:");
                    ui.label("Git User:");
                    ui.text_edit_singleline(&mut self.commit_settings.git_user);
                    ui.label("Git mail:");
                    ui.text_edit_singleline(&mut self.commit_settings.git_mail);

                });
            #[allow(unused_parens)]
            if(open==false){
                self.app_settings.open = open;
            }
        }
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

