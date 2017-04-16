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

    pub fn changes(&self, repo: &Repository) -> Vec<(String, String, String)> {
        let mut changes = vec![];
        for (property, value) in &self.properties {
            changes.push((
                    String::from(property.as_str().unwrap_or("[unknown]")),
                    String::from(value.as_str().unwrap_or("[unknown]")),
                    String::from(value.as_str().unwrap_or("[unknown]")),
                    ));
        }
        changes
    }

    fn from_values(repo: &Repository, commit: &Commit, name: &str, properties: &Hash) -> Task {

        let mut walk = repo.revwalk().unwrap();
        if let Ok(_) = walk.push(commit.id()) {
            for rev in walk {
            }
        }

        Task {
            name: String::from(name),
            properties: properties.clone(),
        }
    }

}
