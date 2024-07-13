// Definitions related to "projects"

use std::{fs::{create_dir_all, File}, io::Write, path::PathBuf, process::Command};

enum Langs {
    Rust,
}

pub fn create_project(path: PathBuf, lang: &str){ //quick creation
    //create dirs
    create_dir_all(&path).unwrap_or_default();
    println!("Created {} project in {}", lang, path.to_str().unwrap());
    //init git
    //init project (cargo init)
    match lang {
        "Rust" => {Command::new("cargo").arg("init").current_dir(path).spawn().expect("Failed to initialize");}
        &_ => println!("Dont know how to initialize"),
    }
}