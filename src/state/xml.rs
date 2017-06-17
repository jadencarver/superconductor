use state::State;
use task::Task;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use termion::color;

extern crate git2;
use self::git2::Repository;
use self::git2::StatusOptions;
use self::git2::Delta;
use self::git2::BranchType;
use self::git2::{Diff, DiffDelta, DiffHunk, DiffLine, DiffBinary};
use git2::ObjectType;
use state::Filter;

extern crate md5;
extern crate chrono;
use maud::PreEscaped;
use self::chrono::{TimeZone, FixedOffset};

use yaml_rust::Yaml;

extern crate base64;

use self::base64::encode;

pub fn generate(state: Option<State>) -> String {
    let repo = match Repository::open_from_env() {
        Ok(repo) => repo,
        Err(_) => Repository::init(env::var("GIT_DIR").unwrap_or(String::from("."))).unwrap()
    };
    println!("Generating from repo: {:?}", repo.path());
    let config = repo.config().unwrap();
    
    // If there is no master branch, start the setup
    if repo.find_branch("master", BranchType::Local).is_err() {
        return html! {
            state {
                setup "1"
                @if let Some(state) = state {
                    message (state.message)
                    task {
                        name "master"
                        @for property in state.property {
                            property {
                                name (property.name)
                                value (property.value)
                            }
                        }
                    }
                } @else {
                    task {
                        name "master"
                    }
                }
                (project())
            }
        }.into_string()
    }

    let head = repo.head().unwrap();
    let head_tree_obj = head.peel(ObjectType::Tree).unwrap();
    let head_tree = head_tree_obj.as_tree().unwrap();
    let changes = repo.diff_tree_to_index(Some(&head_tree), None, None).unwrap();

    let branches = repo.branches(Some(BranchType::Local)).unwrap().filter_map(|b|b.ok());
    let all_tasks: Vec<Task> = branches.filter_map(|(branch, _)| {
        let task = Task::from_ref(branch.get());
        if let Some(ref state) = state {
            if task.name != state.task { Some(task) } else { None }
        } else {
            if !branch.is_head() { Some(task) } else { None }
        }
    }).collect();
    
    // Set Filters
    let filter: Option<Filter> = match all_tasks.is_empty() {
        true => Some(Filter {
            name: String::from("Status"),
            value: String::from("Sprint")
        }),
        false => match state {
            Some(ref state) => match state.filter {
                Some(ref filter) if filter.name != "" => state.filter.clone(),
                _ => None,
            },
            None => None
        }
    };

    // Apply Filters
    let mut tasks: Vec<&Task> = match filter {
        Some(ref filter) => {
            let filter_name = Yaml::String(filter.name.clone());
            let filter_by_value = Yaml::String(filter.value.clone());
            all_tasks.iter().filter(|task| {
                match task.get(&repo, &filter_name) {
                    Some(ref value) if *value == filter_by_value => true,
                    Some(ref value) if *value == Yaml::Null => true,
                    None => true,
                    _ => false
                }
            }).collect()
        },
        None => all_tasks.iter().map(|t|t).collect()
    };

    tasks.sort_by(|a, b| {
        let ordinal = Yaml::from_str("Ordinal");
        let ord_a = a.get(&repo, &ordinal).unwrap_or(Yaml::Real(String::from("1.0"))).as_f64().unwrap_or(1.0);
        let ord_b = b.get(&repo, &ordinal).unwrap_or(Yaml::Real(String::from("1.0"))).as_f64().unwrap_or(1.0);
        ord_a.partial_cmp(&ord_b).unwrap()
    });

    let branch = match state.clone() {
        Some(state) => match repo.find_branch(&state.task, BranchType::Local) {
            Ok(branch) => branch.into_reference(),
            Err(_) => head
        },
        None => head
    };

    let task = Task::from_ref(&branch);

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::SORT_REVERSE);
    revwalk.push(branch.target().unwrap()).unwrap();
    if branch.shorthand().unwrap() != "master" {
        revwalk.hide_ref("refs/heads/master").unwrap();
    }

    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let payload = html! {
        state {
            @if let Some(commit) = state.clone() {
                focus (commit.focus)
            }
            user {
                name  (config.get_string("user.name" ).unwrap_or(String::from("Unknown")))
                email (config.get_string("user.email").unwrap_or(String::from("root@localhost")))
            }
            @if let Some(state) = state.clone() {
                message (state.message)
                @if state.property.len() == 0 {
                    @let task = Task::from_ref(&branch) {
                        (render_task(&repo, &task, task.properties(&repo)))
                    }
                } @else {
                    task {
                        name (state.task)
                        @for property in state.property {
                            property {
                                name (property.name)
                                value (property.value)
                            }
                        }
                    }
                }
            } @else {
                @let task = Task::from_ref(&branch) {
                    (render_task(&repo, &task, task.properties(&repo)))
                }
            }
            tasks {
                @if let Some(filter) = filter {
                    @if filter.name != "" {
                        filter {
                            name (filter.name)
                            value (filter.value)
                        }
                    }
                }
                @for task in tasks {
                    (render_task(&repo, &task, task.properties(&repo)))
                }
            }
            log {
                @for (_, rev) in revwalk.enumerate() {
                    @let commit = repo.find_commit(rev.unwrap()).unwrap() {
                        commit {
                            id (commit.id())
                            @let time = commit.time() {
                                timestamp (time.seconds())
                                localtime (FixedOffset::east(time.offset_minutes()*60).timestamp(time.seconds(), 0).to_rfc3339())
                            }
                            user {
                                @let author = commit.author() {
                                    name (author.name().unwrap())
                                    @let email = author.email().unwrap().trim() {
                                        email (email)
                                        image (format!("https://www.gravatar.com/avatar/{:x}?s=64", md5::compute(email.to_lowercase())))
                                    }
                                }
                            }
                            @let mut message = commit.message().unwrap().split("---\n") {
                                message (message.next().unwrap())
                                @let task = Task::from_commit(&branch.shorthand().unwrap(), &commit) {
                                    (render_task(&repo, &task, task.changes(&repo)))
                                }
                            }
                        }
                    }
                }
            }
            @if task.name == "master" {
                (project())
            } @else {
                (properties())
            }
            changes {
                @if let Ok(delta) = changes.stats() {
                    @if delta.files_changed() + delta.insertions() + delta.deletions() > 0 {
                        statistics {
                            files (delta.files_changed())
                            insertions (delta.insertions())
                            deletions (delta.deletions())
                        }
                    }
                }
                @for change in repo.statuses(Some(&mut status_opts)).unwrap().iter() {
                    @let path = change.path().unwrap() {
                        change id=(path.replace("/", "_").replace(".", "_").replace(" ", "_")) {
                            path (path)
                            insertions {}
                            deletions {}
                            included @match change.head_to_index().map(|d| d.status()).unwrap_or(Delta::Unreadable) {
                                Delta::Modified | Delta::Added | Delta::Deleted => "true",
                                _ => "false"
                            }
                            removal @match change.head_to_index().map(|d| d.status()).unwrap_or(Delta::Unreadable) {
                                Delta::Deleted => "true",
                                _ => "false"
                            }
                        }
                    }
                }
            }
            diffs {
                @for change in diff(changes) {
                    (change)
                }
            }
        }
    }.into_string();
    println!("  {}Sent payload of size: {}{}", color::Fg(color::LightGreen), payload.len(), color::Fg(color::Reset));
    payload
}

