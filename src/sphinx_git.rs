use std::{path::{Path, PathBuf}, time::{Duration, Instant}};

use egui::{Label, Ui, Widget};
use git2::{BranchType, Error, IndexAddOption, Repository, Signature};

use crate::CommitSettings;

// Definitions for git and the git view

pub fn git_init(path: PathBuf) -> Repository{
    let repo = Repository::init(&path).unwrap();
    let _ = create_initial_commit(&repo);
    repo
}

fn create_initial_commit(repo: &Repository) -> Result<(), Error> {
    let sig = repo.signature()?;

    let tree_id = {
        let mut index = repo.index()?;
        let _ = index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None);
        index.write_tree()?
    };

    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Sphinx setup: Initial commit", &tree, &[])?;

    Ok(())
}

fn commit(repo: Repository, settings: CommitSettings) {
    let signature = Signature::now(&settings.git_user, &settings.git_mail).unwrap();

    // Get the index and write a tree
    let mut index = repo.index().unwrap();
    let _ = index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None);
    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();

    // Get the parent commit
    let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();

    // Create the commit
    let _ = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Sphinx commit",
        &tree,
        &[&parent_commit],
    );

}


#[derive(Clone)]
pub struct GitWidget {
    pub repo: PathBuf,
    state: GitState,
    last_update: Instant,
    settings: CommitSettings
}

impl Default for GitWidget {
    fn default() -> Self {
        Self { 
            repo: Default::default(), 
            state: Default::default(), 
            last_update: Instant::now(),
            settings: CommitSettings::default() 
        }
    }
}

#[derive(Default,Clone)]
struct GitState {
    commits: Vec<Commit>,
}

#[derive(Default,Clone)]
struct Commit {
    message: String,
    is_local_head: bool,
    is_remote_head: bool,
}

impl GitWidget {
    pub fn new(repo_path: &Path, git_settings: &CommitSettings, git: GitWidget) -> Result<Self, git2::Error> {
        
        let state = GitState{
            commits: Vec::new(),
        };
        let mut widget = GitWidget {
            repo: repo_path.to_path_buf(),
            state: state,
            settings: git_settings.clone(),
            last_update: Instant::now()
        };
        if repo_path != git.repo || Instant::now().duration_since(git.last_update)>Duration::from_secs(20) {
            widget.update_commits()?;
            Ok(widget)
        }
        else {
            Ok(git)
        }
        
    }

    fn update_commits(&mut self) -> Result<(), git2::Error> {
        let repo = Repository::open(&self.repo).unwrap_or(git_init(self.repo.clone()));
        self.state.commits.clear();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let local_head = repo.head()?.target().unwrap();
        let remote_head = repo
            .find_branch("origin/HEAD", BranchType::Remote)
            .ok()
            .and_then(|b| b.get().target());

        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            self.state.commits.push(Commit {
                message: commit.summary().unwrap_or("").to_string(),
                is_local_head: oid == local_head,
                is_remote_head: Some(oid) == remote_head,
            });
        }

        Ok(())
    }

}


impl Widget for GitWidget {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            let available_height = ui.available_height();
            let button_height = 30.0; // Adjust as needed
            let button_padding = 5.0; 
            let total_button_height = button_height + 2.0 * button_padding;

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(available_height - total_button_height)
                .show(ui, |ui| {
                    for (index, commit) in self.state.commits.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if commit.is_local_head {
                                ui.label("🏠");
                            }
                            if commit.is_remote_head {
                                ui.label("☁️");
                            }
                            ui.add(Label::new(format!("{}",commit.message)).truncate())
                        });
                        
                        if index < self.state.commits.len() - 1 {
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
                    let button_width = ui.available_width() - button_padding; 
                    if ui.add_sized([button_width, button_height], egui::Button::new("Commit")).clicked() {
                        commit(Repository::open(&self.repo).unwrap_or(git_init(self.repo.clone())), self.settings);
                        
                    }
                });
            });
        }).response
    }
}