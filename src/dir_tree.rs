// Definitions for Project/Directory-tree Widget
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use egui::CollapsingHeader;

use crate::AddDialog;



#[derive(Default,Clone)]
pub struct Directory {
    pub name: String,
    pub entries: Vec<Directory>,
    pub depth: u32,
    pub path: String,
}

//filters
pub fn is_dir(name: &PathBuf) -> bool {
    return !name.is_dir();
}

//sorts
pub fn sort_by_name(a: &fs::DirEntry, b: &fs::DirEntry) -> Ordering {
    let a_name: String = a.path().file_name().unwrap().to_str().unwrap().into();
    let b_name: String = b.path().file_name().unwrap().to_str().unwrap().into();
    a_name.cmp(&b_name)
}

//logic
pub fn dir_walk(depth: u32,root: &PathBuf,filter: fn(name: &PathBuf) -> bool,compare: fn(a: &fs::DirEntry, b: &fs::DirEntry) -> Ordering,) -> io::Result<Directory> {
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(root)?
            .filter_map(|result| result.ok())
            .collect();
        entries.sort_by(compare);
        let mut directory: Vec<Directory> = Vec::with_capacity(entries.len());
        for e in entries {
            let path = e.path();
            let name: String = path.file_name().unwrap().to_str().unwrap().into();
            if filter(&path) {
                continue;
            };
            if depth >= 3 {
                continue;
            }
            let node = match path {
                path if path.is_dir() => {
                    dir_walk(depth +1,&root.join(name), filter, compare)?
                },
                _ => unreachable!(),
            };
            directory.push(node);
        }
    let name = root
        .file_name()
        .unwrap_or(OsStr::new("."))
        .to_str()
        .unwrap()
        .into();
    Ok(Directory {
        name: name,
        entries: directory,
        depth: depth,
        path: root.to_str().unwrap().to_owned(),
    })
}

#[allow(dead_code)] //keeping for ebuging
pub fn print_tree(root: &str, dir: &Directory) {
    const OTHER_CHILD: &str = "│   "; // prefix: pipe
    const OTHER_ENTRY: &str = "├── "; // connector: tee
    const FINAL_CHILD: &str = "    "; // prefix: no siblings
    const FINAL_ENTRY: &str = "└── "; // connector: elbow

    println!("{}", root);
    let (d, f) = visit(dir, "");
    println!("\n{} directories, {} files", d, f);

    fn visit(node: &Directory, prefix: &str) -> (usize, usize) {
        let mut dirs: usize = 1; // counting this directory
        let mut files: usize = 0;
        let mut count = node.entries.len();
        for entry in &node.entries {
            count -= 1;
            let connector = if count == 0 { FINAL_ENTRY } else { OTHER_ENTRY };
                println!("{}{}{}", prefix, connector, entry.name);
                let new_prefix = format!(
                    "{}{}",
                    prefix,
                    if count == 0 { FINAL_CHILD } else { OTHER_CHILD }
                );
                let (d, f) = visit(&entry, &new_prefix);
                dirs += d;
                files += f;

        }
        (dirs, files)
    }
}

//ui

pub fn explorer_tree(pnode: &Directory, ui: &mut egui::Ui, add_dialog: &mut AddDialog) -> egui::Response {
    let Directory { name, mut entries ,depth, path} = pnode.to_owned();
    if entries.is_empty() {
        let response = ui.selectable_label(false, name.as_str());
        if response.clicked() {
            open_folder(&PathBuf::from(path));
        }
        response
    } else {
        let header = CollapsingHeader::new(name.as_str()).default_open(pnode.depth < 2);
        let id = ui.make_persistent_id(&name);
        let response = header
            .show(ui, |ui| {
                let mut iter = entries.iter_mut().peekable();
                let mut count = 0;
                while iter.peek().is_some() {
                    if iter.peek().is_some_and(|n| n.entries.is_empty()) {
                        ui.indent(0, |ui| {
                            while let Some(node) =
                                iter.peek_mut().filter(|n| n.entries.is_empty())
                            {
                                explorer_tree(node ,ui,add_dialog);
                                iter.next();
                            }
                        });
                    }
                    if let Some(node) = iter.next() {
                        count += 1;
                        ui.push_id((&name, count), |ui| explorer_tree(node ,ui,add_dialog));
                    }
                }
            });
            let header_response = response.header_response;

            header_response.context_menu(|ui|{
                if ui.button("Open in Explorer").clicked() {
                    open_folder(&PathBuf::from(path));
                    ui.close_menu();
                }
                if ui.button("Quick add Project").clicked() {
                    add_dialog.open = true;
                    ui.close_menu();
                }
            });
            

            header_response
    }
}



//handlers
fn open_folder(path: &PathBuf){
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .expect("Failed to open Explorer");
    }
}