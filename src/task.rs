use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml::Hash;

extern crate time;
extern crate git2;
use self::git2::{Repository, ObjectType};
use self::git2::{Commit, Reference, Oid};

#[derive(Debug)]
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
        Task::from_commit(&name, &commit)
    }

    pub fn from_commit(name: &str, commit: &Commit) -> Task {
        let mut messages = commit.message().unwrap().split("---\n");
        let _message = messages.next().unwrap();
        let mut tasks = vec![];
        if let Some(yaml) = messages.next() {
            if let Ok(loader) = YamlLoader::load_from_str(yaml) {
                for yaml in loader.iter() {
                    if let Some(hash) = yaml.as_hash() {
                        for (key, values) in hash {
                            if let Some(key) = key.as_str() {
                                if let Some(values) = values.as_hash() {
                                    tasks.push(Task {
                                        name: String::from(key),
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
        tasks.retain(|c| c.name == name);
        tasks.pop().unwrap_or(Task {
            name: String::from(name),
            commit: Some(commit.id()),
            changes: Hash::new()
        })
    }

    pub fn get(&self, repo: &Repository, property: &Yaml) -> Option<Yaml> {
        if let Some(value) = self.changes.get(&property) {
            Some(value.clone())
        } else if let Some(commit) = self.commit {
            let commit = repo.find_commit(commit).expect("Unable to find commit!");
            let parents = commit.parents().map(|parent| Task::from_commit(&self.name, &parent));
            let mut candidates: Vec<Yaml> = parents.filter_map(|task| task.get(&repo, property)).collect();
            candidates.pop()
        } else {
            None
        }
    }

    pub fn parent(&self, repo: &Repository) -> Option<Task> {
        if let Some(commit_oid) = self.commit {
            let commit = repo.find_commit(commit_oid).unwrap();
            if let Some(parent) = commit.parents().next() {
                Some(Task::from_commit(&self.name, &parent))
            } else { None }
        } else { None }
    }

    pub fn timestamp(&self, repo: &Repository) -> i64 {
        if let Some(commit_oid) = self.commit {
            let commit = repo.find_commit(commit_oid).unwrap();
            commit.time().seconds()
        } else {
            time::get_time().sec
        }
    }

    pub fn changes(&self, repo: &Repository) -> Vec<(String, Option<String>, String)> {
        let mut changes = vec![];
        for (key, value) in self.changes.clone() {
            let after = match value {
                Yaml::String(ref s) => s.clone(),
                Yaml::Integer(i) => format!("{}", i),
                Yaml::Real(ref i) => format!("{}", i),
                Yaml::Boolean(b) => format!("{}", b),
                Yaml::Null => format!(""),
                _ => String::from("[unknown]")
            };
            let before = if let Some(parent) = self.parent(&repo) {
                match parent.get(&repo, &key) {
                    Some(before_value) => {
                        if before_value == value {
                            None
                        } else {
                            match before_value {
                                Yaml::String(ref s) => Some(s.clone()),
                                Yaml::Integer(i) => Some(format!("{}", i)),
                                Yaml::Real(i) => Some(format!("{}", i)),
                                Yaml::Boolean(b) => Some(format!("{}", b)),
                                Yaml::Null => Some(format!("")),
                                _ => None
                            }
                        }
                    }, _ => None
                }
            } else { None };
            changes.push((String::from(key.as_str().unwrap()), before, after));
        };
        changes
    }

    pub fn properties(&self, repo: &Repository) -> Vec<(String, Option<String>, String)> {
        let mut changes = vec![];
        let properties = ["Ordinal","Status","Project","Estimate","Developer","Manager","Description"];
        for property in properties.iter() {
            let prop = Yaml::String(String::from(*property));
            if let Some(value) = self.get(&repo, &prop) {
                let after = match value {
                    Yaml::String(ref s) => s.clone(),
                    Yaml::Integer(i) => format!("{}", i),
                    Yaml::Real(i) => format!("{}", i),
                    Yaml::Boolean(b) => format!("{}", b),
                    Yaml::Null => format!(""),
                    _ => String::from("[unknown]")
                };
                changes.push((String::from(*property), None, after));
            }
        };
        changes
    }
}
