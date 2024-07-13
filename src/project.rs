use crate::sphinx_git::{self, git_init};
// Definitions related to "projects"

use std::{fs::{create_dir_all, File}, io::Write, path::PathBuf, process::Command};


pub fn create_project(path: PathBuf, lang: &str){ //quick creation
    //create dirs
    create_dir_all(&path).unwrap_or_default();
    println!("Created {} project in {}", lang, path.to_str().unwrap());
    match lang {
        "Rust" => {
            println!("Setting up Rust project"); //cargo already calls git init
            Command::new("cargo").arg("init")
            .current_dir(path)
            .spawn()
            .expect("Failed to initialize");
        }
        &_ => {
            println!("Setting up empty project");
            git_init(path);
        },
    }
}