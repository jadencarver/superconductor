extern crate git2;
use project::git2::Repository;
use project::git2::Reference;

#[derive(Serialize, Deserialize)]
pub struct Project {
    //reference: Option<Reference<'static>>,
    name: String
}

impl Project {
}

pub fn current() -> Project {
    let repo = Repository::open("/Users/jadencarver/dev/superconductor").unwrap();
    let head = repo.head().unwrap();
    Project { name: String::from(head.shorthand().unwrap()) }
}
