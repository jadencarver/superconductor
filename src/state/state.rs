use std::path::Path;
use git2::Repository;
use git2::ObjectType;
use git2::BranchType;
use rand::Rng;
use termion::color;
use rand;

use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlEmitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub task: String,
    pub focus: String,
    pub filter: Option<Filter>,
    pub message: String,
    pub include: Vec<String>,
    pub property: Vec<Property>,
    pub diff: Vec<String>,
    pub save_update: Option<String>,
    pub new_task: Option<String>,
    pub dragged: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Filter {
    pub name: String,
    pub value: String
}

pub enum StateError {
}

impl State {

    pub fn blank() -> State {
        State {
            task: String::from("master"),
            focus: String::new(),
            message: String::new(),
            include: vec![],
            diff: vec![],
            property: vec![],
            save_update: None,
            new_task: None,
            filter: None,
            dragged: None
        }
    }

    pub fn reset(&mut self) {
        self.message = String::new();
        self.property = vec![];
        self.include = vec![];
        self.save_update = None;
        self.new_task = None;
    }

    pub fn reset_with_status(&mut self) {
        self.message = String::new();
        self.property = self.property.iter().filter_map(|p| if p.name == "Status" { Some(p.clone()) } else { None }).collect();
        self.include = vec![];
        self.save_update = None;
        self.new_task = None;
    }

    pub fn apply(&mut self, mut last_state: Option<State>, rng: &mut rand::ThreadRng) -> Result<State, StateError> {
        let new_last_state = self.clone();
        if let Some(ref mut last) = last_state {
            println!("{}{:?}{}", color::Fg(color::LightBlack), last, color::Fg(color::Reset));
            println!("{:?}", self);
            println!("▶ ");
            let repo = Repository::open_from_env().unwrap();
            println!("Applying to repo: {:?}", repo.path());
            if self.task != last.task {
                println!("  {}Task changing{}  {} => {}", color::Fg(color::LightYellow), color::Fg(color::Reset), last.task, self.task);
                if self.dragged.is_some() {
                    println!("  {}Saving last state due to dragging{}", color::Fg(color::LightGreen), color::Fg(color::Reset));
                    last.save_update(repo, rng);
                    self.reset_with_status();
                } else {
                    let switching_to_task = self.task.clone();
                    self.task = last.task.clone();
                    self.save_update(repo, rng);
                    self.task = switching_to_task;
                    self.reset();
                }
            } else {
                println!("  {}Updating task {}{}", color::Fg(color::LightGreen), self.task, color::Fg(color::Reset));
                //self.apply_index(&repo);

                if self.save_update.is_some() || self.new_task.is_some() {
                    self.save_update(repo, rng);
                    self.reset();
                }
            }
        }
        Ok(new_last_state)
    }

    fn save_update(&mut self, repo: Repository, rng: &mut rand::ThreadRng) {
        let mut yaml = String::new();
        {
            // Constructing the properties YAML
            let mut tasks = Hash::new();
            let mut properties = Hash::new();
            let mut emitter = YamlEmitter::new(&mut yaml);
            for property in self.property.clone() {
                properties.insert(Yaml::String(property.name), Yaml::String(property.value));
            }
            tasks.insert(Yaml::String(self.task.clone()), Yaml::Hash(properties));
            emitter.dump(&Yaml::Hash(tasks)).unwrap();
        }
        let message = format!("{}\n{}", self.message, yaml);

        let mut index = repo.index().unwrap();
        index.read(false).unwrap();
        let author = repo.signature().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        let branch = repo.find_branch(&self.task, BranchType::Local);
        match branch {
            Ok(branch) => {
                let head = branch.into_reference();
                let commit = head.peel(ObjectType::Commit).unwrap();
                repo.commit(Some(&head.name().unwrap()), &author, &author, &message, &tree, &[&commit.as_commit().unwrap()]).unwrap();
            }
            Err(_) => {
                println!("Initial Commit");
                repo.commit(Some("refs/heads/master"), &author, &author, &message, &tree, &[]).unwrap();
            }
        };

        println!("  {}Saved changes to {}{} {:?}", color::Fg(color::LightRed), self.task, color::Fg(color::Reset), self);
        if self.new_task.is_some() {
            let num = rng.gen::<u16>();
            let new_task = format!("{:X}", num);
            println!("  {}Creating task {}{}", color::Fg(color::LightBlue), new_task, color::Fg(color::Reset));
            if let Ok(master_branch) = repo.find_branch("master", BranchType::Local) {
                let master = master_branch.into_reference();
                let commit_obj = master.peel(ObjectType::Commit).unwrap();
                let commit = commit_obj.as_commit().unwrap();
                repo.branch(&new_task, &commit, false).unwrap();
                self.task = new_task;
                self.reset();
            }
        }
    }

    fn apply_index(&self, repo: &Repository) {
        let branch = repo.find_branch(&self.task, BranchType::Local);
        let head = match branch {
            Ok(branch) => branch.into_reference(),
            _ => repo.head().unwrap()
        };
        let commit = head.peel(ObjectType::Commit).unwrap();
        let mut index = repo.index().unwrap();

        // apply git index changes only if task is the working directory
        if self.task == repo.head().unwrap().shorthand().unwrap() {
            let to_remove = index.iter().fold(vec![], |mut acc, entry| {
                let entry_path = String::from_utf8_lossy(entry.path.as_ref());
                match self.include.iter().find(|i| i.as_ref() == entry_path) {
                    None => acc.push(entry_path.into_owned()),
                    _ => {}
                };
                acc
            });
            repo.reset_default(Some(&commit), to_remove.iter()).unwrap();
            for change in &self.include {
                let path = Path::new(&change);
                index.add_path(path).unwrap();
            }
            index.write().unwrap();
        }
    }

}
