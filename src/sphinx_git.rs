use std::path::{Path, PathBuf};

use egui::{Label, Ui, Widget};
use git2::{BranchType, Error, Oid, Repository};

// Definitions for git and the git view

pub fn git_init(path: PathBuf){
    let repo = Repository::init(&path).unwrap();
    let _ = create_initial_commit(&repo);
}

fn create_initial_commit(repo: &Repository) -> Result<(), Error> {
    let sig = repo.signature()?;

    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Sphinx setup: Initial commit", &tree, &[])?;

    Ok(())
}

pub struct GitWidget {
    repo: Repository,
    commits: Vec<Commit>,
}

struct Commit {
    id: Oid,
    message: String,
    is_local_head: bool,
    is_remote_head: bool,
}

impl GitWidget {
    pub fn new(repo_path: &Path) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        let mut widget = GitWidget {
            repo,
            commits: Vec::new(),
        };
        widget.update_commits()?;
        Ok(widget)
    }

    fn update_commits(&mut self) -> Result<(), git2::Error> {
        self.commits.clear();
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let local_head = self.repo.head()?.target().unwrap();
        let remote_head = self.repo
            .find_branch("origin/HEAD", BranchType::Remote)
            .ok()
            .and_then(|b| b.get().target());

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            self.commits.push(Commit {
                id: oid,
                message: commit.summary().unwrap_or("").to_string(),
                is_local_head: oid == local_head,
                is_remote_head: Some(oid) == remote_head,
            });
        }

        Ok(())
    }

}


impl Widget for GitWidget {
    fn ui(mut self, ui: &mut Ui) -> egui::Response {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            let available_height = ui.available_height();
            let button_height = 30.0; // Adjust as needed
            let button_padding = 5.0; 
            let total_button_height = button_height + 2.0 * button_padding;

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(available_height - total_button_height)
                .show(ui, |ui| {
                    for (index, commit) in self.commits.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if commit.is_local_head {
                                ui.label("🏠");
                            }
                            if commit.is_remote_head {
                                ui.label("☁️");
                            }
                            ui.add(Label::new(format!("{}",commit.message)).truncate())
                        });
                        
                        if index < self.commits.len() - 1 {
                            ui.add_space(2.0);
                        }
                    }
                    
                    // Add extra space at the bottom to ensure last commit is fully visible
                    ui.add_space(total_button_height);
                });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_space(ui.available_size()); // Push buttons to the bottom
                });
                ui.horizontal(|ui| {
                    let button_width = (ui.available_width() - button_padding) / 2.0;
                    if ui.add_sized([button_width, button_height], egui::Button::new("Pull/Fetch")).clicked() {
                        println!("Pull/Fetch clicked");
                    }
                    if ui.add_sized([button_width, button_height], egui::Button::new("Push")).clicked() {
                        println!("Push clicked");
                    }
                });
            });
        }).response
    }
}