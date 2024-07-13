use std::path::PathBuf;

use git2::{Error, Repository};

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


