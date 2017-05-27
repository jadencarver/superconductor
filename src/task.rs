use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml::Hash;

extern crate git2;
use self::git2::{Repository, ObjectType};
use self::git2::{Commit, Reference, Oid};

pub struct Task {
    pub name: String,
    commit: Option<Oid>,
    changes: Hash
}

impl Task {
    pub fn from_ref(reference: &Reference) -> Task {
        let commit_obj = reference.peel(ObjectType::Commit).unwrap();
        let commit = commit_obj.as_commit().unwrap();
        let name = reference.shorthand().unwrap_or("master");
        let mut tasks = Task::from_commit(&name, &commit);
        tasks.retain(|c| c.name == name);
        tasks.pop().unwrap_or(Task {
            name: String::from(name),
            changes: Hash::new(),
            commit: None
        })
    }

    pub fn from_commit(name: &str, commit: &Commit) -> Vec<Task> {
        let mut messages = commit.message().unwrap().split("---\n");
        let _message = messages.next().unwrap();
        let mut tasks = vec![];
        if let Some(yaml) = messages.next() {
            if let Ok(loader) = YamlLoader::load_from_str(yaml) {
                for yaml in loader.iter() {
                    if let Some(hash) = yaml.as_hash() {
                        for (key, values) in hash {
                            if let Some(_key) = key.as_str() {
                                if let Some(values) = values.as_hash() {
                                    tasks.push(Task {
                                        name: String::from(name),
                                        changes: values.clone(),
                                        commit: Some(commit.id())
                                    });
                                }
                            }
                        }
                    }
                }
            }
        };
        tasks
    }

    pub fn get(&self, repo: &Repository, property: &Yaml) -> Option<Yaml> {
        if let Some(value) = self.changes.get(&property) {
            Some(value.clone())
        } else if let Some(commit) = self.commit {
            let commit = repo.find_commit(commit).expect("Unable to find commit!");
            let mut parents = vec![];
            let mut candidates = vec![];
            for parent in commit.parents() {
                let mut tasks = Task::from_commit(&self.name, &parent);
                parents.append(&mut tasks);
            }
            for parent in parents.iter() {
                if let Some(value) = parent.get(&repo, property) {
                    candidates.push(value);
                }
            }
            candidates.pop()
        } else {
            None
        }
    }

    pub fn changes(&self, repo: &Repository) -> Vec<(String, Option<String>, String)> {
        let mut changes = vec![];
        if let Some(_commit_oid) = self.commit {
            let properties = ["Status","Estimate","Developer","Description"];
            for property in properties.iter() {
                let prop = Yaml::String(String::from(*property));
                if let Some(value) = self.get(&repo, &prop) {
                    let after = match value {
                        Yaml::String(ref s) => s.clone(),
                        Yaml::Integer(i) => format!("{}", i),
                        Yaml::Boolean(b) => format!("{}", b),
                        _ => String::from("[unknown]")
                    };
                    changes.push((String::from(*property), None, after));
                }
            }
        }
        changes
    }
}
