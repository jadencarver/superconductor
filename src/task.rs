use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::yaml::Hash;

extern crate git2;
use self::git2::Repository;
use self::git2::Commit;

pub struct Task {
    pub name: String,
    properties: Hash
}

impl Task {
    pub fn from_commit(repo: &Repository, commit: &Commit, message: &str) -> Vec<Task> {
        let mut tasks = vec![];
        if let Ok(loader) = YamlLoader::load_from_str(message) {
            for yaml in loader.iter() {
                if let Some(hash) = yaml.as_hash() {
                    for (key, values) in hash {
                        if let Some(key) = key.as_str() {
                            if let Some(values) = values.as_hash() {
                                let mut task = Task::from_values(repo, commit, key, values);
                                tasks.push(task);
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
                    let mut parent_message = parent_commit.message().unwrap().split("---\n");
                    parent_message.next().unwrap();
                    if let Some(yaml) = parent_message.next() {
                        parents.append(&mut Task::from_commit(repo, &parent_commit, yaml));
                    }
                }
            }
        }

        for (property, value) in &self.properties {
            let name = String::from(property.as_str().unwrap_or("[unknown]"));
            let after = match value {
                &Yaml::String(ref s) => s.clone(),
                &Yaml::Integer(i) => format!("{}", i),
                &Yaml::Boolean(b) => format!("{}", b),
                _ => String::from("[unknown]")
            };
            let mut before = None;
            for parent in parents.iter() {
                if let Some(value) = parent.properties.get(property) {
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

    fn from_values(repo: &Repository, commit: &Commit, name: &str, properties: &Hash) -> Task {
        let properties = properties.clone();

        Task {
            name: String::from(name),
            properties: properties,
        }
    }

}