fn diff(changes: Diff) -> Vec<PreEscaped<String>> {
    let result = RefCell::new(vec![]);
    changes.foreach(&mut |delta: DiffDelta, _: f32| {
        result.borrow_mut().push(html!(
            @if let Some(path) = delta.new_file().path() {
                label (path.to_str().unwrap_or("[invalid]"))
            }
        ));
        true
    }, Some(&mut |delta: DiffDelta, binary: DiffBinary| {
        if binary.contains_data() {
            result.borrow_mut().push(html!(
                img src=(format!("data:image/png,{}", (String::from_utf8_lossy(&binary.new_file().data())))) {}
            ));
        } else {
            let path = delta.new_file().path().unwrap();
            let mut file = File::open(path).unwrap();
            let mut contents = vec![];
            file.read_to_end(&mut contents).unwrap();
            result.borrow_mut().push(html!(
                img src=(format!("data:image/jpeg;base64,{}", encode(&contents))) alt=(format!("{}", path.to_str().unwrap())) {}
            ));
        }
        true
    }), None, Some(&mut |_: DiffDelta, _: Option<DiffHunk>, line: DiffLine| {
        let class = match line.origin() {
            '+' | '>' => "add",
            '-' | '<' => "sub",
            'H' | 'F' => "meta",
            _ => ""
        };
        result.borrow_mut().push(html!(
            span class=(class) (String::from_utf8_lossy(&line.content()))
        ));
        true
    })).unwrap();
    result.into_inner()
}

fn render_task(repo: &Repository, task: &Task, changes: Vec<(String, Option<String>, String)>) -> PreEscaped<String> {
    html!(task {
        name (task.name)
        @for (name, before, value) in changes {
            property {
                name (name)
                @if let Some(before) = before {
                    before (before)
                }
                value (value)
            }
        }
    })
}

fn properties() -> PreEscaped<String> {
    html! {
        properties {
            property {
                name "Status"
                options {
                    option "Sprint"
                    option "In Progress"
                    option "In Review"
                    option "Blocked"
                    option "Done"
                }
            }
            property {
                name "Ordinal"
                value "1.0"
            }
            property {
                name "Estimate"
            }
            property {
                name "Developer"
                value "Jaden Carver <jaden.carver@gmail.com>"
                options {
                    option value="Jaden Carver <jaden.carver@gmail.com>" "Jaden Carver"
                    option value="Bob Dole <bdole69@gmail.com>" "Bob Dole"
                }
            }
            property {
                name "Manager"
                value "Jaden Carver <jaden.carver@gmail.com>"
                options {
                    option value="Jaden Carver <jaden.carver@gmail.com>" "Jaden Carver"
                    option value="Bob Dole <bdole69@gmail.com>" "Bob Dole"
                }
            }
            property {
                name "Description"
            }
        }
    }
}

fn project() -> PreEscaped<String> {
    html! {
        properties {
            property {
                name "Project"
            }
            property {
                name "Status"
                options {
                    option "Sprint"
                    option "In Progress"
                    option "In Review"
                    option "Blocked"
                    option "Done"
                }
            }
            property {
                name "Manager"
                value "Jaden Carver <jaden.carver@gmail.com>"
                options {
                    option value="Jaden Carver <jaden.carver@gmail.com>" "Jaden Carver"
                    option value="Bob Dole <bdole69@gmail.com>" "Bob Dole"
                }
            }
            property {
                name "Description"
            }
        }
    }
}
