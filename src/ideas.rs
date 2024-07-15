use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use egui::{Frame, Label,Widget};
use sqlx::mysql::MySqlPool;

use crate::DbSettings;

pub fn create_db_pool(settings: &DbSettings) -> sqlx::Pool<sqlx::MySql>{
    let pool = MySqlPool::connect_lazy(&settings.db_url).unwrap();
    return pool;
}

#[derive(Clone)]
struct Idea{
    id: i32,
    title: String,
    description: String,
    lang: String
}

#[derive(Clone)]
pub struct IdeasBoard {
    idea_list: Arc<Mutex<Vec<Idea>>>,
    last_update: Instant,
}

impl Default for IdeasBoard {
    fn default() -> Self {
        Self {
            idea_list: Arc::new(Mutex::new(Vec::new())),
            last_update: Instant::now() - Duration::from_secs(60),
        }
    }
}

impl IdeasBoard {
    pub fn new(settings: &DbSettings, ideas: IdeasBoard) -> Self {
        if Instant::now().duration_since(ideas.last_update) < Duration::from_secs(20) {
            return ideas;
        }
        else {
            let board: IdeasBoard = IdeasBoard { 
                idea_list: Arc::new(Mutex::new(Vec::new())), 
                last_update: Instant::now() 
            };
            let pool = settings.db_pool.clone();
            let idea_list = board.idea_list.clone();
            tokio::spawn(async move {
                let updated_ideas = Self::update_ideas(pool.expect("DB-pool not initialised")).await;
                *idea_list.lock().unwrap() = updated_ideas;
            });
            board
        }
    }

    async fn update_ideas(pool: MySqlPool)->Vec<Idea>{
        println!("pulling");
        let recs = sqlx::query_as!(Idea, "SELECT * FROM `ideas`").fetch_all(&pool).await.unwrap();
        for rec in &recs{
            println!("rec : {}", rec.title);
        }
        println!("done");
        return recs;
    }

}

impl Widget for IdeasBoard{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui|{
            ui.horizontal(|ui| {
                let button_width = ui.available_width(); 
                if ui.add_sized([button_width, 30.0], egui::Button::new("Commit")).clicked() {
                    println!("Push clicked");
                }
            });
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height() - 30.0)
                .show(ui, |ui| {
                    if let Ok(locked_ideas) = self.idea_list.lock() {
                        for idea in locked_ideas.iter() {
                            Frame::none()
                                .fill(ui.visuals().faint_bg_color)
                                .inner_margin(8.0)
                                .outer_margin(2.0)
                                .rounding(4.0)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.heading(&idea.title);
                                    ui.add(Label::new(&idea.description).truncate())
                                });
                        }
                    }
                })
        }).response
    }
}