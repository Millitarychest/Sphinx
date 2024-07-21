use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use egui::{accesskit::Vec2, Frame, Label, Memory, Widget};
use sqlx::mysql::MySqlPool;

use crate::{AddIdea, DbSettings};

pub fn create_db_pool(settings: &DbSettings) -> sqlx::Pool<sqlx::MySql>{
    let pool = MySqlPool::connect_lazy(&settings.db_url).unwrap();
    return pool;
}

pub async fn insert_idea(settings: DbSettings, idea: &AddIdea){
    let _= sqlx::query!(r#"INSERT INTO `ideas` (`title`, `description`, `lang`) VALUES (?, ?, ?)"#,idea.title.clone(),idea.description.clone(),idea.lang.clone())
        .execute(&settings.db_pool.expect("Pool wasnt initialized properly")).await;
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
    pub fn new(settings: &DbSettings) -> Self {
        let board: IdeasBoard = IdeasBoard { 
            idea_list: Arc::new(Mutex::new(Vec::new())), 
            last_update: Instant::now(),
        };
        let pool = settings.db_pool.clone();
        let idea_list = board.idea_list.clone();
        tokio::spawn(async move {
            let updated_ideas = Self::update_ideas(pool.expect("DB-pool not initialised")).await;
            *idea_list.lock().unwrap() = updated_ideas;
        });
        board
    }

    pub fn new_board(settings: &DbSettings, ideas: &mut IdeasBoard, ui: &mut egui::Ui) -> egui::Response {
        if Instant::now().duration_since(ideas.last_update) < Duration::from_secs(20) {
            let i = ideas.clone();
            return i.ui(ui);
        }
        else {
            let pool = settings.db_pool.clone();
            let idea_list = ideas.idea_list.clone();
            tokio::spawn(async move {
                let updated_ideas = Self::update_ideas(pool.expect("DB-pool not initialised")).await;
                *idea_list.lock().unwrap() = updated_ideas;
            });
            ideas.last_update = Instant::now();
            let i = ideas.clone();
            return i.ui(ui);
        }
    }

    pub fn mark_update_ideas(ideas: &mut IdeasBoard){
        ideas.last_update = Instant::now() - Duration::from_secs(60);
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
            let scroll = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height() - 15.0)
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
                                    ui.horizontal(|ui| {
                                        ui.add(Label::new(egui::RichText::new(&idea.title).heading()).truncate())
                                    });
                                    ui.add(Label::new(&idea.description).truncate())
                                });
                        }
                    }
                });
        }).response
    }
}