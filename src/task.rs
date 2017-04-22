use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml::Hash;

extern crate git2;
use self::git2::{Repository, ObjectType};
use self::git2::{Commit, Reference};

pub struct Task {
    pub name: String,
    changes: Hash
}

impl Task {
    pub fn from_ref(repo: &Repository, reference: &Reference) -> Task {
        let commit_obj = reference.peel(ObjectType::Commit).unwrap();
        let commit = commit_obj.as_commit().unwrap();
        let name = reference.shorthand().unwrap_or("master");
        let mut tasks = Task::from_commit(&repo, &name, &commit);
        tasks.retain(|c| c.name == name);
        tasks.pop().unwrap_or(Task {
            name: String::from(name),
            changes: Hash::new()
        })
    }

    pub fn from_commit(repo: &Repository, name: &str, commit: &Commit) -> Vec<Task> {
        let mut messages = commit.message().unwrap().split("---\n");
        let message = messages.next().unwrap();
        let mut tasks = vec![];
        if let Some(yaml) = messages.next() {
            if let Ok(loader) = YamlLoader::load_from_str(yaml) {
                for yaml in loader.iter() {
                    if let Some(hash) = yaml.as_hash() {
                        for (key, values) in hash {
                            if let Some(key) = key.as_str() {
                                if let Some(values) = values.as_hash() {
                                    tasks.push(Task {
                                        name: String::from(name),
                                        changes: values.clone(),
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

    pub fn changes(&self, repo: &Repository, commit: &Commit) -> Vec<(String, Option<String>, String)> {
        let mut changes = vec![];
        let mut parents = vec![];

        for parent in commit.parents() {
            let mut walk = repo.revwalk().unwrap();
            walk.set_sorting(git2::SORT_REVERSE);
            if let Ok(_) = walk.push(parent.id()) {
                for rev in walk {
                    let parent_commit = repo.find_commit(rev.unwrap()).unwrap();
                    parents.append(&mut Task::from_commit(repo, &self.name, &parent_commit));
                }
            }
        }

        for (property, value) in &self.changes {
            let name = String::from(property.as_str().unwrap_or("[unknown]"));
            let after = match value {
                &Yaml::String(ref s) => s.clone(),
                &Yaml::Integer(i) => format!("{}", i),
                &Yaml::Boolean(b) => format!("{}", b),
                _ => String::from("[unknown]")
            };
            let mut before = None;
            for parent in parents.iter() {
                if let Some(value) = parent.changes.get(property) {
                    before = match value {
                        &Yaml::String(ref s) => Some(s.clone()),
                        &Yaml::Integer(i) => Some(format!("{}", i)),
                        &Yaml::Boolean(b) => Some(format!("{}", b)),
                        _ => None
                    }
                }
            }
            if let Some(before) = before {
                if before != after {
                    changes.push((name, Some(before), after));
                }
            } else {
                changes.push((name, before, after));
            }
        }
        changes
    }

}
