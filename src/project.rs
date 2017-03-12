extern crate git2;
extern crate yaml_rust;
use project::git2::Repository;
use project::git2::Reference;
use project::git2::Statuses;
use self::yaml_rust::{YamlLoader, YamlEmitter};

pub struct User {
    pub name: String,
    pub email: String
}

pub struct Task {
    pub id: String
    //reference: Option<Reference<'static>>,
}

pub struct Project {
    pub name: String,
    pub user: User,
    pub task: Task,
}

impl Project {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub fn current() -> Project {
    let repo = Repository::open("/Users/jadencarver/dev/superconductor").unwrap();
    let config = repo.config().unwrap();
    let head = repo.head().unwrap();
    let task_id = head.shorthand().unwrap();
    Project {
        name: String::from("TBD"),
        user: User {
            name:  config.get_string("user.name" ).unwrap_or(String::from("Unknown")),
            email: config.get_string("user.email").unwrap_or(String::from("root@localhost"))
        },
        task: Task {
            id: String::from(task_id)
        }
    }
}
